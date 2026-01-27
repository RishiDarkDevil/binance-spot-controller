use std::io::Error;

use atx_feed::{FeedCollection, FeedGroup, FeedParseProtocol};
use ctl_core::{Top, Trade};
use ctl_websocket::BSWebsocketConn;
use dpdk::Aligned;

pub enum BSFeedGroup {
    JsonTop(FeedGroup<BSWebsocketConn, Top, TopParser>),
    JsonTrade(FeedGroup<BSWebsocketConn, Trade, TradeParser>),
}

impl AsMut<FeedGroup<BSWebsocketConn, Top, TopParser>> for BSFeedGroup {
    fn as_mut(&mut self) -> &mut FeedGroup<BSWebsocketConn, Top, TopParser> {
        match self {
            BSFeedGroup::JsonTop(group) => group,
            _ => panic!("Not a Top feed group"),
        }
    }
}

impl AsMut<FeedGroup<BSWebsocketConn, Trade, TradeParser>> for BSFeedGroup {
    fn as_mut(&mut self) -> &mut FeedGroup<BSWebsocketConn, Trade, TradeParser> {
        match self {
            BSFeedGroup::JsonTrade(group) => group,
            _ => panic!("Not a Trade feed group"),
        }
    }
}

pub struct feedcollection(FeedCollection<BSFeedGroup>);

impl feedcollection {
    pub fn hehe(mut self) {
        self.0.feed_group_mut::<BSWebsocketConn, Top, TopParser>("name");
    }
}

pub struct TopParser;

impl FeedParseProtocol for TopParser {
    type FeedKind = Top;
    type FeedProtocol = BSWebsocketConn;
    type FeedParsedMessage = usize;
    type FeedParseError = Error;

    fn parse(
            &mut self, 
            raw_data: atx_feed::FeedData,
            parsed_data: &mut Aligned<Self::FeedParsedMessage>
        ) -> Result<(), Self::FeedParseError> {
        Ok(())
    }
}

pub struct TradeParser;

impl FeedParseProtocol for TradeParser {
    type FeedKind = Trade;
    type FeedProtocol = BSWebsocketConn;
    type FeedParsedMessage = usize;
    type FeedParseError = Error;

    fn parse(
            &mut self, 
            raw_data: atx_feed::FeedData,
            parsed_data: &mut Aligned<Self::FeedParsedMessage>
        ) -> Result<(), Self::FeedParseError> {
        Ok(())
    }
}