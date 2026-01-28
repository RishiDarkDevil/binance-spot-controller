use atx_websocket::WebsocketConnError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebsocketConnectorError {
    #[error("Websocket error: {0}")]
    WebsocketConnError(#[from] WebsocketConnError)
}