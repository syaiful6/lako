use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use hyper::{StatusCode};
use gotham::handler::{HandlerError, IntoHandlerError};

pub struct AppError {
    status_code: StatusCode,
    cause: Box<dyn Error + Send>,
}

impl IntoHandlerError for AppError {
    fn into_handler_error(self) -> HandlerError {
        HandlerError {
            status_code: self.status_code,
            cause: self.cause,
        }
    }
}

