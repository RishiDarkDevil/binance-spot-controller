use atx_feed::{FeedProtocol, FeedProtocolOps, Streams};
use ctl_websocket::{WSConn, WSRequest, WSRequestKind};

use crate::{AggTrade, Top, Trade};

impl FeedProtocol<Top> for WSConn {
    fn update(&mut self, streams: &Streams<Top>) -> Result<(), Self::FeedProtocolError> {
        let stream_names: Vec<String> = streams
            .iter()
            .map(|s| format!("{}@bookTicker", s.name))
            .collect();
        let req: WSRequest = (
            WSRequestKind::Subscribe(stream_names), 
            None
        ).into();
        let request_json = serde_json::to_vec(&req)?;
        self.send(&request_json)
    }
}

impl FeedProtocol<Trade> for WSConn {
    fn update(&mut self, streams: &Streams<Trade>) -> Result<(), Self::FeedProtocolError> {
        let stream_names: Vec<String> = streams
            .iter()
            .map(|s| format!("{}@trade", s.name))
            .collect();
        let req: WSRequest = (
            WSRequestKind::Subscribe(stream_names), 
            None
        ).into();
        let request_json = serde_json::to_vec(&req)?;
        self.send(&request_json)
    }
}

impl FeedProtocol<AggTrade> for WSConn {
    fn update(&mut self, streams: &Streams<AggTrade>) -> Result<(), Self::FeedProtocolError> {
        let stream_names: Vec<String> = streams
            .iter()
            .map(|s| format!("{}@aggTrade", s.name))
            .collect();
        let req: WSRequest = (
            WSRequestKind::Subscribe(stream_names), 
            None
        ).into();
        let request_json = serde_json::to_vec(&req)?;
        self.send(&request_json)
    }
}