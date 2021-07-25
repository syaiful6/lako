use bcrypt::BcryptError;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum AppError {
    #[error("IO Error: `{0}`")]
    IO(#[from] io::Error),

    #[error("Bcrypt error: `{0}`")]
    BcryptError(#[from] BcryptError),

    #[error("Database error: `{0}`")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("JSON Error: `{0}`")]
    JSONDecode(#[from] serde_json::Error),
}

pub type AppResult<T> = Result<T, AppError>;
