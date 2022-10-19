use std::error::Error;

use anyhow::Result;

pub fn error<T>(err: impl Error + Send + Sync + 'static) -> Result<T> {
    Err(err.into())
}
