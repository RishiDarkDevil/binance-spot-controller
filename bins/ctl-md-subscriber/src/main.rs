use std::error::Error;

use ctl_feed::RawMessage;
use dpdk::{ConsumeStartState, DpdkEnvBuilder, DpdkProcessType};

fn main() -> Result<(), Box<dyn Error>> {
    let dpdk_env = DpdkEnvBuilder::default()
        .process_type(DpdkProcessType::Secondary)
        .lcore_ids(vec![7])
        .main_lcore_id(7)
        .build()?;

    // Look up the ring by name and type - must match what was registered
    let ring = dpdk_env.pubsub_lookup::<RawMessage>("TOP_PUBSUB")?;

    let mut consumer = ring.attach_consumer()?;

    loop {
        match consumer.consume_start() {
            ConsumeStartState::Success(mut guard) => {
                match guard.try_commit() {
                    Ok(_) => {},
                    Err(_) => continue,
                }
                let msg = guard.as_ref();
                println!("Received message: {}", String::from_utf8_lossy(&msg.get().data));
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

    #[allow(unreachable_code)]
    Ok(())
}
