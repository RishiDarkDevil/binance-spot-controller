use atx_feed::FeedGroup;
use ctl_websocket::WSConn;
use derive_more::From;

use crate::{AggTrade, DummyParser, Top, Trade};

#[derive(From)]
pub enum FeedGroups<'a> {
    JsonTop(FeedGroup<'a, WSConn<Top>, Top, DummyParser>),
    JsonTrade(FeedGroup<'a, WSConn<Trade>, Trade, DummyParser>),
    JsonAggTrade(FeedGroup<'a, WSConn<AggTrade>, AggTrade, DummyParser>),
}