use arraystring::{typenum::U36, ArrayString};
use serde::{Deserialize, Serialize};

use super::WSRequestError;

/// Maximum length for string request IDs (36 characters at max).
/// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#live-subscribingunsubscribing-to-streams
pub type RequestIdString = ArrayString<U36>;

/// Request ID can be either an integer or a string.
/// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#live-subscribingunsubscribing-to-streams
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WSRequestId {
    Int(i64),
    String(RequestIdString),
}

impl From<i64> for WSRequestId {
    fn from(v: i64) -> Self {
        WSRequestId::Int(v)
    }
}

impl From<u64> for WSRequestId {
    fn from(v: u64) -> Self {
        WSRequestId::Int(v as i64)
    }
}

impl TryFrom<String> for WSRequestId {
    type Error = WSRequestError;

    fn try_from(v: String) -> Result<Self, Self::Error> {
        RequestIdString::try_from_str(&v)
            .map(WSRequestId::String)
            .map_err(|_| WSRequestError::RequestIdTooLong {
                id: RequestIdString::try_from_str(&v).unwrap_or_default(),
                len: v.len(),
                max: 36,
            })
    }
}

impl TryFrom<&str> for WSRequestId {
    type Error = WSRequestError;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        RequestIdString::try_from_str(v)
            .map(WSRequestId::String)
            .map_err(|_| WSRequestError::RequestIdTooLong {
                id: RequestIdString::try_from_str(&v).unwrap_or_default(),
                len: v.len(),
                max: 36,
            })
    }
}