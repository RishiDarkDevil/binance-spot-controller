//! Shared memory message types for ctl-feed.
//!
//! These types are used as the element types in DPDK shared memory rings.
//! Each type is registered via `register_ring!` for automatic allocation.

/// Maximum size for raw message buffer.
pub const RAW_MESSAGE_SIZE: usize = 512;

/// A raw message buffer for unparsed data.
///
/// This is a simple byte array used by DummyParser before proper
/// message types are implemented.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RawMessage {
    /// The raw bytes of the message.
    pub data: [u8; RAW_MESSAGE_SIZE],
}

impl Default for RawMessage {
    fn default() -> Self {
        Self {
            data: [0u8; RAW_MESSAGE_SIZE],
        }
    }
}

// Future: Add structured message types for different feed kinds
// 
// #[repr(C)]
// #[derive(Copy, Clone, Debug)]
// pub struct TopMessage {
//     pub symbol_id: u32,
//     pub bid_price: u64,  // Fixed-point price
//     pub bid_qty: u64,
//     pub ask_price: u64,
//     pub ask_qty: u64,
//     pub timestamp: u64,
// }
// 
// impl SharedMemSafe for TopMessage {}
// register_ring!(TopMessage, "TOP_PARSED_PUBSUB", 65536);
