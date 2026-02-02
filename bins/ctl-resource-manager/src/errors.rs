use thiserror::Error;

/// Errors that can occur when parsing or validating the hardware resources configuration.
#[derive(Debug, Error)]
pub enum HwResourcesConfigError {
    /// Error reading the configuration file.
    #[error("Failed to read configuration file: {0}")]
    FileReadError(#[from] std::io::Error),
    /// Error parsing the YAML configuration.
    #[error("Failed to parse YAML configuration: {0}")]
    YamlParseError(#[from] serde_yaml::Error),
    /// Validation error with a descriptive message.
    #[error("Configuration validation error: {0}")]
    ValidationError(String),
}
