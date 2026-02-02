//! Configuration module for the Market Data Handler.
//!
//! This module provides the YAML parser and validation for hardware resources
//! configuration defined in `configs/market-data/hw-resources.yaml`.

use atx_handler::{HandlerConfig, HandlerWorkerConfig};
use serde::Deserialize;
use hashbrown::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::ops::RangeInclusive;

use crate::{HwResourcesConfigError, SymbolInfoConfigError};

/// A protocol/parser combination for data transmission.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct Medium {
    /// Protocol type (e.g., "websocket").
    pub protocol: String,
    /// Parser type (e.g., "json", "sbe", "fix").
    pub parser: String,
}

impl Medium {
    /// Validates the medium configuration.
    fn validate(&self) -> Result<(), HwResourcesConfigError> {
        if self.protocol.is_empty() {
            return Err(HwResourcesConfigError::ValidationError(
                "Medium protocol cannot be empty".to_string(),
            ));
        }
        if self.parser.is_empty() {
            return Err(HwResourcesConfigError::ValidationError(
                "Medium parser cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// Returns a string representation of the medium.
    pub fn name(&self) -> String {
        format!("{}/{}", self.protocol, self.parser)
    }
}

/// A named set of symbols with their own CPU, ring buffer, and medium configuration.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SymbolSet {
    /// Name of this symbol set (e.g., "A", "B").
    pub name: String,
    /// Number of CPU cores to use for this set.
    pub num_cpus: u32,
    /// Ring buffer size for this set.
    pub ring_size: u32,
    /// List of symbols in this set.
    pub symbols: Vec<String>,
    /// List of protocol/parser mediums for this set.
    pub medium: Vec<Medium>,
}

impl SymbolSet {
    /// Validates the symbol set configuration.
    fn validate(&self) -> Result<(), HwResourcesConfigError> {
        // Validate set name is not empty
        if self.name.is_empty() {
            return Err(HwResourcesConfigError::ValidationError(
                "Symbol set name cannot be empty".to_string(),
            ));
        }

        // Validate ring_size is a power of 2
        if !self.ring_size.is_power_of_two() {
            return Err(HwResourcesConfigError::ValidationError(format!(
                "Ring size {} for set '{}' must be a power of 2",
                self.ring_size, self.name
            )));
        }

        // Validate symbols list is not empty
        if self.symbols.is_empty() {
            return Err(HwResourcesConfigError::ValidationError(format!(
                "Symbol set '{}' must have at least one symbol",
                self.name
            )));
        }

        // Validate each symbol is not empty
        for symbol in &self.symbols {
            if symbol.is_empty() {
                return Err(HwResourcesConfigError::ValidationError(format!(
                    "Symbol set '{}' contains an empty symbol",
                    self.name
                )));
            }
        }

        // Check for duplicate symbols within the set
        let mut seen = HashSet::new();
        for symbol in &self.symbols {
            if !seen.insert(symbol.as_str()) {
                return Err(HwResourcesConfigError::ValidationError(format!(
                    "Duplicate symbol '{}' in set '{}'",
                    symbol, self.name
                )));
            }
        }

        // Validate medium list is not empty
        if self.medium.is_empty() {
            return Err(HwResourcesConfigError::ValidationError(format!(
                "Symbol set '{}' must have at least one medium",
                self.name
            )));
        }

        // Validate each medium
        for m in &self.medium {
            m.validate()?;
        }

        // Check for duplicate mediums within the set
        let mut seen_mediums = HashSet::new();
        for m in &self.medium {
            if !seen_mediums.insert(m.name()) {
                return Err(HwResourcesConfigError::ValidationError(format!(
                    "Duplicate medium '{}' in set '{}'",
                    m.name(), self.name
                )));
            }
        }

        Ok(())
    }
}

/// Configuration for a single feed.
///
/// A feed can either have:
/// - Direct configuration with `num_cpus`, `ring_size`, `symbols`, and `medium`
/// - Named `sets` that group symbols with their own configurations including `medium`
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct FeedConfig {
    /// Feed kind (e.g., "top", "trade").
    pub kind: String,
    /// Number of CPU cores to use (used when not using sets).
    pub num_cpus: Option<u32>,
    /// Ring buffer size (used when not using sets).
    pub ring_size: Option<u32>,
    /// List of symbols for this feed (used when not using sets).
    #[serde(default)]
    pub symbols: Vec<String>,
    /// List of protocol/parser mediums (used when not using sets).
    #[serde(default)]
    pub medium: Vec<Medium>,
    /// Optional named symbol sets with individual configurations.
    #[serde(default)]
    pub sets: Vec<SymbolSet>,
}

impl FeedConfig {
    /// Returns the feed kind.
    pub fn name(&self) -> &str {
        &self.kind
    }

    /// Validates the feed configuration.
    fn validate(&self) -> Result<(), HwResourcesConfigError> {
        // Validate kind is not empty
        if self.kind.is_empty() {
            return Err(HwResourcesConfigError::ValidationError(
                "Feed kind cannot be empty".to_string(),
            ));
        }

        // Check if using sets or direct configuration
        let has_sets = !self.sets.is_empty();
        let has_direct = self.num_cpus.is_some() || self.ring_size.is_some() || !self.symbols.is_empty() || !self.medium.is_empty();

        if has_sets && has_direct {
            return Err(HwResourcesConfigError::ValidationError(format!(
                "Feed '{}' cannot have both 'sets' and direct configuration (num_cpus/ring_size/symbols/medium)",
                self.kind
            )));
        }

        if has_sets {
            // Validate all sets
            for set in &self.sets {
                set.validate()?;
            }

            // Check for duplicate set names
            let mut seen_names = HashSet::new();
            for set in &self.sets {
                if !seen_names.insert(set.name.as_str()) {
                    return Err(HwResourcesConfigError::ValidationError(format!(
                        "Duplicate set name '{}' in feed '{}'",
                        set.name, self.kind
                    )));
                }
            }

            // Check for duplicate symbols across all sets
            let mut all_symbols = HashSet::new();
            for set in &self.sets {
                for symbol in &set.symbols {
                    if !all_symbols.insert(symbol.as_str()) {
                        return Err(HwResourcesConfigError::ValidationError(format!(
                            "Duplicate symbol '{}' across sets in feed '{}'",
                            symbol, self.kind
                        )));
                    }
                }
            }
        } else {
            // Direct configuration - validate required fields
            if self.num_cpus.is_none() {
                return Err(HwResourcesConfigError::ValidationError(format!(
                    "Feed '{}' must specify 'num_cpus' when not using sets",
                    self.kind
                )));
            }

            if self.ring_size.is_none() {
                return Err(HwResourcesConfigError::ValidationError(format!(
                    "Feed '{}' must specify 'ring_size' when not using sets",
                    self.kind
                )));
            }

            // Validate ring_size is a power of 2
            if let Some(ring_size) = self.ring_size {
                if !ring_size.is_power_of_two() {
                    return Err(HwResourcesConfigError::ValidationError(format!(
                        "Ring size {} for feed '{}' must be a power of 2",
                        ring_size, self.kind
                    )));
                }
            }

            if self.symbols.is_empty() {
                return Err(HwResourcesConfigError::ValidationError(format!(
                    "Feed '{}' must have at least one symbol when not using sets",
                    self.kind
                )));
            }

            // Validate symbols are not empty
            for symbol in &self.symbols {
                if symbol.is_empty() {
                    return Err(HwResourcesConfigError::ValidationError(format!(
                        "Feed '{}' contains an empty symbol",
                        self.kind
                    )));
                }
            }

            // Check for duplicate symbols
            let mut seen = HashSet::new();
            for symbol in &self.symbols {
                if !seen.insert(symbol.as_str()) {
                    return Err(HwResourcesConfigError::ValidationError(format!(
                        "Duplicate symbol '{}' in feed '{}'",
                        symbol, self.kind
                    )));
                }
            }

            // Validate medium list is not empty
            if self.medium.is_empty() {
                return Err(HwResourcesConfigError::ValidationError(format!(
                    "Feed '{}' must have at least one medium when not using sets",
                    self.kind
                )));
            }

            // Validate each medium
            for m in &self.medium {
                m.validate()?;
            }

            // Check for duplicate mediums
            let mut seen_mediums = HashSet::new();
            for m in &self.medium {
                if !seen_mediums.insert(m.name()) {
                    return Err(HwResourcesConfigError::ValidationError(format!(
                        "Duplicate medium '{}' in feed '{}'",
                        m.name(), self.kind
                    )));
                }
            }
        }

        Ok(())
    }

    /// Returns all symbols across all sets or direct symbols.
    pub fn all_symbols(&self) -> Vec<&str> {
        if !self.sets.is_empty() {
            self.sets
                .iter()
                .flat_map(|s| s.symbols.iter().map(|sym| sym.as_str()))
                .collect()
        } else {
            self.symbols.iter().map(|s| s.as_str()).collect()
        }
    }

    /// Returns whether this feed uses symbol sets.
    pub fn uses_sets(&self) -> bool {
        !self.sets.is_empty()
    }

    /// Returns all mediums across all sets or direct medium list.
    pub fn all_mediums(&self) -> Vec<&Medium> {
        if !self.sets.is_empty() {
            self.sets
                .iter()
                .flat_map(|s| s.medium.iter())
                .collect()
        } else {
            self.medium.iter().collect()
        }
    }
}

/// Wrapper for a feed configuration in the YAML structure.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct FeedWrapper {
    /// The feed configuration.
    pub feed: FeedConfig,
}

/// Configuration for a pub/sub group.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PubSubConfig {
    /// List of feed configurations in this pub/sub group.
    pub pubsubs: Vec<FeedWrapper>,
}

impl PubSubConfig {
    /// Validates the pub/sub configuration.
    fn validate(&self) -> Result<(), HwResourcesConfigError> {
        if self.pubsubs.is_empty() {
            return Err(HwResourcesConfigError::ValidationError(
                "PubSub configuration must have at least one feed".to_string(),
            ));
        }

        // Validate each feed
        for feed_wrapper in &self.pubsubs {
            feed_wrapper.feed.validate()?;
        }

        // Check for duplicate feed kinds
        let mut seen_kinds = HashSet::new();
        for feed_wrapper in &self.pubsubs {
            let feed_kind = &feed_wrapper.feed.kind;
            if !seen_kinds.insert(feed_kind.clone()) {
                return Err(HwResourcesConfigError::ValidationError(format!(
                    "Duplicate feed kind '{}'",
                    feed_kind
                )));
            }
        }

        Ok(())
    }
}

/// Represents a single configuration item in the YAML root array.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum ConfigItem {
    MainCpu {
        main_cpu: u32,
    },
    WorkerCpus {
        worker_cpus: String,
    },
    PubSubs {
        pubsubs: Vec<FeedWrapper>,
    },
}

