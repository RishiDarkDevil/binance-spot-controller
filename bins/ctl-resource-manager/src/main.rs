use std::error::Error;
use std::fs;

use dpdk::{DpdkEnvBuilder, DpdkOwnedPubSubRing, DpdkProcessType};
use hashbrown::HashMap;

// Import ctl_feed to ensure its ring registrations are linked.
// The `inventory` crate collects all `register_ring!` invocations at link time.
use ctl_feed::RawMessage;
use ctl_md_handler::{HwResourcesConfig as MdHwResourcesConfig, SymbolInfoConfig};
use ctl_resource_manager::HwResourcesConfig;

const CONFIG_PATH: &str = "configs/resource-manager/hw-resources.yaml";
const MD_CONFIG_PATH: &str = "configs/market-data/hw-resources.yaml";
const SYMBOL_INFO_PATH: &str = "configs/market-data/symbolinfo.yaml";

fn main() -> Result<(), Box<dyn Error>> {
    // Load hardware resources configuration
    let config = HwResourcesConfig::from_file(CONFIG_PATH)?;

    // Load market data configuration
    let md_config = MdHwResourcesConfig::from_file(MD_CONFIG_PATH)?;

    // Load symbol info configuration
    let symbol_info = SymbolInfoConfig::from_file(SYMBOL_INFO_PATH)?;

    // Configure hugepages
    let hugepage_size = config.hugepages().size()?;
    let hugepage_count = config.hugepages().count;
    let sysfs_path = hugepage_size.sysfs_path();

    println!(
        "Configuring {} x {}kB hugepages via {}",
        hugepage_count,
        hugepage_size.size_kb(),
        sysfs_path
    );

    fs::write(sysfs_path, hugepage_count.to_string())
        .map_err(|e| format!("Failed to configure hugepages at {}: {}. Run as root?", sysfs_path, e))?;

    // Initialize DPDK environment with configured CPU core
    let dpdk_env = DpdkEnvBuilder::default()
        .process_type(DpdkProcessType::Primary)
        .lcore_ids(vec![config.lcore_id() as usize])
        .build()?;

    // Create PubSubRings for each symbol/kind combination
    // Ring naming convention: {KIND}_{symbol_id}_PS
    let mut rings: HashMap<String, DpdkOwnedPubSubRing<RawMessage>> = HashMap::new();

    for feed in md_config.all_feeds() {
        let kind = feed.kind.to_uppercase();

        // Get ring_size based on whether feed uses sets or direct config
        for symbol in feed.all_symbols() {
            let symbol_id = symbol_info
                .symbol_id(symbol)
                .ok_or_else(|| format!("Symbol '{}' not found in symbolinfo.yaml", symbol))?;

            // Get ring size for this symbol
            let ring_size = if feed.uses_sets() {
                // Find the set containing this symbol
                feed.sets
                    .iter()
                    .find(|set| set.symbols.iter().any(|s| s == symbol))
                    .map(|set| set.ring_size)
                    .ok_or_else(|| format!("Symbol '{}' not found in any set", symbol))?
            } else {
                feed.ring_size
                    .ok_or_else(|| format!("Feed '{}' missing ring_size", feed.kind))?
            };

            let ring_name = format!("{}_{}_PS", kind, symbol_id);

            println!(
                "Creating ring: {} (symbol: {}, size: {})",
                ring_name, symbol, ring_size
            );

            let ring = dpdk_env.pubsub_create::<RawMessage>(&ring_name, ring_size as usize)?;
            rings.insert(ring_name, ring);
        }
    }

    println!(
        "Created {} PubSubRings for market data feeds",
        rings.len()
    );

    // Keep the primary process alive to maintain shared memory.
    // The rings HashMap keeps all DpdkOwnedPubSubRing instances alive.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    #[allow(unreachable_code)]
    Ok(())
}