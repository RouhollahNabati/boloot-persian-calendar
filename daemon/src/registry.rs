use std::collections::HashMap;
use std::sync::Arc;

use boloot_cal_core::{home_dir_for_uid, BolootService};
use tokio::sync::RwLock;
use zbus::message::Header;
use zbus::Connection;
use zbus::fdo::DBusProxy;

pub struct ServiceRegistry {
    services: RwLock<HashMap<u32, Arc<RwLock<BolootService>>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, uid: u32) -> zbus::fdo::Result<Arc<RwLock<BolootService>>> {
        {
            let map = self.services.read().await;
            if let Some(service) = map.get(&uid) {
                return Ok(service.clone());
            }
        }

        let service = Arc::new(RwLock::new(
            BolootService::from_config_for_uid(uid)
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?,
        ));
        let mut map = self.services.write().await;
        let entry = map.entry(uid).or_insert_with(|| service.clone());
        Ok(entry.clone())
    }
}

pub async fn caller_uid(conn: &Connection, hdr: &Header<'_>) -> zbus::fdo::Result<u32> {
    let sender = hdr
        .sender()
        .ok_or_else(|| zbus::fdo::Error::Failed("no dbus sender".into()))?;
    let dbus = DBusProxy::new(conn)
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
    let creds = dbus
        .get_connection_credentials(sender.to_owned().into())
        .await
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
    creds
        .unix_user_id()
        .ok_or_else(|| zbus::fdo::Error::Failed("peer uid unavailable".into()))
}

pub fn caller_can_persist(uid: u32) -> zbus::fdo::Result<()> {
    if home_dir_for_uid(uid).is_some() {
        return Ok(());
    }
    Err(zbus::fdo::Error::AccessDenied(
        "settings cannot be saved for this account".into(),
    ))
}
