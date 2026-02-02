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

/// Errors that can occur when parsing or validating the symbol info configuration.
#[derive(Debug, Error)]
pub enum SymbolInfoConfigError {
    /// Error reading the configuration file.
    #[error("Failed to read symbol info file: {0}")]
    IoError(#[from] std::io::Error),
    /// Error parsing the YAML configuration.
    #[error("Failed to parse symbol info YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),
    /// Duplicate symbol ID found.
    #[error("Duplicate symbol ID: {0}")]
    DuplicateId(u32),
    /// Duplicate symbol name found.
    #[error("Duplicate symbol name: {0}")]
    DuplicateName(String),
}