use std::error::Error;

use atx_ring_registry::allocate_all_rings;
use dpdk::{DpdkEnvBuilder, DpdkProcessType};

// Import ctl_feed to ensure its ring registrations are linked.
// The `inventory` crate collects all `register_ring!` invocations at link time.
use ctl_feed as _;

fn main() -> Result<(), Box<dyn Error>> {

    // Initialize DPDK environment
    // TODO: Get the build options from a config file.
    let dpdk_env = DpdkEnvBuilder::default()
        .process_type(DpdkProcessType::Primary)
        .lcore_ids(vec![3])
        .build()?;

    // Allocate all rings registered via register_ring! macro.
    // This replaces manual ring allocation - any crate that uses register_ring!
    // and is linked into this binary will have its rings auto-allocated.
    let count = allocate_all_rings(&dpdk_env)?;
    println!("[ctl-resource-manager] Allocated {} shared memory rings", count);

    // Keep the primary process alive to maintain shared memory.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    #[allow(unreachable_code)]
    Ok(())
}