/// The root hardware resources configuration.
///
/// This represents the entire `hw-resources.yaml` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HwResourcesConfig {
    /// Main CPU core for the handler.
    pub main_cpu: u32,
    /// Worker CPU range (e.g., "1-12" -> 1..=12).
    pub worker_cpus: RangeInclusive<u32>,
    /// List of pub/sub configurations.
    pub pubsub_configs: Vec<PubSubConfig>,
}

impl HwResourcesConfig {
    /// Parses the hardware resources configuration from a YAML file.
    ///
    /// # Arguments
    /// * `path` - Path to the YAML configuration file.
    ///
    /// # Returns
    /// A validated `HwResourcesConfig` instance.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read, parsed, or fails validation.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, HwResourcesConfigError> {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parses the hardware resources configuration from a YAML string.
    ///
    /// # Arguments
    /// * `content` - YAML configuration string.
    ///
    /// # Returns
    /// A validated `HwResourcesConfig` instance.
    ///
    /// # Errors
    /// Returns an error if the YAML cannot be parsed or fails validation.
    pub fn from_str(content: &str) -> Result<Self, HwResourcesConfigError> {
        let items: Vec<ConfigItem> = serde_yaml::from_str(content)?;
        
        let mut main_cpu: Option<u32> = None;
        let mut worker_cpus: Option<String> = None;
        let mut pubsub_configs: Vec<PubSubConfig> = Vec::new();

        for item in items {
            match item {
                ConfigItem::MainCpu { main_cpu: cpu } => {
                    if main_cpu.is_some() {
                        return Err(HwResourcesConfigError::ValidationError(
                            "Duplicate 'main_cpu' configuration".to_string(),
                        ));
                    }
                    main_cpu = Some(cpu);
                }
                ConfigItem::WorkerCpus { worker_cpus: cpus } => {
                    if worker_cpus.is_some() {
                        return Err(HwResourcesConfigError::ValidationError(
                            "Duplicate 'worker_cpus' configuration".to_string(),
                        ));
                    }
                    worker_cpus = Some(cpus);
                }
                ConfigItem::PubSubs { pubsubs } => {
                    pubsub_configs.push(PubSubConfig { pubsubs });
                }
            }
        }

        let main_cpu = main_cpu.ok_or_else(|| {
            HwResourcesConfigError::ValidationError(
                "Missing 'main_cpu' configuration".to_string(),
            )
        })?;

        let worker_cpus_str = worker_cpus.ok_or_else(|| {
            HwResourcesConfigError::ValidationError(
                "Missing 'worker_cpus' configuration".to_string(),
            )
        })?;

        let worker_cpus = Self::parse_cpu_range(&worker_cpus_str)?;

        let config = Self {
            main_cpu,
            worker_cpus,
            pubsub_configs,
        };
        config.validate()?;
        Ok(config)
    }

