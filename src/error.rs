use sqlx::migrate::MigrateError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io;

#[derive(Debug)]
pub struct InternalError(String);

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error: {}", self.0))
    }
}

impl Error for InternalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl InternalError {
    pub fn message(msg: String) -> Self {
        InternalError(msg)
    }
}

impl From<sqlx::Error> for InternalError {
    fn from(e: sqlx::Error) -> Self {
        InternalError(format!("{}", e))
    }
}

impl From<MigrateError> for InternalError {
    fn from(e: MigrateError) -> Self {
        InternalError(format!("{}", e))
    }
}

impl From<io::Error> for InternalError {
    fn from(e: io::Error) -> Self {
        InternalError(format!("{}", e))
    }
}
