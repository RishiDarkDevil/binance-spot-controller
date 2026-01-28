use thiserror::Error;

use super::RequestIdString;

#[derive(Debug, Error)]
pub enum WSRequestError {
    #[error("ws request error: request ID {id} length {len} exceeds maximum of {max}")]
    RequestIdTooLong { id: RequestIdString, len: usize, max: usize },
}