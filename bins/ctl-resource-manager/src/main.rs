use std::error::Error;
use std::fs;

use dpdk::{DpdkEnvBuilder, DpdkProcessType};

// Import ctl_feed to ensure its ring registrations are linked.
// The `inventory` crate collects all `register_ring!` invocations at link time.
use ctl_feed as _;
use ctl_resource_manager::HwResourcesConfig;

const CONFIG_PATH: &str = "configs/resource-manager/hw-resources.yaml";

fn main() -> Result<(), Box<dyn Error>> {
    // Load hardware resources configuration
    let config = HwResourcesConfig::from_file(CONFIG_PATH)?;

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

    // Keep the primary process alive to maintain shared memory.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    #[allow(unreachable_code)]
    Ok(())
}