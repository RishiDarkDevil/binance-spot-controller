//! Example: Subscribe to kline/candlestick streams.
//!
//! This example demonstrates how to:
//! - Connect to Binance WebSocket Streams
//! - Subscribe to 1-minute kline streams
//! - Parse and display candlestick data
//!
//! Run with: cargo run --example subscribe_klines -p ctl-websocket

use atx_feed::FeedProtocolOps;
use ctl_websocket::{WSConn, WSRequest, WSRequestId, WSRequestKind};

/// Binance WebSocket Streams base URL
const BINANCE_WS_STREAMS_URL: &str = "wss://stream.binance.com:9443/ws";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Binance WebSocket Streams...");

    // Create a new WebSocket connection
    let mut conn = WSConn::new(BINANCE_WS_STREAMS_URL)?;

    println!("Connected! Subscribing to kline streams...");

    // Create a subscribe request for 1-minute kline streams
    // Available intervals: 1s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
    let subscribe_request = WSRequest {
        kind: WSRequestKind::Subscribe(vec![
            "btcusdt@kline_1m".to_string(),
            "ethusdt@kline_1m".to_string(),
        ]),
        id: Some(WSRequestId::Int(1)),
    };

    // Serialize and send the subscribe request
    let request_json = serde_json::to_vec(&subscribe_request)?;
    conn.send(&request_json)?;

    println!("Subscribed to btcusdt@kline_1m and ethusdt@kline_1m");
    println!("Waiting for kline data...\n");

    // Poll for messages
    loop {
        match conn.poll()? {
            atx_feed::FeedPoll::Data(data) => {
                if let Ok(json_str) = std::str::from_utf8(data) {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                        // Check if this is a kline event
                        if value.get("e").and_then(|e| e.as_str()) == Some("kline") {
                            let symbol = value.get("s").and_then(|s| s.as_str()).unwrap_or("?");
                            
                            if let Some(k) = value.get("k") {
                                let open = k.get("o").and_then(|o| o.as_str()).unwrap_or("?");
                                let high = k.get("h").and_then(|h| h.as_str()).unwrap_or("?");
                                let low = k.get("l").and_then(|l| l.as_str()).unwrap_or("?");
                                let close = k.get("c").and_then(|c| c.as_str()).unwrap_or("?");
                                let volume = k.get("v").and_then(|v| v.as_str()).unwrap_or("?");
                                let is_closed = k.get("x").and_then(|x| x.as_bool()).unwrap_or(false);
                                let trades = k.get("n").and_then(|n| n.as_i64()).unwrap_or(0);

                                println!(
                                    "[{}] O: {} | H: {} | L: {} | C: {} | Vol: {} | Trades: {} | Closed: {}",
                                    symbol, open, high, low, close, volume, trades, is_closed
                                );
                            }
                        } else if value.get("result").is_some() {
                            // Subscription response
                            println!("Subscription confirmed: {}", serde_json::to_string(&value)?);
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
