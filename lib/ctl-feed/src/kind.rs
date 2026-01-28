//! The feed kinds supported for Binance Spot.

use atx_feed::FeedKind;

/// Book Top feed kind.
/// This feed provides real-time best bid and ask prices and quantities in the order book.
/// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#individual-symbol-book-ticker-streams
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Top;

/// Raw Trade feed kind.
/// This feed provides real-time trade data including price, quantity, and trade time with unique buyer and seller.
/// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#trade-streams
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Trade;

/// Aggregated Trade feed kind.
/// This feed provides real-time trade data including price, quantity, and trade time that is aggregated for a single taker.
/// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#aggregate-trade-streams
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AggTrade;

impl FeedKind for Top {}
impl FeedKind for Trade {}
impl FeedKind for AggTrade {}