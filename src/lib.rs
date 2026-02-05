pub mod crawler;
pub mod i18n;
pub mod models;
pub mod sources;
pub mod storage;

use pyo3::prelude::*;
use crate::models::Event;

/// Opaque wrapper for Event to be passed to Python.
#[pyclass(name = "Event")]
#[derive(Clone)]
pub struct PyEvent(pub Event);

/// Scrapes the websites and saves the HTML files relevant for exporting data.
#[pyfunction]
fn download_sources(config_path: String) -> PyResult<()> {
    crate::crawler::client::run_download(&config_path)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Scraper error: {}", e)))
}

/// Extracts the competition data from saved HTML files in the given directory.
#[pyfunction]
fn extract_competitions(data_dir: String) -> PyResult<PyEvent> {
    crate::sources::dtv_native::extract_event_data(&data_dir)
        .map(|event| PyEvent(event))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Extraction error: {}", e)))
}

/// Checks whether the competitions extracted reproduce the downloaded sources (Fidelity Gate).
#[pyfunction]
fn validate_extracted_competitions(event: &PyEvent) -> bool {
    crate::models::validation::validate_event_fidelity(&event.0)
}

/// Orchestrator that calls the scraping, extraction, and validation steps.
#[pyfunction]
fn collect_dancing_data(config_path: String) -> PyResult<Vec<PyEvent>> {
    crate::crawler::client::collect_all_data(&config_path)
        .map(|events| events.into_iter().map(PyEvent).collect())
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Data collection error: {}", e)))
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
