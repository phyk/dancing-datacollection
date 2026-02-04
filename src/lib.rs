pub mod i18n;
pub mod models;
pub mod scraper;
pub mod sources;

use crate::i18n::I18n;
use crate::models::*;
use crate::scraper::{Config, Scraper};
use crate::sources::dtv_parser::{DtvParser, SelectorConfig};
use crate::sources::ResultSource;
use pyo3::prelude::*;
use std::fs;
use std::path::Path;

/// Opaque wrapper for Event to be passed to Python.
#[pyclass]
#[derive(Clone)]
pub struct PyEvent(pub Event);

/// Scrapes the websites and saves the HTML files relevant for exporting data.
#[pyfunction]
fn download_sources(config_path: String) -> PyResult<()> {
    let config_content = fs::read_to_string(&config_path).map_err(|e| {
        pyo3::exceptions::PyIOError::new_err(format!("Failed to read config: {}", e))
    })?;
    let config: Config = toml::from_str(&config_content).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse config: {}", e))
    })?;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut scraper = Scraper::new();
        scraper
            .scrape_all(&config)
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Scraper error: {}", e)))
    })
}

/// Helper function to load config and i18n.
fn load_config_and_i18n(config_path: &str) -> PyResult<(Config, I18n)> {
    let config_content = fs::read_to_string(config_path).map_err(|e| {
        pyo3::exceptions::PyIOError::new_err(format!("Failed to read config: {}", e))
    })?;
    let config: Config = toml::from_str(&config_content).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse config: {}", e))
    })?;

    let aliases_path = "config/aliases.toml";
    let i18n = I18n::new(aliases_path).map_err(|e| {
        pyo3::exceptions::PyIOError::new_err(format!("Failed to read aliases: {}", e))
    })?;

    Ok((config, i18n))
}

/// Extracts the competition data from saved HTML files in the given directory.
#[pyfunction]
fn extract_competitions(data_dir: String) -> PyResult<PyEvent> {
    let config_path = "config/config.toml";
    let (config, i18n) = load_config_and_i18n(config_path)?;
    let parser = DtvParser::new(config, SelectorConfig::default(), i18n);

    let index_path = Path::new(&data_dir).join("index.htm");
    if !index_path.exists() {
        return Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
            "Index file not found in {}",
            data_dir
        )));
    }

    let html = fs::read_to_string(index_path).map_err(|e| {
        pyo3::exceptions::PyIOError::new_err(format!("Failed to read index file: {}", e))
    })?;

    let event = parser
        .parse(&html)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Parsing error: {}", e)))?;

    Ok(PyEvent(event))
}

/// Checks whether the competitions extracted reproduce the downloaded sources (Fidelity Gate).
#[pyfunction]
fn validate_extracted_competitions(event: &PyEvent) -> PyResult<bool> {
    let event = &event.0;
    for comp in &event.competitions_list {
        if comp.dances.is_empty() {
            return Ok(false);
        }
    }
    Ok(!event.competitions_list.is_empty())
}

/// Orchestrator that calls the scraping, extraction, and validation steps.
#[pyfunction]
fn collect_dancing_data(config_path: String) -> PyResult<Vec<PyEvent>> {
    download_sources(config_path.clone())?;

    let mut all_events = Vec::new();

    let entries = fs::read_dir("data").map_err(|e| {
        pyo3::exceptions::PyIOError::new_err(format!("Failed to read data dir: {}", e))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        if entry.path().is_dir() {
            let data_dir = entry.path().to_string_lossy().to_string();
            let (config, i18n) = load_config_and_i18n(&config_path)?;
            let parser = DtvParser::new(config, SelectorConfig::default(), i18n);

            let index_path = Path::new(&data_dir).join("index.htm");
            if index_path.exists() {
                let html = fs::read_to_string(index_path).map_err(|e| {
                    pyo3::exceptions::PyIOError::new_err(format!("Failed to read index file: {}", e))
                })?;

                if let Ok(event) = parser.parse(&html) {
                    let py_event = PyEvent(event);
                    if validate_extracted_competitions(&py_event)? {
                        all_events.push(py_event);
                    }
                }
            }
        }
    }

    if all_events.is_empty() {
        Err(pyo3::exceptions::PyRuntimeError::new_err(
            "No valid competition data collected",
        ))
    } else {
        Ok(all_events)
    }
}

#[pymodule]
fn _dancing_datacollection(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(download_sources, m)?)?;
    m.add_function(wrap_pyfunction!(extract_competitions, m)?)?;
    m.add_function(wrap_pyfunction!(validate_extracted_competitions, m)?)?;
    m.add_function(wrap_pyfunction!(collect_dancing_data, m)?)?;

    Ok(())
}
