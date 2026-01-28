//! Example: Subscribe to trade streams for multiple symbols.
//!
//! This example demonstrates how to:
//! - Connect to Binance WebSocket Streams
//! - Subscribe to trade streams for BTCUSDT and ETHUSDT
//! - Receive and print trade data
//!
//! Run with: cargo run --example subscribe_trades -p ctl-websocket

use atx_feed::FeedProtocolOps;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ctl_websocket::{WSConn, WSRequest, WSRequestId, WSRequestKind};

/// Binance WebSocket Streams base URL
const BINANCE_WS_STREAMS_URL: &str = "wss://stream.binance.com:9443/ws";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Binance WebSocket Streams...");

    // Create a new WebSocket connection
    let mut conn = WSConn::new(BINANCE_WS_STREAMS_URL)?;

    println!("Connected! Subscribing to trade streams...");

    // Create a subscribe request for trade streams
    let subscribe_request = WSRequest {
        kind: WSRequestKind::Subscribe(vec![
            "btcusdt@trade".to_string(),
            "ethusdt@trade".to_string(),
        ]),
        id: Some(WSRequestId::Int(1)),
    };

    // Serialize and send the subscribe request
    let request_json = serde_json::to_vec(&subscribe_request)?;

    println!("Sending subscribe request: {}", String::from_utf8_lossy(&request_json));
    println!("Sending Binary Data (base64): {}", STANDARD.encode(&request_json));
    conn.send(&request_json)?;

    println!("Subscribed to btcusdt@trade and ethusdt@trade");
    println!("Waiting for trade data...\n");

    // Poll for messages
    loop {
        match conn.poll()? {
            atx_feed::FeedPoll::Data(data) => {
                // Parse and print the received data
                if let Ok(json_str) = std::str::from_utf8(data) {
                    // Pretty print if it's valid JSON
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                        println!("{}", serde_json::to_string_pretty(&value)?);
                    } else {
                        println!("{}", json_str);
                    }
                }
            }
            atx_feed::FeedPoll::Empty => {
                // No data available, sleep briefly to avoid busy-waiting
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
