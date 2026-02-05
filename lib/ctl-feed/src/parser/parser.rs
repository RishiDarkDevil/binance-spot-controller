use atx_feed::FeedParseProtocol;
use ctl_websocket::WSConn;
use dpdk::Aligned;

use crate::{AggTrade, Top, Trade, RawMessage};
use super::DummyParserError;

#[derive(Debug, Clone)]
pub struct DummyParser;

impl FeedParseProtocol<WSConn<Top>, Top> for DummyParser {

    type FeedParsedMessage = RawMessage;
    type FeedParseError = DummyParserError;

    fn parse(
            &mut self, 
            raw_data: atx_feed::FeedData,
            parsed_data: &mut Aligned<Self::FeedParsedMessage>
        ) -> Result<(), Self::FeedParseError> {

        std::str::from_utf8(raw_data)
            .map(|s| {
                let bytes = s.as_bytes();
                let buf = &mut parsed_data.get_mut().data;
                buf[..bytes.len()].copy_from_slice(bytes);
                buf[bytes.len()..].fill(0);
            })
            .map_err(|_| DummyParserError::General)?;
        // println!("parsed_data: {}", String::from_utf8_lossy(&parsed_data.get().data)); // TODO: REMOVE
        Ok(())
    }
}

impl FeedParseProtocol<WSConn<Trade>, Trade> for DummyParser {

    type FeedParsedMessage = RawMessage;
    type FeedParseError = DummyParserError;

    fn parse(
            &mut self, 
            raw_data: atx_feed::FeedData,
            parsed_data: &mut Aligned<Self::FeedParsedMessage>
        ) -> Result<(), Self::FeedParseError> {

        std::str::from_utf8(raw_data)
            .map(|s| {
                let bytes = s.as_bytes();
                let buf = &mut parsed_data.get_mut().data;
                buf[..bytes.len()].copy_from_slice(bytes);
                buf[bytes.len()..].fill(0);
            })
            .map_err(|_| DummyParserError::General)?;
        // println!("parsed_data: {}", String::from_utf8_lossy(&parsed_data.get().data)); // TODO: REMOVE
        Ok(())
    }
}

impl FeedParseProtocol<WSConn<AggTrade>, AggTrade> for DummyParser {

    type FeedParsedMessage = RawMessage;
    type FeedParseError = DummyParserError;

    fn parse(
            &mut self, 
            raw_data: atx_feed::FeedData,
            parsed_data: &mut Aligned<Self::FeedParsedMessage>
        ) -> Result<(), Self::FeedParseError> {

        std::str::from_utf8(raw_data)
            .map(|s| {
                let bytes = s.as_bytes();
                let buf = &mut parsed_data.get_mut().data;
                buf[..bytes.len()].copy_from_slice(bytes);
                buf[bytes.len()..].fill(0);
            })
            .map_err(|_| DummyParserError::General)?;
        // println!("parsed_data: {}", String::from_utf8_lossy(&parsed_data.get().data)); // TODO: REMOVE
        Ok(())
    }
}