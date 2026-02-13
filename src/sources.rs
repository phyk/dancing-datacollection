use thiserror::Error;

/// Error types for the scraping and parsing process.
#[derive(Error, Debug)]
pub enum ParsingError {
    /// Network-related failures.
    #[error("NETWORK_ERROR: {0}")]
    NetworkError(String),
    /// Required data elements are missing from the source.
    #[error("MISSING_REQUIRED_DATA: {0}")]
    MissingRequiredData(String),
    /// General parsing failures or structural changes in the source.
    #[error("PARSING_ERROR: {0}")]
    ParsingError(String),
    #[error("Missing required metadata: {0}")]
    MissingRequiredMetadata(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Invalid table structure: {0}")]
    InvalidTableStructure(String),
}

pub mod dtv_native;
