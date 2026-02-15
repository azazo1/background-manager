use sea_orm::{Database, DatabaseConnection};
use tauri::async_runtime::RwLock;

use crate::config::{config_dir, db_path, AppConfig};

pub(crate) struct AppState {
    pub(crate) config: RwLock<AppConfig>,
    pub(crate) db: RwLock<DatabaseConnection>,
}

impl AppState {
    async fn open_database() -> crate::Result<DatabaseConnection> {
        let db_path = db_path()?;
        let mut url = url::Url::from_file_path(db_path).unwrap();
        url.set_query(Some("mode=rwc"));
        let mut url = url.to_string();
        url.replace_range(0..4, "sqlite");
        Database::connect(url).await.map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Db,
                "failed to connect to sqlite database",
                Box::new(e),
            )
        })
    }

    pub(crate) async fn build() -> crate::Result<Self> {
        let db = Self::open_database().await?;
        let config = AppConfig::load_from_file(config_dir()?.join("config.toml")).await?;
        Ok(AppState {
            config: RwLock::new(config),
            db: RwLock::new(db),
        })
    }
}
