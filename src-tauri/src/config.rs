use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
};

use crate::utils::EnsureDirExists;

pub(crate) fn data_dir() -> crate::Result<PathBuf> {
    dirs_next::data_dir()
        .ok_or_else(|| {
            crate::Error::with_message(
                crate::ErrorKind::DirUnknown,
                "can't determine data directory",
            )
        })?
        .join(PKG_NAME)
        .ensure_dir_exists()
}

pub(crate) fn db_path() -> crate::Result<PathBuf> {
    Ok(data_dir()?.join("data.db"))
}

pub(crate) fn log_dir() -> crate::Result<PathBuf> {
    data_dir()?.join("logs").ensure_dir_exists()
}

pub(crate) fn config_dir() -> crate::Result<PathBuf> {
    dirs_next::config_dir()
        .ok_or_else(|| {
            crate::Error::with_message(
                crate::ErrorKind::DirUnknown,
                "can't determine config directory",
            )
        })?
        .join(PKG_NAME)
        .ensure_dir_exists()
}

pub(crate) const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Serialize, Deserialize, bon::Builder, Clone)]
pub(crate) struct AppConfig {
    #[serde(skip)]
    #[builder(skip)]
    file: Option<PathBuf>,

    /// 后台启动应用.
    #[serde(default)]
    #[builder(default = false)]
    quiet_launch: bool,
}

impl AppConfig {
    pub(crate) async fn load_from_file(file: impl AsRef<Path>) -> crate::Result<Self> {
        let failed_to_open = |e| {
            crate::Error::with_source(
                crate::ErrorKind::Io,
                format!("failed to open config file: {}", file.as_ref().display()),
                Box::new(e),
            )
        };
        // 不存在则尝试创建, 如果无法创建, 那么报错.
        let mut f = File::options()
            .read(true)
            .create(true)
            .write(true)
            .truncate(false)
            .open(file.as_ref())
            .await
            .map_err(failed_to_open)?;
        let metadata = f.metadata().await.map_err(failed_to_open)?;
        let mut content = vec![0u8; metadata.len() as usize];
        f.read_exact(&mut content).await.map_err(failed_to_open)?;

        let mut config: AppConfig = toml::from_slice(&content).map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::TomlDer,
                "failed to deserialize config",
                Box::new(e),
            )
        })?;
        config.file = Some(file.as_ref().into());
        Ok(config)
    }

    pub(crate) async fn save(&self) -> crate::Result<()> {
        let Some(file) = &self.file else {
            return Err(crate::Error::with_message(
                crate::ErrorKind::Io,
                "no file bounded",
            ));
        };
        if file.as_os_str().is_empty() {
            return Err(crate::Error::with_message(
                crate::ErrorKind::Io,
                "invalid saving file path: [empty]",
            ));
        }
        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::TomlSer,
                "failed to serialize config",
                Box::new(e),
            )
        })?;
        fs::write(&file, content).await.map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Io,
                format!("failed to write config file: {}", file.display()),
                Box::new(e),
            )
        })
    }

    pub(crate) fn update(&mut self, config: Self) {
        let file = self.file.take();
        *self = config;
        self.file = file;
    }

    #[inline]
    #[must_use]
    pub(crate) fn quiet_launch(&self) -> bool {
        self.quiet_launch
    }
}
