//! Example: List and manage WebSocket subscriptions.
//!
//! This example demonstrates how to:
//! - Subscribe to streams
//! - List current subscriptions
//! - Unsubscribe from specific streams
//! - Set/get stream properties
//!
//! Run with: cargo run --example list_and_manage_subscriptions -p ctl-websocket

use atx_feed::FeedProtocolOps;
use ctl_websocket::{WSConn, WSRequest, WSRequestId, WSRequestKind};
use serde_json::json;

/// Binance WebSocket Streams base URL
const BINANCE_WS_STREAMS_URL: &str = "wss://stream.binance.com:9443/ws";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Binance WebSocket Subscription Management Demo ===\n");

    // Create a new WebSocket connection
    let mut conn = WSConn::new(BINANCE_WS_STREAMS_URL)?;
    println!("Connected to Binance WebSocket Streams\n");

    // Helper to send request and wait for response
    let send_and_receive = |conn: &mut WSConn, request: &WSRequest| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let request_json = serde_json::to_vec(request)?;
        conn.send(&request_json)?;
        
        // Wait for response (with timeout)
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > std::time::Duration::from_secs(5) {
                return Err("Timeout waiting for response".into());
            }
            match conn.poll()? {
                atx_feed::FeedPoll::Data(data) => {
                    if let Ok(json_str) = std::str::from_utf8(data) {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                            return Ok(value);
                        }
                    }
                }
                atx_feed::FeedPoll::Empty => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    };

    // Step 1: Subscribe to multiple streams
    println!("Step 1: Subscribing to streams...");
    let subscribe_request = WSRequest {
        kind: WSRequestKind::Subscribe(vec![
            "btcusdt@trade".to_string(),
            "ethusdt@trade".to_string(),
            "bnbusdt@trade".to_string(),
        ]),
        id: Some(WSRequestId::Int(1)),
    };
    let response = send_and_receive(&mut conn, &subscribe_request)?;
    println!("Subscribe response: {}\n", serde_json::to_string_pretty(&response)?);

    // Step 2: List current subscriptions
    println!("Step 2: Listing current subscriptions...");
    let list_request = WSRequest {
        kind: WSRequestKind::ListSubscriptions,
        id: Some(WSRequestId::Int(2)),
    };
    let response = send_and_receive(&mut conn, &list_request)?;
    println!("Current subscriptions: {}\n", serde_json::to_string_pretty(&response)?);

    // Step 3: Unsubscribe from one stream
    println!("Step 3: Unsubscribing from bnbusdt@trade...");
    let unsubscribe_request = WSRequest {
        kind: WSRequestKind::Unsubscribe(vec!["bnbusdt@trade".to_string()]),
        id: Some(WSRequestId::Int(3)),
    };
    let response = send_and_receive(&mut conn, &unsubscribe_request)?;
    println!("Unsubscribe response: {}\n", serde_json::to_string_pretty(&response)?);

    // Step 4: List subscriptions again
    println!("Step 4: Listing subscriptions after unsubscribe...");
    let list_request = WSRequest {
        kind: WSRequestKind::ListSubscriptions,
        id: Some(WSRequestId::Int(4)),
    };
    let response = send_and_receive(&mut conn, &list_request)?;
    println!("Current subscriptions: {}\n", serde_json::to_string_pretty(&response)?);

    // Step 5: Get "combined" property
    println!("Step 5: Getting 'combined' property...");
    let get_property_request = WSRequest {
        kind: WSRequestKind::GetProperty(vec!["combined".to_string()]),
        id: Some(WSRequestId::Int(5)),
    };
    let response = send_and_receive(&mut conn, &get_property_request)?;
    println!("Combined property: {}\n", serde_json::to_string_pretty(&response)?);

    // Step 6: Set "combined" property to true
    println!("Step 6: Setting 'combined' property to true...");
    let set_property_request = WSRequest {
        kind: WSRequestKind::SetProperty(vec![json!("combined"), json!(true)]),
        id: Some(WSRequestId::Int(6)),
    };
    let response = send_and_receive(&mut conn, &set_property_request)?;
    println!("Set property response: {}\n", serde_json::to_string_pretty(&response)?);

    // Step 7: Get "combined" property again to verify
    println!("Step 7: Verifying 'combined' property...");
    let get_property_request = WSRequest {
        kind: WSRequestKind::GetProperty(vec!["combined".to_string()]),
        id: Some(WSRequestId::Int(7)),
    };
    let response = send_and_receive(&mut conn, &get_property_request)?;
    println!("Combined property: {}\n", serde_json::to_string_pretty(&response)?);

    println!("=== Demo Complete ===");
    Ok(())
}
