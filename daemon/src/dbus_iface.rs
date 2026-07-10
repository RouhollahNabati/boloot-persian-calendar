use boloot_cal_core::BolootConfig;
use std::sync::Arc;
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::message::Header;
use zbus::Connection;

use crate::adhan;
use crate::registry::{caller_can_persist, caller_uid, ServiceRegistry};

pub struct CalendarService {
    pub registry: Arc<ServiceRegistry>,
}

impl CalendarService {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }
}

#[interface(name = "org.boloot.Calendar")]
impl CalendarService {
    async fn get_date(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<String> {
        let uid = caller_uid(conn, &hdr).await?;
        let service = self.registry.get(uid).await?;
        let service = service.read().await;
        let view = service
            .today_view()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(view.primary)
    }

    async fn get_calendar_view(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<String> {
        let uid = caller_uid(conn, &hdr).await?;
        let service = self.registry.get(uid).await?;
        let service = service.read().await;
        let view = service
            .today_view()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        serde_json::to_string(&view)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn get_month_view(
        &self,
        year: i32,
        month: i32,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<String> {
        let uid = caller_uid(conn, &hdr).await?;
        let service = self.registry.get(uid).await?;
        let service = service.read().await;
        let year = if year <= 0 { None } else { Some(year) };
        let month = if month <= 0 {
            None
        } else if (1..=12).contains(&month) {
            Some(month as u8)
        } else {
            return Err(zbus::fdo::Error::InvalidArgs(format!(
                "month must be 1–12, got {month}"
            )));
        };
        let view = service
            .month_view(year, month)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        serde_json::to_string(&view)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn get_top_bar_text(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<String> {
        let uid = caller_uid(conn, &hdr).await?;
        let service = self.registry.get(uid).await?;
        let service = service.read().await;
        service
            .top_bar_text()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn get_holidays_today(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<String> {
        let uid = caller_uid(conn, &hdr).await?;
        let service = self.registry.get(uid).await?;
        let service = service.read().await;
        let holidays = service
            .holidays_for_today()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        serde_json::to_string(&holidays)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn get_prayer_times(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<String> {
        let uid = caller_uid(conn, &hdr).await?;
        let service = self.registry.get(uid).await?;
        let service = service.read().await;
        let schedule = service
            .prayer_today()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        serde_json::to_string(&schedule)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn get_settings(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<String> {
        let uid = caller_uid(conn, &hdr).await?;
        let service = self.registry.get(uid).await?;
        let service = service.read().await;
        service
            .config()
            .export_json()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn set_settings(
        &self,
        config_json: &str,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> zbus::fdo::Result<()> {
        let uid = caller_uid(conn, &hdr).await?;
        caller_can_persist(uid)?;
        let config = BolootConfig::import_json(config_json)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(e.to_string()))?;
        config
            .validate()
            .map_err(|e| zbus::fdo::Error::InvalidArgs(e.to_string()))?;
        let service = self.registry.get(uid).await?;
        let mut service = service.write().await;
        service.apply_settings(config);
        service
            .save_config_for_uid(uid)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        emitter
            .settings_changed()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    async fn reload(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> zbus::fdo::Result<()> {
        let uid = caller_uid(conn, &hdr).await?;
        let service = self.registry.get(uid).await?;
        {
            let mut service = service.write().await;
            service
                .reload_for_uid(uid)
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        }
        emitter
            .settings_changed()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    async fn is_adhan_playing(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<bool> {
        let uid = caller_uid(conn, &hdr).await?;
        Ok(adhan::is_adhan_playing_for_uid(uid))
    }

    async fn stop_adhan(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<bool> {
        let uid = caller_uid(conn, &hdr).await?;
        Ok(adhan::stop_adhan_for_uid(uid))
    }

    #[zbus(signal)]
    async fn settings_changed(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}