    /// Parses a CPU range string like "1-12" into a RangeInclusive.
    fn parse_cpu_range(s: &str) -> Result<RangeInclusive<u32>, HwResourcesConfigError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(HwResourcesConfigError::ValidationError(format!(
                "Invalid worker_cpus format '{}'. Expected format: 'start-end' (e.g., '1-12')",
                s
            )));
        }

        let start: u32 = parts[0].trim().parse().map_err(|_| {
            HwResourcesConfigError::ValidationError(format!(
                "Invalid start CPU in worker_cpus '{}'",
                s
            ))
        })?;

        let end: u32 = parts[1].trim().parse().map_err(|_| {
            HwResourcesConfigError::ValidationError(format!(
                "Invalid end CPU in worker_cpus '{}'",
                s
            ))
        })?;

        if start > end {
            return Err(HwResourcesConfigError::ValidationError(format!(
                "Invalid worker_cpus range '{}': start ({}) must be <= end ({})",
                s, start, end
            )));
        }

        Ok(start..=end)
    }

    /// Validates the entire configuration.
    fn validate(&self) -> Result<(), HwResourcesConfigError> {
        if self.pubsub_configs.is_empty() {
            return Err(HwResourcesConfigError::ValidationError(
                "Configuration must have at least one pub/sub configuration".to_string(),
            ));
        }

        for pubsub in &self.pubsub_configs {
            pubsub.validate()?;
        }

        Ok(())
    }

    /// Returns an iterator over all feeds across all pub/sub configurations.
    pub fn all_feeds(&self) -> impl Iterator<Item = &FeedConfig> {
        self.pubsub_configs
            .iter()
            .flat_map(|ps| ps.pubsubs.iter().map(|fw| &fw.feed))
    }

    /// Finds a feed by kind.
    pub fn find_feed(&self, kind: &str) -> Option<&FeedConfig> {
        self.all_feeds().find(|f| f.kind == kind)
    }

    /// Returns all unique symbols across all feeds.
    pub fn all_symbols(&self) -> HashSet<&str> {
        self.all_feeds()
            .flat_map(|f| f.all_symbols())
            .collect()
    }
}

