//! Configuration module for the Resource Manager.
//!
//! This module provides the YAML parser and validation for hardware resources
//! configuration defined in `configs/resource-manager/hw-resources.yaml`.

use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::HwResourcesConfigError;

/// Hugepage size options in KB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugepageSize {
    /// 2MB hugepages (2048 KB)
    Size2MB,
    /// 1GB hugepages (1048576 KB)
    Size1GB,
}

impl HugepageSize {
    /// Returns the sysfs path for configuring this hugepage size.
    pub fn sysfs_path(&self) -> &'static str {
        match self {
            HugepageSize::Size2MB => "/sys/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages",
            HugepageSize::Size1GB => "/sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages",
        }
    }

    /// Returns the size in KB.
    pub fn size_kb(&self) -> u32 {
        match self {
            HugepageSize::Size2MB => 2048,
            HugepageSize::Size1GB => 1048576,
        }
    }
}

/// Hugepage configuration.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct HugepagesConfig {
    /// Hugepage size in KB (2048 for 2MB, 1048576 for 1GB).
    pub size_kb: u32,
    /// Number of hugepages to allocate.
    pub count: u32,
}

impl HugepagesConfig {
    /// Returns the hugepage size enum.
    pub fn size(&self) -> Result<HugepageSize, HwResourcesConfigError> {
        match self.size_kb {
            2048 => Ok(HugepageSize::Size2MB),
            1048576 => Ok(HugepageSize::Size1GB),
            _ => Err(HwResourcesConfigError::ValidationError(format!(
                "Invalid hugepage size: {}kB. Must be 2048 (2MB) or 1048576 (1GB)",
                self.size_kb
            ))),
        }
    }
}

/// Hardware resources configuration for the Resource Manager.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct HwResourcesConfig {
    /// CPU core to pin the resource manager process.
    pub cpu: u32,
    /// Hugepage configuration.
    pub hugepages: HugepagesConfig,
}

impl HwResourcesConfig {
    /// Loads and parses the configuration from a YAML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML configuration file.
    ///
    /// # Returns
    ///
    /// The parsed and validated configuration, or an error if parsing/validation fails.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, HwResourcesConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: HwResourcesConfig = serde_yaml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Validates the configuration.
    fn validate(&self) -> Result<(), HwResourcesConfigError> {
        // Validate hugepage size
        self.hugepages.size()?;
        
        // Validate hugepage count is non-zero
        if self.hugepages.count == 0 {
            return Err(HwResourcesConfigError::ValidationError(
                "Hugepage count must be greater than 0".to_string(),
            ));
        }
        
        Ok(())
    }

    /// Returns the CPU core ID for DPDK lcore configuration.
    pub fn lcore_id(&self) -> u32 {
        self.cpu
    }

    /// Returns the hugepage configuration.
    pub fn hugepages(&self) -> &HugepagesConfig {
        &self.hugepages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_valid_config() {
        let content = r#"
cpu: 3
hugepages:
  size_kb: 2048
  count: 128
"#;
        let file = create_temp_config(content);
        let config = HwResourcesConfig::from_file(file.path()).unwrap();
        assert_eq!(config.cpu, 3);
        assert_eq!(config.lcore_id(), 3);
        assert_eq!(config.hugepages.size_kb, 2048);
        assert_eq!(config.hugepages.count, 128);
    }

    #[test]
    fn test_parse_1gb_hugepages() {
        let content = r#"
cpu: 0
hugepages:
  size_kb: 1048576
  count: 4
"#;
        let file = create_temp_config(content);
        let config = HwResourcesConfig::from_file(file.path()).unwrap();
        assert_eq!(config.hugepages.size().unwrap(), HugepageSize::Size1GB);
        assert_eq!(config.hugepages.count, 4);
    }

    #[test]
    fn test_hugepage_sysfs_path() {
        assert_eq!(
            HugepageSize::Size2MB.sysfs_path(),
            "/sys/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages"
        );
        assert_eq!(
            HugepageSize::Size1GB.sysfs_path(),
            "/sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages"
        );
    }

    #[test]
    fn test_invalid_hugepage_size() {
        let content = r#"
cpu: 0
hugepages:
  size_kb: 4096
  count: 10
"#;
        let file = create_temp_config(content);
        let result = HwResourcesConfig::from_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_hugepage_count() {
        let content = r#"
cpu: 0
hugepages:
  size_kb: 2048
  count: 0
"#;
        let file = create_temp_config(content);
        let result = HwResourcesConfig::from_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_hugepages() {
        let content = r#"
cpu: 0
"#;
        let file = create_temp_config(content);
        let result = HwResourcesConfig::from_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_yaml() {
        let content = r#"
cpu: [invalid
"#;
        let file = create_temp_config(content);
        let result = HwResourcesConfig::from_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_file_not_found() {
        let result = HwResourcesConfig::from_file("/nonexistent/path/config.yaml");
        assert!(result.is_err());
    }
}
