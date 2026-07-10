mod adhan;
mod dbus_iface;
mod notifications;
mod registry;

use std::sync::Arc;

use boloot_cal_core::install_salah_panic_hook;
use dbus_iface::CalendarService;
use registry::ServiceRegistry;
use tracing::info;
use zbus::connection;

const SERVICE_NAME: &str = "org.boloot.Calendar";
const OBJECT_PATH: &str = "/org/boloot/Calendar";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_salah_panic_hook();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "boloot_calendard=info".into()),
        )
        .init();

    let registry = Arc::new(ServiceRegistry::new());
    notifications::spawn_notification_loop(registry.clone());
    let iface = CalendarService::new(registry);

    let _connection = connection::Builder::system()?
        .name(SERVICE_NAME)?
        .serve_at(OBJECT_PATH, iface)?
        .build()
        .await?;

    info!("{SERVICE_NAME} running on system bus at {OBJECT_PATH}");

    std::future::pending::<()>().await;
    Ok(())
}