impl HandlerConfig for HwResourcesConfig {
    type ValidationError = HwResourcesConfigError;

    fn validate(&self) -> Result<(), Self::ValidationError> {
        // Re-use the internal validation
        HwResourcesConfig::validate(self)
    }
}

impl HandlerWorkerConfig for FeedConfig {
    type WorkerValidationError = HwResourcesConfigError;

    fn validate(&self) -> Result<(), Self::WorkerValidationError> {
        // Re-use the internal validation
        FeedConfig::validate(self)
    }
}

impl HandlerWorkerConfig for SymbolSet {
    type WorkerValidationError = HwResourcesConfigError;

    fn validate(&self) -> Result<(), Self::WorkerValidationError> {
        // Re-use the internal validation
        SymbolSet::validate(self)
    }
}

// ============================================================================
// Symbol Info Configuration
// ============================================================================

/// A single symbol's information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolInfo {
    /// Symbol name (e.g., "BTCUSDT").
    pub name: String,
    /// Unique numeric ID for the symbol.
    pub id: u32,
}

/// Helper struct for YAML parsing (matches the YAML format).
#[derive(Debug, Clone, Deserialize)]
struct SymbolInfoEntry {
    id: u32,
}

/// Configuration holding all symbol information.
///
/// Provides O(1) lookup by both symbol name and ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolInfoConfig {
    /// Symbols indexed by name for O(1) lookup.
    symbols_by_name: HashMap<String, SymbolInfo>,
    /// Symbols indexed by ID for O(1) lookup.
    symbols_by_id: HashMap<u32, SymbolInfo>,
}

