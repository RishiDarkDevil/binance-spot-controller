use std::error::Error;

use atx_feed::FeedParseProtocol;
use ctl_feed::{DummyParser, Top};
use ctl_websocket::WSConn;
use dpdk::{DpdkEnvBuilder, DpdkProcessType};

fn main() -> Result<(), Box<dyn Error>> {

    // Initialize DPDK environment
    // TODO: Get the build options from a config file.
    let dpdk_env = DpdkEnvBuilder::default()
        .process_type(DpdkProcessType::Primary)
        .lcore_ids(vec![3])
        .build()?;

    let ring = dpdk_env.pubsub::<<DummyParser as FeedParseProtocol<WSConn<Top>, Top>>::FeedParsedMessage>("TOP_PUBSUB", 65536.into())?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}