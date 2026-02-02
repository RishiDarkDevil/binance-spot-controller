//! Market Data Handler library.
//!
//! This crate provides the configuration and handler logic for processing
//! market data from Binance Spot.

mod config;
mod errors;

pub use errors::HwResourcesConfigError;

pub use config::{
    FeedConfig, FeedWrapper, HwResourcesConfig, PubSubConfig, SymbolSet,
};

