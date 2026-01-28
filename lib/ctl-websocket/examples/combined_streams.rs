//! Example: Subscribe to multiple stream types using combined streams.
//!
//! This example demonstrates how to:
//! - Connect to Binance combined WebSocket Streams endpoint
//! - Subscribe to multiple stream types simultaneously
//! - Handle different message types
//!
//! Run with: cargo run --example combined_streams -p ctl-websocket

use atx_feed::FeedProtocolOps;
use ctl_websocket::{WSConn, WSRequest, WSRequestId, WSRequestKind};

/// Binance WebSocket Combined Streams URL
/// Note: Using /stream endpoint for combined streams
const BINANCE_WS_COMBINED_URL: &str = "wss://stream.binance.com:9443/stream";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Binance Combined WebSocket Streams...");

    // Create a new WebSocket connection to combined streams endpoint
    let mut conn = WSConn::new(BINANCE_WS_COMBINED_URL)?;

    println!("Connected! Subscribing to multiple stream types...");

    // Subscribe to various stream types for BTCUSDT:
    // - aggTrade: Aggregated trade data
    // - ticker: 24hr rolling window ticker
    // - bookTicker: Best bid/ask prices
    let subscribe_request = WSRequest {
        kind: WSRequestKind::Subscribe(vec![
            "btcusdt@aggTrade".to_string(),
            "btcusdt@ticker".to_string(),
            "btcusdt@bookTicker".to_string(),
        ]),
        id: Some(WSRequestId::Int(1)),
    };

    // Serialize and send the subscribe request
    let request_json = serde_json::to_vec(&subscribe_request)?;
    conn.send(&request_json)?;

    println!("Subscribed to btcusdt@aggTrade, btcusdt@ticker, btcusdt@bookTicker");
    println!("Waiting for market data...\n");

    // Poll for messages
    loop {
        match conn.poll()? {
            atx_feed::FeedPoll::Data(data) => {
                if let Ok(json_str) = std::str::from_utf8(data) {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                        // Combined streams wrap data in {"stream": "...", "data": {...}}
                        if let Some(stream) = value.get("stream").and_then(|s| s.as_str()) {
                            let data = value.get("data");
                            
                            if stream.ends_with("@aggTrade") {
                                // Aggregated trade
                                if let Some(d) = data {
                                    let price = d.get("p").and_then(|p| p.as_str()).unwrap_or("?");
                                    let qty = d.get("q").and_then(|q| q.as_str()).unwrap_or("?");
                                    let is_buyer_maker = d.get("m").and_then(|m| m.as_bool()).unwrap_or(false);
                                    let side = if is_buyer_maker { "SELL" } else { "BUY" };
                                    println!("[TRADE] {} {} @ {} ({})", side, qty, price, stream);
                                }
                            } else if stream.ends_with("@ticker") {
                                // 24hr ticker
                                if let Some(d) = data {
                                    let last_price = d.get("c").and_then(|c| c.as_str()).unwrap_or("?");
                                    let price_change_pct = d.get("P").and_then(|p| p.as_str()).unwrap_or("?");
                                    let volume = d.get("v").and_then(|v| v.as_str()).unwrap_or("?");
                                    println!(
                                        "[TICKER] Last: {} | Change: {}% | Vol: {}",
                                        last_price, price_change_pct, volume
                                    );
                                }
                            } else if stream.ends_with("@bookTicker") {
                                // Book ticker (best bid/ask)
                                if let Some(d) = data {
                                    let bid_price = d.get("b").and_then(|b| b.as_str()).unwrap_or("?");
                                    let bid_qty = d.get("B").and_then(|b| b.as_str()).unwrap_or("?");
                                    let ask_price = d.get("a").and_then(|a| a.as_str()).unwrap_or("?");
                                    let ask_qty = d.get("A").and_then(|a| a.as_str()).unwrap_or("?");
                                    println!(
                                        "[BOOK] Bid: {} ({}) | Ask: {} ({})",
                                        bid_price, bid_qty, ask_price, ask_qty
                                    );
                                }
                            }
                        } else if value.get("result").is_some() {
                            // Subscription response
                            let id = value.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
                            println!("Subscription confirmed (id: {})", id);
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
