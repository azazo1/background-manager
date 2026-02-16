use migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::{RwLock, RwLockReadGuard};
use tracing::warn;

use crate::config::{AppConfig, config_dir, db_path};

pub(crate) struct AppState {
    config: RwLock<AppConfig>,
    db: RwLock<DatabaseConnection>,
}

impl AppState {
    async fn open_database() -> crate::Result<DatabaseConnection> {
        let db_path = db_path()?;
        let mut url = url::Url::from_file_path(db_path).unwrap();
        url.set_query(Some("mode=rwc"));
        let mut url = url.to_string();
        url.replace_range(0..4, "sqlite");
        let db = Database::connect(url).await.map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Db,
                "failed to connect to sqlite database",
                Box::new(e),
            )
        })?;
        migration::Migrator::up(&db, None).await.map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Db,
                "failed to migrate database",
                Box::new(e),
            )
        })?;
        Ok(db)
    }

    pub(crate) async fn build() -> crate::Result<Self> {
        let db = Self::open_database().await?;
        let config = AppConfig::load_from_file(config_dir()?.join("config.toml")).await?;
        Ok(AppState {
            config: RwLock::new(config),
            db: RwLock::new(db),
        })
    }

    pub(crate) async fn reconnect_db(&self) -> crate::Result<()> {
        let mut db = self.db.write().await;
        if let Err(e) = db.close_by_ref().await {
            warn!("failed to close database: {e:?}");
        };
        *db = Self::open_database().await?;
        Ok(())
    }

    pub(crate) async fn db(&self) -> RwLockReadGuard<'_, DatabaseConnection> {
        self.db.read().await
    }
}
