use std::path::Path;

use tokio::io;

pub(crate) trait EnsureDirExists: Sized {
    fn ensure_dir_exists(self) -> crate::Result<Self>;
}

impl<T> EnsureDirExists for T
where
    T: AsRef<Path>,
{
    fn ensure_dir_exists(self) -> crate::Result<Self> {
        std::fs::create_dir_all(self.as_ref()).map_err(|e| {
            crate::Error::with_source(
                crate::ErrorKind::Io,
                format!("failed to create directory: {}", self.as_ref().display()),
                Box::new(e),
            )
        })?;
        Ok(self)
    }
}
