pub mod crawler;
pub mod i18n;
pub mod models;
pub mod sources;
pub mod storage;

use pyo3::prelude::*;

#[pymodule]
fn _dancing_datacollection(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crate::crawler::client::download_sources, m)?)?;
    m.add_function(wrap_pyfunction!(crate::sources::dtv_native::extract_competitions, m)?)?;
    m.add_function(wrap_pyfunction!(crate::models::validation::validate_extracted_competitions, m)?)?;
    m.add_function(wrap_pyfunction!(crate::crawler::client::collect_dancing_data, m)?)?;

    m.add_class::<crate::models::Event>()?;
    m.add_class::<crate::models::Competition>()?;
    m.add_class::<crate::models::Participant>()?;
    m.add_class::<crate::models::Level>()?;
    m.add_class::<crate::models::Style>()?;
    m.add_class::<crate::models::AgeGroup>()?;
    m.add_class::<crate::models::Dance>()?;
    m.add_class::<crate::models::IdentityType>()?;
    m.add_class::<crate::models::Judge>()?;
    m.add_class::<crate::models::CommitteeMember>()?;
    m.add_class::<crate::models::Officials>()?;
    m.add_class::<crate::models::WDSFScore>()?;
    m.add_class::<crate::models::Round>()?;
    m.add_class::<crate::storage::StorageManager>()?;

    Ok(())
}
