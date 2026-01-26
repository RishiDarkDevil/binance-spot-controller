use std::sync::atomic::{AtomicU64, Ordering};

use atx_feed::{FeedData, FeedPoll, FeedProtocol, FeedProtocolOps, Streams};
use atx_websocket::{WebsocketConfig, WebsocketConn};
use ctl_core::{Top, Trade};

use crate::BinanceWebsocketConnError;

/// Atomic counter for generating unique request IDs.
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// The exchange websocket connector.
/// This provides all the necessary methods to connect to the exchange websocket.
pub struct BSWebsocketConn {
    /// The underlying websocket connection.
    websocket: WebsocketConn,
    /// Buffer for storing received message data.
    recv_buffer: Vec<u8>,
}

impl BSWebsocketConn {
    /// Creates a new BSWebsocketConn instance.
    pub fn new(url: &str) -> Result<Self, BinanceWebsocketConnError> {
        let mut websocket = WebsocketConn::new(url, WebsocketConfig::default())?;
        websocket.connect()?;
        Ok(Self {
            websocket,
            recv_buffer: Vec::with_capacity(4096),
        })
    }

    /// Generates the next unique request ID.
    fn next_request_id() -> u64 {
        REQUEST_ID.fetch_add(1, Ordering::Relaxed)
    }

    /// Subscribes to a list of streams.
    fn subscribe(&mut self, streams: &[&str]) -> Result<(), BinanceWebsocketConnError> {
        if streams.is_empty() {
            return Ok(());
        }

        let request = serde_json::json!({
            "method": "SUBSCRIBE",
            "params": streams,
            "id": Self::next_request_id()
        });

        self.websocket.send_text(&request.to_string())?;
        Ok(())
    }

    /// Unsubscribes from a list of streams.
    #[allow(dead_code)]
    fn unsubscribe(&mut self, streams: &[&str]) -> Result<(), BinanceWebsocketConnError> {
        if streams.is_empty() {
            return Ok(());
        }

        let request = serde_json::json!({
            "method": "UNSUBSCRIBE",
            "params": streams,
            "id": Self::next_request_id()
        });

        self.websocket.send_text(&request.to_string())?;
        Ok(())
    }
}

impl FeedProtocolOps for BSWebsocketConn {
    type FeedProtocolError = BinanceWebsocketConnError;

    fn poll(&mut self) -> Result<FeedPoll<'_>, Self::FeedProtocolError> {
        match self.websocket.poll()? {
            Some(msg) => {
                self.recv_buffer.clear();
                self.recv_buffer.extend_from_slice(msg.as_bytes());
                Ok(FeedPoll::Data(&self.recv_buffer))
            }
            None => Ok(FeedPoll::Empty),
        }
    }

    fn send(&mut self, data: FeedData) -> Result<(), Self::FeedProtocolError> {
        self.websocket.send_binary(data)?;
        Ok(())
    }
}

impl FeedProtocol<Top> for BSWebsocketConn {
    fn update(&mut self, streams: &Streams<Top>) -> Result<(), Self::FeedProtocolError> {
        // For BookTicker (Top), the stream name format is: <symbol>@bookTicker
        // The stream name in Streams<Top> is expected to be the symbol (lowercase)
        // e.g., "btcusdt" -> "btcusdt@bookTicker"
        let stream_names: Vec<String> = streams
            .iter()
            .map(|s| format!("{}@bookTicker", s.name))
            .collect();

        let stream_refs: Vec<&str> = stream_names.iter().map(|s| s.as_str()).collect();

        // For simplicity, we subscribe to all streams.
        // A more sophisticated implementation would track current subscriptions
        // and only subscribe/unsubscribe the diff.
        self.subscribe(&stream_refs)
    }
}

impl FeedProtocol<Trade> for BSWebsocketConn {
    fn update(&mut self, streams: &Streams<Trade>) -> Result<(), Self::FeedProtocolError> {
        // For Trade, the stream name format is: <symbol>@trade
        // The stream name in Streams<Trade> is expected to be the symbol (lowercase)
        // e.g., "btcusdt" -> "btcusdt@trade"
        let stream_names: Vec<String> = streams
            .iter()
            .map(|s| format!("{}@trade", s.name))
            .collect();

        let stream_refs: Vec<&str> = stream_names.iter().map(|s| s.as_str()).collect();

        // For simplicity, we subscribe to all streams.
        // A more sophisticated implementation would track current subscriptions
        // and only subscribe/unsubscribe the diff.
        self.subscribe(&stream_refs)
    }
}
