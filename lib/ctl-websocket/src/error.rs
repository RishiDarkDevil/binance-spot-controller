use atx_websocket::WebsocketError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinanceWebsocketConnError {
    #[error("Websocket error: {0}")]
    WebsocketError(#[from] WebsocketError)
}