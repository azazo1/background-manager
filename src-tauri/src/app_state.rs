use sea_orm::{ActiveModelTrait, Database, DatabaseConnection};
use tauri::async_runtime::RwLock;

use crate::{
    config::{config_dir, db_path, AppConfig},
    task::Task,
};

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

    pub(crate) async fn save_task(&self, task: Task) -> crate::Result<()> {
        let am: entity::tasks::ActiveModel = task.into();
        am.save(&*self.db.read().await).await.map_err(|e| {
            crate::Error::with_source(crate::ErrorKind::Db, "failed to insert task", Box::new(e))
        })?;
        Ok(())
    }
}