impl SymbolInfoConfig {
    /// Parses the symbol info configuration from a YAML file.
    ///
    /// # Arguments
    /// * `path` - Path to the YAML configuration file.
    ///
    /// # Returns
    /// A validated `SymbolInfoConfig` instance.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read, parsed, or contains duplicates.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SymbolInfoConfigError> {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parses the symbol info configuration from a YAML string.
    ///
    /// # Arguments
    /// * `content` - YAML configuration string.
    ///
    /// # Returns
    /// A validated `SymbolInfoConfig` instance.
    ///
    /// # Errors
    /// Returns an error if the YAML cannot be parsed or contains duplicates.
    pub fn from_str(content: &str) -> Result<Self, SymbolInfoConfigError> {
        // Parse as Vec of single-key maps (use std::collections::HashMap for serde)
        let entries: Vec<std::collections::HashMap<String, SymbolInfoEntry>> =
            serde_yaml::from_str(content)?;

        let mut symbols_by_name = HashMap::new();
        let mut symbols_by_id = HashMap::new();

        for entry in entries {
            for (name, info) in entry {
                let symbol_info = SymbolInfo {
                    name: name.clone(),
                    id: info.id,
                };

                // Check for duplicate IDs
                if symbols_by_id.contains_key(&info.id) {
                    return Err(SymbolInfoConfigError::DuplicateId(info.id));
                }

                // Check for duplicate names
                if symbols_by_name.contains_key(&name) {
                    return Err(SymbolInfoConfigError::DuplicateName(name));
                }

                symbols_by_name.insert(name, symbol_info.clone());
                symbols_by_id.insert(info.id, symbol_info);
            }
        }

        Ok(Self {
            symbols_by_name,
            symbols_by_id,
        })
    }

    /// Get symbol info by name.
    pub fn get_by_name(&self, name: &str) -> Option<&SymbolInfo> {
        self.symbols_by_name.get(name)
    }

    /// Get symbol info by ID.
    pub fn get_by_id(&self, id: u32) -> Option<&SymbolInfo> {
        self.symbols_by_id.get(&id)
    }

    /// Get symbol ID by name.
    pub fn symbol_id(&self, name: &str) -> Option<u32> {
        self.symbols_by_name.get(name).map(|s| s.id)
    }

    /// Iterator over all symbols.
    pub fn symbols(&self) -> impl Iterator<Item = &SymbolInfo> {
        self.symbols_by_name.values()
    }

    /// Number of symbols.
    pub fn len(&self) -> usize {
        self.symbols_by_name.len()
    }

