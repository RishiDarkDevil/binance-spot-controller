//! Market Data Handler for Binance Spot.
//!
//! This binary connects to Binance WebSocket streams and publishes raw market data
//! to DPDK shared memory rings. It runs as a DPDK secondary process, looking up
//! rings created by ctl-resource-manager.
//!
//! # Architecture
//!
//! - Creates FeedGroups for each feed kind (Top, Trade, AggTrade)
//! - Each FeedGroup manages one or more WebSocket connections (Feeds)
//! - Workers poll feeds, parse messages, and publish to shared rings
//! - Main thread coordinates feedgroups, polls feedback, and handles commands

use std::error::Error;

use atx_feed::{
    Feed, FeedGroup, FeedGroupConfig, FeedGroupWorkerCommandAck, FeedGroupWorkerFeedback,
    FeedKind, FeedProtocol, Stream, Streams,
};
use atx_handler::{HandlerBuilder, HandlerRunner};
use ctl_feed::{AggTrade, DummyParser, RawMessage, Top, Trade};
use ctl_md_handler::{HwResourcesConfig, SymbolInfoConfig};
use ctl_websocket::WSConn;
use dpdk::{DpdkEnv, DpdkEnvBuilder, DpdkLCoreId, DpdkPubSubRing, DpdkProcessType, MultiJoinHandle};

// Configuration file paths
const MD_CONFIG_PATH: &str = "configs/market-data/hw-resources.yaml";
const SYMBOL_INFO_PATH: &str = "configs/market-data/symbolinfo.yaml";

// WebSocket endpoint for Binance Spot
const BINANCE_WS_ENDPOINT: &str = "wss://stream.binance.com:9443/ws";

// Channel capacities for command/feedback queues
const COMMAND_CHANNEL_CAPACITY: usize = 1024;
const FEEDBACK_CHANNEL_CAPACITY: usize = 1024;

/// Creates a FeedGroup for the Top (book ticker) feed kind.
///
/// Looks up rings for each symbol and creates WebSocket feeds to subscribe to bookTicker streams.
fn create_top_feedgroup<'a>(
    dpdk_env: &'a DpdkEnv,
    md_config: &HwResourcesConfig,
    symbol_info: &SymbolInfoConfig,
    worker_lcore_ids: Vec<DpdkLCoreId>,
) -> Result<FeedGroup<'a, WSConn<Top>, Top, DummyParser>, Box<dyn Error>> {
    let feed_config = md_config
        .find_feed("top")
        .ok_or("Feed kind 'top' not found in config")?;

    let symbols: Vec<&str> = feed_config.all_symbols();
    if symbols.is_empty() {
        return Err("No symbols configured for 'top' feed".into());
    }

    // Create streams for all symbols
    let mut streams: Streams<Top> = Streams::new();
    for symbol in &symbols {
        streams.insert(Stream::new(symbol.to_lowercase().leak()));
    }

    // Create WebSocket connection and subscribe to streams
    let mut ws_conn = WSConn::<Top>::new(BINANCE_WS_ENDPOINT)?;
    FeedProtocol::update(&mut ws_conn, &streams)?;

    // Create feeds (one feed per connection for now)
    let feeds = vec![Feed::new("TopFeed", ws_conn)];

    // Lookup the ring for the first symbol (for now, using single ring per kind)
    // Ring naming convention: {KIND}_{symbol_id}_PS
    let first_symbol = symbols.first().ok_or("No symbols for top feed")?;
    let symbol_id = symbol_info
        .symbol_id(first_symbol)
        .ok_or_else(|| format!("Symbol '{}' not found in symbolinfo.yaml", first_symbol))?;
    let ring_name = format!("TOP_{}_PS", symbol_id);
    let ring: DpdkPubSubRing<RawMessage> = dpdk_env.pubsub_lookup::<RawMessage>(&ring_name)?;

    println!(
        "[TopFeedGroup] Created with {} symbols, {} workers, ring: {}",
        symbols.len(),
        worker_lcore_ids.len(),
        ring_name
    );

    let config = FeedGroupConfig {
        name: "TopFeedGroup",
        dpdk_env,
        worker_lcore_ids,
        publisher: ring,
        parser: DummyParser,
        feeds,
        command_channel_capacity: COMMAND_CHANNEL_CAPACITY,
        feedback_channel_capacity: FEEDBACK_CHANNEL_CAPACITY,
    };

    Ok(FeedGroup::validated_build(config)?)
}

