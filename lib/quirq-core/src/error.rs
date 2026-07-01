use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("hamlib error {code}: {description}")]
    Hamlib { code: i32, description: String },

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("operation not supported by this radio")]
    NotSupported,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
