//! Example: Subscribe to order book depth streams.
//!
//! This example demonstrates how to:
//! - Connect to Binance WebSocket Streams
//! - Subscribe to partial book depth streams (top 10 levels)
//! - Receive and print order book updates
//!
//! Run with: cargo run --example subscribe_depth -p ctl-websocket

use atx_feed::FeedProtocolOps;
use ctl_websocket::{WSConn, WSRequest, WSRequestId, WSRequestKind};

/// Binance WebSocket Streams base URL
const BINANCE_WS_STREAMS_URL: &str = "wss://stream.binance.com:9443/ws";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Binance WebSocket Streams...");

    // Create a new WebSocket connection
    let mut conn = WSConn::new(BINANCE_WS_STREAMS_URL)?;

    println!("Connected! Subscribing to depth streams...");

    // Create a subscribe request for partial book depth streams
    // @depth10 gives top 10 bid/ask levels, updated every second
    // @depth10@100ms gives updates every 100ms
    let subscribe_request = WSRequest {
        kind: WSRequestKind::Subscribe(vec![
            "btcusdt@depth10@100ms".to_string(),
        ]),
        id: Some(WSRequestId::Int(1)),
    };

    // Serialize and send the subscribe request
    let request_json = serde_json::to_vec(&subscribe_request)?;
    conn.send(&request_json)?;

    println!("Subscribed to btcusdt@depth10@100ms");
    println!("Waiting for order book data...\n");

    let mut update_count = 0;

    // Poll for messages
    loop {
        match conn.poll()? {
            atx_feed::FeedPoll::Data(data) => {
                if let Ok(json_str) = std::str::from_utf8(data) {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                        // Check if this is a depth update (has "bids" and "asks")
                        if value.get("bids").is_some() && value.get("asks").is_some() {
                            update_count += 1;
                            println!("=== Order Book Update #{} ===", update_count);
                            
                            // Print top 3 bids
                            if let Some(bids) = value.get("bids").and_then(|b| b.as_array()) {
                                println!("Top Bids:");
                                for (i, bid) in bids.iter().take(3).enumerate() {
                                    if let Some(arr) = bid.as_array() {
                                        let price = arr.first().and_then(|p| p.as_str()).unwrap_or("?");
                                        let qty = arr.get(1).and_then(|q| q.as_str()).unwrap_or("?");
                                        println!("  {}. Price: {} | Qty: {}", i + 1, price, qty);
                                    }
                                }
                            }

                            // Print top 3 asks
                            if let Some(asks) = value.get("asks").and_then(|a| a.as_array()) {
                                println!("Top Asks:");
                                for (i, ask) in asks.iter().take(3).enumerate() {
                                    if let Some(arr) = ask.as_array() {
                                        let price = arr.first().and_then(|p| p.as_str()).unwrap_or("?");
                                        let qty = arr.get(1).and_then(|q| q.as_str()).unwrap_or("?");
                                        println!("  {}. Price: {} | Qty: {}", i + 1, price, qty);
                                    }
                                }
                            }
                            println!();
                        } else {
                            // Subscription response or other message
                            println!("Response: {}", serde_json::to_string_pretty(&value)?);
                        }
                    }
                }
            }
            atx_feed::FeedPoll::Empty => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
