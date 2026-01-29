use std::error::Error;

use atx_feed::FeedParseProtocol;
use ctl_feed::{DummyParser, Top};
use ctl_websocket::WSConn;
use dpdk::{ConsumeStartState, DpdkEnvBuilder, DpdkProcessType};

fn main() -> Result<(), Box<dyn Error>> {
    let dpdk_env = DpdkEnvBuilder::default()
        .process_type(DpdkProcessType::Secondary)
        .lcore_ids(vec![7])
        .main_lcore_id(7)
        .build()?;

    let ring = dpdk_env.pubsub::<<DummyParser as FeedParseProtocol<WSConn<Top>, Top>>::FeedParsedMessage>("TOP_PUBSUB", None)?;

    let mut consumer = ring.attach_consumer()?;

    loop {
        match consumer.consume_start(&ring) {
            ConsumeStartState::Success(mut guard) => {
                let msg = guard.as_ref();
                println!("Received message: {}", String::from_utf8_lossy(msg.get()));
                if guard.retry() {
                    continue;
                }
            },
            ConsumeStartState::InFlight(_) => {
                // Another consumer is in-flight - retry
                println!("In-flight, retrying...");
            },
            ConsumeStartState::SpedPast(_) => {
                // Consumer was overtaken - some messages were missed
                println!("Consumer was overtaken!");
            },
            ConsumeStartState::Empty => {}
        }
    }

    Ok(())
}
