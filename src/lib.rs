pub mod scraper;

use pyo3::prelude::*;
use crate::scraper::{Config, Scraper};
use std::fs;

#[pyfunction]
fn run_scraper(config_path: String) -> PyResult<()> {
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(format!("Failed to read config: {}", e)))?;
    let config: Config = toml::from_str(&config_content)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to parse config: {}", e)))?;

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut scraper = Scraper::new();
        scraper.scrape_all(&config).await.map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Scraper error: {}", e))
        })
    })
}

#[pymodule]
fn _dancing_datacollection(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_scraper, m)?)?;
    Ok(())
}
