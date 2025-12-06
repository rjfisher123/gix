use thiserror::Error;

#[derive(Error, Debug)]
pub enum GixError {
    #[error("Cryptographic verification failed")]
    CryptoFailure,
    #[error("Protocol violation: {0}")]
    Protocol(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}
