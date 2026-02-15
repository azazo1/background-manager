#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum ErrorKind {
    #[error("app dir error")]
    DirUnknown,
    #[error("database error")]
    Db,
    #[error("io error")]
    Io,
    #[error("toml deserialzing error")]
    TomlDer,
    #[error("toml serializing error")]
    TomlSer,
}

#[derive(thiserror::Error, Debug)]
#[error("{kind}{}", .message.as_ref().map(|m| format!(": {m}")).unwrap_or_default())]
pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: None,
            source: None,
        }
    }

    pub(crate) fn with_message(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: Some(message.into()),
            source: None,
        }
    }

    pub(crate) fn with_source(
        kind: ErrorKind,
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    ) -> Self {
        Self {
            kind,
            message: Some(message.into()),
            source: Some(source),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}
