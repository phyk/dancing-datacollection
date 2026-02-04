pub mod i18n;
pub mod models;
pub mod scraper;
pub mod sources;
pub mod storage;

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

    let dir_path = Path::new(&data_dir);
    let index_path = dir_path.join("index.htm");

    // Try to find at least one htm file if index doesn't exist
    let html = if index_path.exists() {
        fs::read_to_string(&index_path).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to read index file: {}", e))
        })?
    } else {
        // Fallback to erg.htm or similar if index is missing
        let erg_path = dir_path.join("erg.htm");
        if erg_path.exists() {
             fs::read_to_string(&erg_path).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!("Failed to read erg file: {}", e))
            })?
        } else {
            return Err(pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                "No valid htm files found in {}",
                data_dir
            )));
        }
    };

    let mut event = parser
        .parse(&html)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Parsing error: {}", e)))?;

    // Enrichment
    for comp in &mut event.competitions_list {
        // Look for related files in the same directory
        let files = ["erg.htm", "deck.htm", "tabges.htm", "ergwert.htm"];
        for file in files {
            let p = dir_path.join(file);
            if p.exists() {
                if let Ok(content) = fs::read_to_string(&p) {
                    match file {
                        "erg.htm" => {
                            if let Ok(parts) = parser.parse_participants(&content) {
                                comp.participants = parts;
                            }
                        }
                        "deck.htm" => {
                            if let Ok(off) = parser.parse_officials(&content) {
                                comp.officials = off;
                            }
                        }
                        "tabges.htm" | "ergwert.htm" => {
                            let rounds = parser.parse_rounds(&content, &comp.dances);
                            for r in rounds {
                                if !comp.rounds.iter().any(|existing| existing.name == r.name) {
                                    comp.rounds.push(r);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(PyEvent(event))
}

/// Checks whether the competitions extracted reproduce the downloaded sources (Fidelity Gate).
#[pyfunction]
fn validate_extracted_competitions(event: &PyEvent) -> PyResult<bool> {
    let event = &event.0;
    for comp in &event.competitions_list {
        // Fidelity Gate: A competition is invalid if it lacks Officials, Judges, or Results.
        if comp.officials.judges.is_empty() {
             return Ok(false);
        }
        if comp.participants.is_empty() {
            return Ok(false);
        }
        if comp.rounds.is_empty() {
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
            if let Ok(event) = extract_competitions(data_dir) {
                if validate_extracted_competitions(&event)? {
                    all_events.push(event);
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

    m.add_class::<PyEvent>()?;
    m.add_class::<crate::storage::StorageManager>()?;

    Ok(())
}
