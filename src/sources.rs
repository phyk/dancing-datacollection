use thiserror::Error;

/// Error types for the scraping and parsing process.
#[derive(Error, Debug)]
pub enum ParsingError {
    /// Required data elements are missing from the source.
    #[error("MISSING_REQUIRED_DATA: {0}")]
    MissingRequiredData(String),
    /// General parsing failures or structural changes in the source.
    #[error("PARSING_ERROR: {0}")]
    ParsingError(String),
}

pub mod dtv_native;
pub mod html_gen;
pub mod topturnier_table;