/// Creates a FeedGroup for the Trade feed kind.
///
/// Looks up rings for each symbol and creates WebSocket feeds to subscribe to trade streams.
fn create_trade_feedgroup<'a>(
    dpdk_env: &'a DpdkEnv,
    md_config: &HwResourcesConfig,
    symbol_info: &SymbolInfoConfig,
    worker_lcore_ids: Vec<DpdkLCoreId>,
) -> Result<FeedGroup<'a, WSConn<Trade>, Trade, DummyParser>, Box<dyn Error>> {
    let feed_config = md_config
        .find_feed("trade")
        .ok_or("Feed kind 'trade' not found in config")?;

    let symbols: Vec<&str> = feed_config.all_symbols();
    if symbols.is_empty() {
        return Err("No symbols configured for 'trade' feed".into());
    }

    // Create streams for all symbols
    let mut streams: Streams<Trade> = Streams::new();
    for symbol in &symbols {
        streams.insert(Stream::new(symbol.to_lowercase().leak()));
    }

    // Create WebSocket connection and subscribe to streams
    let mut ws_conn = WSConn::<Trade>::new(BINANCE_WS_ENDPOINT)?;
    FeedProtocol::update(&mut ws_conn, &streams)?;

    // Create feeds
    let feeds = vec![Feed::new("TradeFeed", ws_conn)];

    // Lookup the ring for the first symbol
    let first_symbol = symbols.first().ok_or("No symbols for trade feed")?;
    let symbol_id = symbol_info
        .symbol_id(first_symbol)
        .ok_or_else(|| format!("Symbol '{}' not found in symbolinfo.yaml", first_symbol))?;
    let ring_name = format!("TRADE_{}_PS", symbol_id);
    let ring: DpdkPubSubRing<RawMessage> = dpdk_env.pubsub_lookup::<RawMessage>(&ring_name)?;

    println!(
        "[TradeFeedGroup] Created with {} symbols, {} workers, ring: {}",
        symbols.len(),
        worker_lcore_ids.len(),
        ring_name
    );

    let config = FeedGroupConfig {
        name: "TradeFeedGroup",
        dpdk_env,
        worker_lcore_ids,
        publisher: ring,
        parser: DummyParser,
        feeds,
        command_channel_capacity: COMMAND_CHANNEL_CAPACITY,
        feedback_channel_capacity: FEEDBACK_CHANNEL_CAPACITY,
    };

    Ok(FeedGroup::validated_build(config)?)
}

