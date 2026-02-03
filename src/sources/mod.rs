use crate::models::Event;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Missing required metadata: {0}")]
    MissingRequiredMetadata(String),
    #[error("Parse error: {0}")]
    Other(String),
}

pub trait ResultSource {
    fn parse(&self, html: &str) -> Result<Event, ParsingError>;
}

pub mod dtv_parser;
