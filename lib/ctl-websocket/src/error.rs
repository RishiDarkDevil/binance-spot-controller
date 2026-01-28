use atx_websocket::WebsocketConnError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebsocketConnectorError {
    #[error("websocket connector error: websocket error {0}")]
    WebsocketConnError(#[from] WebsocketConnError),
    #[error("websocket connector error: serde json error {0}")]
    SerdeError(#[from] serde_json::Error),
}