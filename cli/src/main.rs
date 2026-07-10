use anyhow::{Context, Result};
use boloot_cal_core::{
    install_salah_panic_hook, local_today, BolootConfig, BolootService, CountryProfile, APP_NAME,
    DONATE_BTC, DONATE_USDT_TRC20, WEBSITE, WEBSITE_LABEL,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "boloot-calendar-ctl", about = "BOLOOT Persian Calendar control tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show service status and today's date
    Status,
    /// Show today's formatted date
    Date,
    /// Show prayer times
    Prayer {
        #[arg(long)]
        city: Option<String>,
        #[arg(long)]
        date: Option<String>,
    },
    /// List holidays for a month
    Holidays {
        #[arg(long, default_value_t = 1404)]
        year: i32,
        #[arg(long, default_value_t = 1)]
        month: u8,
    },
    /// List today's holidays as JSON
    HolidaysToday,
    /// Export settings to JSON
    Export,
    /// Import settings from JSON file
    Import {
        path: String,
    },
    /// Apply system locale hook (requires root)
    ApplySystem,
    /// Show top bar preview text
    Preview,
    /// Show month calendar view as JSON
    Month {
        #[arg(long, default_value_t = 0)]
        year: i32,
        #[arg(long, default_value_t = 0)]
        month: i32,
    },
    /// Detect system locale (country, language, numerals)
    DetectLocale,
    /// Stop adhan playback via D-Bus
    StopAdhan,
}

fn main() -> Result<()> {
    install_salah_panic_hook();
    let cli = Cli::parse();

    if let Commands::Import { path } = &cli.command {
        return import_settings(path);
    }

    let mut service = BolootService::from_config_file().context("load config")?;

    match cli.command {
        Commands::Status => {
            let view = service.today_view()?;
            println!("{APP_NAME}");
            println!("{WEBSITE_LABEL}: {WEBSITE}");
            println!("Support USDT (TRC20): {DONATE_USDT_TRC20}");
            println!("Support BTC: {DONATE_BTC}");
            println!("Country: {:?}", service.config().calendar.country);
            println!("Today: {}", view.primary);
            if let Some(sec) = view.secondary {
                println!("Secondary: {sec}");
            }
            let holidays = service.holidays_for_today()?;
            if !holidays.is_empty() {
                println!("Holidays: {}", holidays.iter().map(|h| h.name.as_str()).collect::<Vec<_>>().join(", "));
            }
        }
        Commands::Date => {
            let view = service.today_view()?;
            println!("{}", view.primary);
        }
        Commands::Prayer { city, date } => {
            if let Some(city) = city {
                service.config_mut().prayer.city = city;
            }
            let day = if let Some(date) = date {
                chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                    .with_context(|| format!("invalid date: {date}"))?
            } else {
                local_today()
            };
            let schedule = service.prayer_for(day)?;
            println!("شهر: {}", schedule.times.city);
            for entry in &schedule.times.entries {
                println!("{}: {}", entry.label, entry.time);
            }
            if let Some(next) = schedule.next {
                println!("بعدی: {} ({})", next.label, next.time);
            }
        }
        Commands::Holidays { year, month } => {
            let holidays = service.holidays_for_month(year, month);
            for holiday in holidays {
                println!("{}/{}/{} - {}", holiday.jalali_day, holiday.jalali_month, year, holiday.name);
            }
        }
        Commands::HolidaysToday => {
            let holidays = service.holidays_for_today()?;
            println!("{}", serde_json::to_string(&holidays)?);
        }
        Commands::Export => {
            println!("{}", service.config().export_json()?);
        }
        Commands::Import { .. } => unreachable!("handled in main"),
        Commands::ApplySystem => {
            apply_system_locale(service.config())?;
            println!("system locale hook applied");
        }
        Commands::Preview => {
            println!("{}", service.top_bar_text()?);
        }
        Commands::Month { year, month } => {
            let y = if year <= 0 { None } else { Some(year) };
            let m = if month <= 0 {
                None
            } else if (1..=12).contains(&month) {
                Some(month as u8)
            } else {
                anyhow::bail!("month must be 1–12, got {month}");
            };
            let view = service.month_view(y, m)?;
            println!("{}", serde_json::to_string_pretty(&view)?);
        }
        Commands::DetectLocale => {
            let detected = boloot_cal_core::detect_from_env();
            println!("{}", serde_json::to_string_pretty(&detected)?);
        }
        Commands::StopAdhan => {
            let stopped = stop_adhan_via_dbus()?;
            if stopped {
                println!("adhan stopped");
            } else {
                println!("no adhan playing");
            }
        }
    }

    Ok(())
}

fn stop_adhan_via_dbus() -> Result<bool> {
    let connection = zbus::blocking::Connection::system().context("connect to system bus")?;
    let reply: bool = connection
        .call_method(
            Some("org.boloot.Calendar"),
            "/org/boloot/Calendar",
            Some("org.boloot.Calendar"),
            "StopAdhan",
            &(),
        )
        .context("StopAdhan D-Bus call")?
        .body()
        .deserialize()
        .context("decode StopAdhan reply")?;
    Ok(reply)
}

fn import_settings(path: &str) -> Result<()> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    let config = BolootConfig::import_json(&raw)?;
    config.validate().context("invalid settings")?;
    let mut service = BolootService::new(BolootConfig::default())?;
    service.apply_settings(config);
    service.save_config()?;
    println!("imported settings from {path}");
    Ok(())
}

fn apply_system_locale(config: &BolootConfig) -> Result<()> {
    let locale = boloot_cal_core::LocaleProfile::resolve(
        config.calendar.country.clone(),
        config.calendar.language,
    );
    let profile = format!(
        r#"# Generated by BOLOOT Persian Calendar — boloot.ir
export LC_TIME="{locale}@calendar=persian"
export BOLOOT_CALENDAR_PROFILE="{profile}"
export BOLOOT_CALENDAR_TIMEZONE="{tz}"
"#,
        locale = locale.locale_code,
        profile = profile_name(&config.calendar.country),
        tz = config.calendar.timezone,
    );

    let tmp = std::env::temp_dir().join(format!(
        "boloot-apply-system-{}.sh",
        std::process::id()
    ));
    std::fs::write(&tmp, profile).with_context(|| format!("write temp profile {tmp:?}"))?;

    let helper = resolve_apply_system_helper();
    let status = std::process::Command::new("pkexec")
        .arg(&helper)
        .arg(&tmp)
        .status()
        .with_context(|| "run pkexec (install polkit and boloot-calendar-apply-system)")?;

    let _ = std::fs::remove_file(&tmp);

    if !status.success() {
        anyhow::bail!("system locale hook was not applied (authorization denied or helper missing)");
    }
    Ok(())
}

fn resolve_apply_system_helper() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("BOLOOT_APPLY_SYSTEM_HELPER") {
        return path.into();
    }
    for candidate in [
        "/usr/libexec/boloot-calendar-apply-system",
        "/usr/local/libexec/boloot-calendar-apply-system",
    ] {
        let path = std::path::PathBuf::from(candidate);
        if path.exists() {
            return path;
        }
    }
    std::path::PathBuf::from("/usr/libexec/boloot-calendar-apply-system")
}

fn profile_name(country: &CountryProfile) -> &str {
    country.as_str()
}
