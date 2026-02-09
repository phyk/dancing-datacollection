use crate::models::Competition;
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
}

/// Trait for all competition result sources.
pub trait ResultSource {
    /// Returns the name of the source.
    fn name(&self) -> &str;

    /// Fetches the HTML content from the given URL.
    fn fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error>>;

    /// Parses the HTML content into a Competition model.
    fn parse(&self, html: &str) -> Result<Competition, ParsingError>;

    /// Parses a date string into a NaiveDate.
    fn parse_date(&self, s: &str) -> Option<chrono::NaiveDate>;
}

pub mod dtv_native;

/// Factory function to get the appropriate ResultSource for a given URL.
pub fn get_source_for_url(url: &str) -> Option<Box<dyn ResultSource>> {
    let url_lower = url.to_lowercase();
    if url_lower.contains("dancecomp.de")
        || url_lower.contains("topturnier.de")
        || url_lower.contains("tanzsport-hamburg.de")
        || url_lower.contains("hessen-tanzt.de")
        || url_lower.contains("nrw-tanzt.de")
    {
        Some(Box::new(dtv_native::DtvNative::new(
            dtv_native::SelectorConfig::default(),
        )))
    } else {
        None
    }
}
