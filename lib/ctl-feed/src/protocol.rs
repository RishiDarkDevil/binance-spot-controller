use atx_feed::{FeedProtocol, FeedProtocolOps, Streams};
use ctl_websocket::{WSConn, WSRequest, WSRequestKind};

use crate::{AggTrade, Top, Trade};

impl FeedProtocol<Top> for WSConn<Top> {
    /// Updates the subscribed streams for book ticker feed kind.
    /// 
    /// LATENCY: SLOW_PATH
    /// ERROR: FULLY_HANDLED
    fn update(&mut self, streams: &Streams<Top>) -> Result<(), Self::FeedProtocolError> {
        
        let unsubscribe = self.streams().difference(streams);
        let unsubscribe_streams = unsubscribe.into_iter()
            .map(|s| format!("{}@bookTicker", s.name)) // TODO: Add a better way to do this.
            .collect::<Vec<String>>();
        if !unsubscribe_streams.is_empty() {
            let req: WSRequest = (
                WSRequestKind::Unsubscribe(unsubscribe_streams), 
                None
            ).into();
            let request_json = serde_json::to_vec(&req)?;
            self.send(&request_json)?;
        }

        let subscribe = streams.difference(self.streams());
        let subscribe_streams = subscribe.into_iter()
            .map(|s| format!("{}@bookTicker", s.name))
            .collect::<Vec<String>>();
        if !subscribe_streams.is_empty() {
            let req: WSRequest = (
                WSRequestKind::Subscribe(subscribe_streams), 
                None
            ).into();
            let request_json = serde_json::to_vec(&req)?;
            self.send(&request_json)?;
        }

        Ok(())
    }
}

impl FeedProtocol<Trade> for WSConn<Trade> {
    /// Updates the subscribed streams for trade feed kind.
    /// 
    /// LATENCY: SLOW_PATH
    /// ERROR: FULLY_HANDLED
    fn update(&mut self, streams: &Streams<Trade>) -> Result<(), Self::FeedProtocolError> {
        
        let unsubscribe = self.streams().difference(streams);
        let unsubscribe_streams = unsubscribe.into_iter()
            .map(|s| format!("{}@trade", s.name)) // TODO: Add a better way to do this.
            .collect::<Vec<String>>();
        if !unsubscribe_streams.is_empty() {
            let req: WSRequest = (
                WSRequestKind::Unsubscribe(unsubscribe_streams), 
                None
            ).into();
            let request_json = serde_json::to_vec(&req)?;
            self.send(&request_json)?;
        }

        let subscribe = streams.difference(self.streams());
        let subscribe_streams = subscribe.into_iter()
            .map(|s| format!("{}@trade", s.name)) // TODO: Add a better way to do this.
            .collect::<Vec<String>>();
        if !subscribe_streams.is_empty() {
            let req: WSRequest = (
                WSRequestKind::Subscribe(subscribe_streams), 
                None
            ).into();
            let request_json = serde_json::to_vec(&req)?;
            self.send(&request_json)?;
        }

        Ok(())
    }
}

impl FeedProtocol<AggTrade> for WSConn<AggTrade> {
    /// Updates the subscribed streams for aggregated trade feed kind.
    /// 
    /// LATENCY: SLOW_PATH
    /// ERROR: FULLY_HANDLED
    fn update(&mut self, streams: &Streams<AggTrade>) -> Result<(), Self::FeedProtocolError> {
        
        let unsubscribe = self.streams().difference(streams);
        let unsubscribe_streams = unsubscribe.into_iter()
            .map(|s| format!("{}@aggTrade", s.name))
            .collect::<Vec<String>>();
        if !unsubscribe_streams.is_empty() {
            let req: WSRequest = (
                WSRequestKind::Unsubscribe(unsubscribe_streams), 
                None
            ).into();
            let request_json = serde_json::to_vec(&req)?;
            self.send(&request_json)?;
        }

        let subscribe = streams.difference(self.streams());
        let subscribe_streams = subscribe.into_iter()
            .map(|s| format!("{}@aggTrade", s.name))
            .collect::<Vec<String>>();
        if !subscribe_streams.is_empty() {
            let req: WSRequest = (
                WSRequestKind::Subscribe(subscribe_streams), 
                None
            ).into();
            let request_json = serde_json::to_vec(&req)?;
            self.send(&request_json)?;
        }

        Ok(())
    }
}