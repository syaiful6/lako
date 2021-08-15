use bcrypt::BcryptError;
use lettre::smtp::error::Error as SmtpError;
use lettre_email::error::Error as LettreEmailError;
use native_tls::Error as NativeTlsError;
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

    #[error("SMTP Error: `{0}`")]
    LettreSMTPError(#[from] SmtpError),

    #[error("LettreEmail Error: `{0}`")]
    LettreEmailError(#[from] LettreEmailError),

    #[error("NativeTlsError error: `{0}`")]
    NativeTlsError(#[from] NativeTlsError),

    #[error("Invalid config error")]
    InvalidConfig,
}

pub type AppResult<T> = Result<T, AppError>;
