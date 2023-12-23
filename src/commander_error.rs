use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommanderError {
    #[error("Failed to read history location from environment")]
    EnvError,
    #[error("File Read error")]
    FileReadError {source: std::io::Error},
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
