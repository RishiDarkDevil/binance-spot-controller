//! The Resource Manager is the core component of the ATX Trading System Controller.
//! It is responsible for allocating, owning, and managing all shared memory resources
//! that define the communication contracts between controller components.
//!
//! These resources form the architectural backbone of the system, enabling a
//! micro-service style controller design without relying on IPC or message brokers.
//! All inter-component communication occurs through shared memory regions
//! provisioned and governed by the Resource Manager.
//!
//! Components such as the Market Data Handler, Private Data Handler,
//! Order Management System, and Strategy Executor interact exclusively through
//! these shared resources, ensuring strict separation of concerns and
//! well-defined data ownership.
//!
//! The Resource Manager is the first component to be started and must remain
//! alive for the lifetime of the controller. If it terminates, all shared
//! memory contracts become invalid.
//!
//! Below is the Binance Spot Controller architecture as governed by the
//! Resource Manager.
//! 
//! 
//! ```text
//!                      +----------------------------------+
//!                      |        Market Data Handler       |
//!                      |----------------------------------|
//!                      | - Exchange connections           |
//!                      | - Subscription management        |
//!                      | - Feed normalization             |
//!                      +-----------------+----------------+
//!                                        |
//!                                        | writes
//!                                        v
//!  +====================================================================================+
//!  |                                Resource Manager                                    |
//!  |------------------------------------------------------------------------------------|
//!  |                                                                                    |
//!  |   +-------------------------------+                                                |
//!  |   |        Symbol Info Table      |  (shared, read-only for consumers)             |
//!  |   |-------------------------------|                                                |
//!  |   | - Symbol metadata             |                                                |
//!  |   | - Tick size / lot size        |                                                |
//!  |   | - Price & quantity filters    |                                                |
//!  |   | - Trading status              |                                                |
//!  |   +-------------------------------+                                                |
//!  |                                                                                    |
//!  |   +----------------------------------------------------------------------------+   |
//!  |   |                              Market Data Rings                             |   |
//!  |   |                                                                            |   |
//!  |   |   Symbol: BTCUSDT                                                          |   |
//!  |   |   +-------------------+        +-------------------+                       |   |
//!  |   |   |  Trade Ring       |        |   Top Ring        |                       |   |
//!  |   |   |  (pub-sub)        |        |  (pub-sub)        |                       |   |
//!  |   |   +-------------------+        +-------------------+                       |   |
//!  |   |                                                                            |   |
//!  |   |   Symbol: ETHUSDT                                                          |   |
//!  |   |   +-------------------+        +-------------------+                       |   |
//!  |   |   |  Trade Ring       |        |   Top Ring        |                       |   |
//!  |   |   |  (pub-sub)        |        |  (pub-sub)        |                       |   |
//!  |   |   +-------------------+        +-------------------+                       |   |
//!  |   |                                                                            |   |
//!  |   |   ...                                                                      |   |
//!  |   +----------------------------------------------------------------------------+   |
//!  |                                                                                    |
//!  +====================================================================================+
//!                                        |
//!                                        | reads
//!                                        v
//!             +-------------------+   +-------------------+   +-------------------+
//!             | Strategy Executor |   | Order Management  |   | Private Data      |
//!             |                   |   | System            |   | Handler           |
//!             +-------------------+   +-------------------+   +-------------------+
//! ```

mod config;
mod errors;

pub use config::{HugepageSize, HugepagesConfig, HwResourcesConfig};
pub use errors::HwResourcesConfigError;