/// Handles feedback from a FeedGroup worker.
///
/// Logs acknowledgements and errors for debugging/monitoring.
fn handle_feedback<P, K>(group_name: &str, feedback: FeedGroupWorkerFeedback<P, K>)
where
    P: atx_feed::FeedProtocol<K>,
    K: FeedKind,
{
    match feedback {
        FeedGroupWorkerFeedback::FeedGroupWorkerCommandAck(ack) => match ack {
            FeedGroupWorkerCommandAck::AddFeed(removed) => {
                if let Some(_feed) = removed {
                    println!("[{}] AddFeed: replaced existing feed", group_name);
                } else {
                    println!("[{}] AddFeed: new feed added", group_name);
                }
            }
            FeedGroupWorkerCommandAck::RemoveFeed(removed) => {
                if removed.is_some() {
                    println!("[{}] RemoveFeed: feed removed", group_name);
                } else {
                    println!("[{}] RemoveFeed: feed not found", group_name);
                }
            }
            FeedGroupWorkerCommandAck::AddStream(is_new) => {
                println!(
                    "[{}] AddStream: {}",
                    group_name,
                    if is_new { "newly added" } else { "already existed" }
                );
            }
            FeedGroupWorkerCommandAck::RemoveStream(removed) => {
                if removed.is_some() {
                    println!("[{}] RemoveStream: stream removed", group_name);
                } else {
                    println!("[{}] RemoveStream: stream not found", group_name);
                }
            }
        },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Binance Spot Market Data Handler ===");
    println!("Starting as DPDK secondary process...\n");

    // Load configurations
    let md_config = HwResourcesConfig::from_file(MD_CONFIG_PATH)?;
    let symbol_info = SymbolInfoConfig::from_file(SYMBOL_INFO_PATH)?;

    println!("Loaded market data config from: {}", MD_CONFIG_PATH);
    println!("Loaded symbol info from: {}", SYMBOL_INFO_PATH);
    println!("Main CPU: {}", md_config.main_cpu);
    println!("Worker CPUs: {:?}", md_config.worker_cpus);
    println!();

    // Collect all lcore IDs needed
    let main_lcore_id = md_config.main_cpu as DpdkLCoreId;
    let worker_cpus: Vec<DpdkLCoreId> = md_config
        .worker_cpus
        .clone()
        .map(|cpu| cpu as DpdkLCoreId)
        .collect();

    // All lcores = main + workers
    let mut all_lcores = vec![main_lcore_id];
    all_lcores.extend(worker_cpus.iter().cloned());

    // Initialize DPDK as SECONDARY process (Primary is ctl-resource-manager)
    let dpdk_env = DpdkEnvBuilder::default()
        .process_type(DpdkProcessType::Secondary)
        .lcore_ids(all_lcores)
        .main_lcore_id(main_lcore_id)
        .build()?;

    println!("DPDK environment initialized as secondary process");
    println!();

    // Allocate worker CPUs to feed groups
    // For now, split workers evenly between configured feed kinds
    let mut available_workers = worker_cpus.clone();
    let num_feed_kinds = md_config.all_feeds().count();
    let workers_per_kind = if num_feed_kinds > 0 {
        available_workers.len() / num_feed_kinds
    } else {
        0
    };

    // Track all handles for multi-join
    let mut handles: Vec<MultiJoinHandle<Result<(), atx_feed::FeedGroupError>>> = Vec::new();

    // Create Top FeedGroup if configured
    let mut top_feedgroup = if md_config.find_feed("top").is_some() {
        let top_workers: Vec<DpdkLCoreId> = available_workers
            .drain(..workers_per_kind.min(available_workers.len()))
            .collect();

        if !top_workers.is_empty() {
            Some(create_top_feedgroup(
                &dpdk_env,
                &md_config,
                &symbol_info,
                top_workers,
            )?)
        } else {
            println!("[Warning] No workers available for TopFeedGroup");
            None
        }
    } else {
        None
    };

    // Create Trade FeedGroup if configured
    let mut trade_feedgroup = if md_config.find_feed("trade").is_some() {
        let trade_workers: Vec<DpdkLCoreId> = available_workers
            .drain(..workers_per_kind.min(available_workers.len()))
            .collect();

        if !trade_workers.is_empty() {
            Some(create_trade_feedgroup(
                &dpdk_env,
                &md_config,
                &symbol_info,
                trade_workers,
            )?)
        } else {
            println!("[Warning] No workers available for TradeFeedGroup");
            None
        }
    } else {
        None
    };

    // Run all feedgroups
    println!("\nStarting FeedGroup workers...\n");

    if let Some(ref mut fg) = top_feedgroup {
        let handle = fg.run()?;
        println!("[TopFeedGroup] Workers started on lcores: {:?}", handle.lcore_ids());
        handles.push(handle);
    }

    if let Some(ref mut fg) = trade_feedgroup {
        let handle = fg.run()?;
        println!("[TradeFeedGroup] Workers started on lcores: {:?}", handle.lcore_ids());
        handles.push(handle);
    }

    println!("\n=== Market Data Handler Running ===");
    println!("Polling for feedback and monitoring workers...\n");

    // Main coordination loop
    loop {
        // Poll feedback from all feedgroups
        if let Some(ref mut fg) = top_feedgroup {
            while let Some(feedback) = fg.poll_feedback() {
                handle_feedback("TopFeedGroup", feedback);
            }
        }

        if let Some(ref mut fg) = trade_feedgroup {
            while let Some(feedback) = fg.poll_feedback() {
                handle_feedback("TradeFeedGroup", feedback);
            }
        }

        // Check if any workers have completed/errored using try_join
        for (i, handle) in handles.iter().enumerate() {
            if let Some(result) = handle.try_join() {
                match result {
                    Ok(results) => {
                        for (j, worker_result) in results.into_iter().enumerate() {
                            if let Err(e) = worker_result {
                                eprintln!(
                                    "[Error] Handle {} Worker {} error: {:?}",
                                    i, j, e
                                );
                            }
                        }
                        println!("[Info] Handle {} workers completed", i);
                    }
                    Err(e) => {
                        eprintln!("[Error] Handle {} join error: {:?}", i, e);
                    }
                }
                // Worker completed - in production would restart or shutdown gracefully
                println!("[Warning] Workers completed unexpectedly, continuing...");
            }
        }

        // Small sleep to avoid busy-spinning on the main thread
        // In production, this could be replaced with more sophisticated event handling
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Note: This is unreachable in the current implementation
    // In production, we'd handle graceful shutdown via signals
    #[allow(unreachable_code)]
    Ok(())
}