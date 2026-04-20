use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("model '{0}' not found in local cache")]
    ModelNotFound(String),

    #[error("quantization '{0}' not found for model '{1}'")]
    QuantNotFound(String, String),

    #[error("quantization '{0}' exists but is not downloaded for model '{1}'")]
    QuantNotDownloaded(String, String),

    #[error("no downloaded quantization found for model '{0}'")]
    NoDownloadedQuant(String),

    #[error("model manager not initialized; call ml_model_init() first")]
    NotInitialized,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("hub error: {0}")]
    Hub(String),
}
