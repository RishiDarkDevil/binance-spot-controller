use std::error::Error;

use atx_feed::{Feed, FeedGroup, FeedGroupConfig, FeedParseProtocol, FeedProtocol, Stream, Streams};
use atx_handler::{HandlerBuilder, HandlerRunner};
use ctl_feed::{DummyParser, Top};
use ctl_websocket::WSConn;
use dpdk::{DpdkEnvBuilder, DpdkProcessType};

fn main() -> Result<(), Box<dyn Error>> {
    let dpdk_env = DpdkEnvBuilder::default()
        .process_type(DpdkProcessType::Secondary)
        .lcore_ids(vec![4, 5, 6])
        .main_lcore_id(4)
        .build()?;

    let dummy_parser = DummyParser;

    let ring = dpdk_env
        .pubsub::<<DummyParser as FeedParseProtocol<WSConn<Top>, Top>>::FeedParsedMessage>
        ("TOP_PUBSUB", None)?;

    let symbols = vec!["btcusdt", "ethusdt", "bnbusdt", "xrpusdt"];
    let mut streams = Streams::new();
    symbols.iter().for_each(|&symbol| {
        streams.insert(Stream::new(symbol));
    });

    let mut websocket_conn = WSConn::<Top>::new("wss://stream.binance.com:9443/ws")?;
    websocket_conn.update(&streams)?;
    let feeds = vec![Feed::new("TopFeed", websocket_conn)];

    let topfeedgroup_config = FeedGroupConfig {
        dpdk_env: &dpdk_env,
        num_workers: 1,
        publisher: ring,
        parser: dummy_parser,
        feeds
    };

    let mut topfeedgroup = FeedGroup::validated_build(topfeedgroup_config)?;

    topfeedgroup.run()?;

    Ok(())
}