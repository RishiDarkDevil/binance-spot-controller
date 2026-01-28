use atx_feed::{FeedData, FeedPoll, FeedProtocolOps};
use atx_websocket::{WebsocketConfig, WebsocketConn};

use crate::WebsocketConnectorError;

/// The exchange websocket connector.
/// This provides all the necessary methods to connect to the exchange websocket.
pub struct WSConn {
    /// The underlying websocket connection.
    websocket: WebsocketConn,
    /// Buffer for storing received message data.
    recv_buffer: Vec<u8>,
}

impl WSConn {
    /// Creates a new WSConn instance.
    pub fn new(url: &str) -> Result<Self, WebsocketConnectorError> {
        let mut websocket = WebsocketConn::new(url, WebsocketConfig::default())?;
        websocket.connect()?;
        Ok(Self {
            websocket,
            recv_buffer: Vec::with_capacity(4096),
        })
    }
}

impl FeedProtocolOps for WSConn {
    type FeedProtocolError = WebsocketConnectorError;

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
        let text = unsafe { std::str::from_utf8_unchecked(data) };
        self.websocket.send_text(text)?;
        Ok(())
    }
}