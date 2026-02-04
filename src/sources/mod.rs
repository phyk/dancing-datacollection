use crate::models::Event;
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
}

/// Trait for all competition result sources.
pub trait ResultSource {
    /// Returns the name of the source.
    fn name(&self) -> &str;

    /// Fetches the HTML content from the given URL.
    fn fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error>>;

    /// Parses the HTML content into an Event model.
    fn parse(&self, html: &str) -> Result<Event, ParsingError>;
}

pub mod dtv_parser;
