//! Dummy Market Data Subscriber for testing shared ring consumption.
//!
//! This binary connects as a DPDK secondary process and reads RawMessage
//! data from the shared rings created by ctl-resource-manager and published
//! to by ctl-md-handler.

use std::error::Error;

use ctl_feed::RawMessage;
use dpdk::{ConsumeStartState, DpdkEnvBuilder, DpdkProcessType};

// Ring naming convention: {KIND}_{symbol_id}_PS
// Using BTCUSDT (symbol_id=0) as default for testing
const RING_NAME: &str = "TOP_0_PS";

// Use a separate lcore that doesn't conflict with md-handler workers
const SUBSCRIBER_LCORE: usize = 13;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Binance Spot Market Data Subscriber ===");
    println!("Starting as DPDK secondary process...\n");

    let dpdk_env = DpdkEnvBuilder::default()
        .process_type(DpdkProcessType::Secondary)
        .lcore_ids(vec![SUBSCRIBER_LCORE])
        .main_lcore_id(SUBSCRIBER_LCORE)
        .build()?;

    println!("DPDK environment initialized");
    println!("Looking up ring: {}", RING_NAME);

    // Look up the ring by name and type - must match what was registered by resource-manager
    let ring = dpdk_env.pubsub_lookup::<RawMessage>(RING_NAME)?;

    println!("Ring found, attaching consumer...");
    let mut consumer = ring.attach_consumer()?;

    println!("Consumer attached, starting to read messages...\n");

    let mut msg_count: u64 = 0;
    let mut empty_polls: u64 = 0;

    loop {
        match consumer.consume_start() {
            ConsumeStartState::Success(mut guard) => {
                // Try to commit first (mark message as consumed)
                match guard.try_commit() {
                    Ok(_) => {
                        let msg = guard.as_ref();
                        let data = &msg.get().data;
                        
                        // Find the actual message length (up to first null byte or end)
                        let len = data.iter().position(|&b| b == 0).unwrap_or(data.len());
                        let msg_str = String::from_utf8_lossy(&data[..len]);
                        
                        msg_count += 1;
                        println!("[{}] Received: {}", msg_count, msg_str);
                    }
                    Err(_) => {
                        // Commit failed, retry
                        continue;
                    }
                }
            }
            ConsumeStartState::InFlight(_guard) => {
                // Another consumer is in-flight - retry
                // This is rare in single-consumer scenarios
            }
            ConsumeStartState::SpedPast(_guard) => {
                // Consumer was overtaken by the producer - some messages were missed
                // The guard still contains valid data we can read
                println!("[Warning] Consumer overtaken by producer, some messages missed");
            }
            ConsumeStartState::Empty => {
                empty_polls += 1;
                // Periodically report we're still alive
                if empty_polls % 1_000_000 == 0 {
                    println!("[Status] Waiting for messages... (total received: {})", msg_count);
                }
            }
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}
