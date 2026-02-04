pub mod i18n;
pub mod models;
pub mod scraper;
pub mod sources;

use crate::models::*;
use crate::scraper::{Config, Scraper};
use pyo3::prelude::*;
use std::fs;

/// Scrapes competition results based on the provided configuration file.
#[pyfunction]
fn run_scraper(config_path: String) -> PyResult<()> {
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

#[pymodule]
fn _dancing_datacollection(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_scraper, m)?)?;

    // Expose models to Python
    m.add_class::<Level>()?;
    m.add_class::<Style>()?;
    m.add_class::<Dance>()?;
    m.add_class::<AgeGroup>()?;
    m.add_class::<Judge>()?;
    m.add_class::<CommitteeMember>()?;
    m.add_class::<Officials>()?;
    m.add_class::<IdentityType>()?;
    m.add_class::<Participant>()?;
    m.add_class::<WDSFScore>()?;
    m.add_class::<Round>()?;
    m.add_class::<Competition>()?;
    m.add_class::<Event>()?;

    Ok(())
}
