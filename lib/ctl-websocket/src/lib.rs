mod websocket;
mod requests;
mod error;

pub use websocket::WSConn;
pub use requests::{
    WSRequest, WSRequestKind, WSRequestId, WSRequestError, RequestIdString
};
pub use error::WebsocketConnectorError;