    /// Returns true if there are no symbols.
    pub fn is_empty(&self) -> bool {
        self.symbols_by_name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_CONFIG: &str = r#"
- main_cpu: 0
- worker_cpus: 1-12
- pubsubs:
    - feed:
        kind: top
        sets:
          - name: A
            num_cpus: 4
            ring_size: 65536
            symbols:
              - BTCUSDT
              - ETHUSDT
              - SOLUSDT
            medium:
              - protocol: websocket
                parser: json
              - protocol: websocket
                parser: sbe
          - name: B
            num_cpus: 4
            ring_size: 65536
            symbols:
              - ADAUSDT
              - XRPUSDT
              - DOTUSDT
            medium:
              - protocol: websocket
                parser: json
    - feed:
        kind: trade
        num_cpus: 4
        ring_size: 65536
        symbols:
          - BTCUSDT
          - ETHUSDT
          - SOLUSDT
        medium:
          - protocol: websocket
            parser: json
"#;

    #[test]
    fn test_parse_valid_config() {
        let config = HwResourcesConfig::from_str(VALID_CONFIG).expect("Failed to parse config");
        
        assert_eq!(config.main_cpu, 0);
        assert_eq!(config.worker_cpus, 1..=12);
        assert_eq!(config.pubsub_configs.len(), 1);
        assert_eq!(config.pubsub_configs[0].pubsubs.len(), 2);
        
        let top_feed = config.find_feed("top").expect("top feed not found");
        assert!(top_feed.uses_sets());
        assert_eq!(top_feed.sets.len(), 2);
        assert_eq!(top_feed.kind, "top");
        assert_eq!(top_feed.sets[0].medium.len(), 2);
        
        let trade_feed = config.find_feed("trade").expect("trade feed not found");
        assert!(!trade_feed.uses_sets());
        assert_eq!(trade_feed.num_cpus, Some(4));
        assert_eq!(trade_feed.ring_size, Some(65536));
        assert_eq!(trade_feed.symbols.len(), 3);
        assert_eq!(trade_feed.medium.len(), 1);
    }

    #[test]
    fn test_all_symbols() {
        let config = HwResourcesConfig::from_str(VALID_CONFIG).expect("Failed to parse config");
        let symbols = config.all_symbols();
        
        assert!(symbols.contains("BTCUSDT"));
        assert!(symbols.contains("ETHUSDT"));
        assert!(symbols.contains("SOLUSDT"));
        assert!(symbols.contains("ADAUSDT"));
        assert!(symbols.contains("XRPUSDT"));
        assert!(symbols.contains("DOTUSDT"));
    }

    #[test]
    fn test_all_mediums() {
        let config = HwResourcesConfig::from_str(VALID_CONFIG).expect("Failed to parse config");
        
        let top_feed = config.find_feed("top").expect("top feed not found");
        let mediums = top_feed.all_mediums();
        assert_eq!(mediums.len(), 3); // 2 from set A + 1 from set B
        
        let trade_feed = config.find_feed("trade").expect("trade feed not found");
        let mediums = trade_feed.all_mediums();
        assert_eq!(mediums.len(), 1);
        assert_eq!(mediums[0].protocol, "websocket");
        assert_eq!(mediums[0].parser, "json");
    }

    #[test]
    fn test_invalid_ring_size() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        num_cpus: 1
        ring_size: 1000
        symbols:
          - TEST
        medium:
          - protocol: websocket
            parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("power of 2"));
    }

    #[test]
    fn test_empty_kind() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: ""
        num_cpus: 1
        ring_size: 1024
        symbols:
          - TEST
        medium:
          - protocol: websocket
            parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("kind cannot be empty"));
    }

    #[test]
    fn test_duplicate_symbols() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        num_cpus: 1
        ring_size: 1024
        symbols:
          - TEST
          - TEST
        medium:
          - protocol: websocket
            parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate symbol"));
    }

    #[test]
    fn test_duplicate_set_names() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        sets:
          - name: A
            num_cpus: 1
            ring_size: 1024
            symbols:
              - TEST1
            medium:
              - protocol: websocket
                parser: json
          - name: A
            num_cpus: 2
            ring_size: 1024
            symbols:
              - TEST2
            medium:
              - protocol: websocket
                parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate set name"));
    }

    #[test]
    fn test_missing_num_cpus_without_sets() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        ring_size: 1024
        symbols:
          - TEST
        medium:
          - protocol: websocket
            parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must specify 'num_cpus'"));
    }

    #[test]
    fn test_empty_pubsubs() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs: []
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one feed"));
    }

    #[test]
    fn test_missing_medium() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        num_cpus: 1
        ring_size: 1024
        symbols:
          - TEST
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one medium"));
    }

    #[test]
    fn test_empty_protocol_in_medium() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        num_cpus: 1
        ring_size: 1024
        symbols:
          - TEST
        medium:
          - protocol: ""
            parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("protocol cannot be empty"));
    }

    #[test]
    fn test_empty_parser_in_medium() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        num_cpus: 1
        ring_size: 1024
        symbols:
          - TEST
        medium:
          - protocol: websocket
            parser: ""
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parser cannot be empty"));
    }

    #[test]
    fn test_missing_main_cpu() {
        let config_str = r#"
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        num_cpus: 1
        ring_size: 1024
        symbols:
          - TEST
        medium:
          - protocol: websocket
            parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'main_cpu'"));
    }

    #[test]
    fn test_missing_worker_cpus() {
        let config_str = r#"
- main_cpu: 0
- pubsubs:
    - feed:
        kind: test
        num_cpus: 1
        ring_size: 1024
        symbols:
          - TEST
        medium:
          - protocol: websocket
            parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'worker_cpus'"));
    }

    #[test]
    fn test_invalid_worker_cpus_format() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1,2,3
- pubsubs:
    - feed:
        kind: test
        num_cpus: 1
        ring_size: 1024
        symbols:
          - TEST
        medium:
          - protocol: websocket
            parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid worker_cpus format"));
    }

    #[test]
    fn test_duplicate_medium_in_set() {
        let config_str = r#"
- main_cpu: 0
- worker_cpus: 1-4
- pubsubs:
    - feed:
        kind: test
        sets:
          - name: A
            num_cpus: 1
            ring_size: 1024
            symbols:
              - TEST
            medium:
              - protocol: websocket
                parser: json
              - protocol: websocket
                parser: json
"#;
        let result = HwResourcesConfig::from_str(config_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate medium"));
    }
}
