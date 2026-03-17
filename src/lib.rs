pub mod assets;
pub mod i18n;
pub mod models;
pub mod sources;

use pyo3::prelude::*;

#[pymodule]
fn _dancing_datacollection(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
