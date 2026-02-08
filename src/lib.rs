pub mod crawler;
pub mod i18n;
pub mod models;
pub mod sources;
pub mod storage;

use pyo3::prelude::*;
use crate::models::{Level, sanitize_name};
use std::str::FromStr;
use std::path::Path;
use std::fs;

/// Orchestrator to load, parse, validate, and store competition results.
///
/// This is the primary entry point for the Python API.
#[pyfunction]
#[pyo3(signature = (target_folder, url, date=None, age_group=None, style=None, level=None, download_html=true, output_format="json"))]
#[allow(clippy::too_many_arguments)]
fn load_competition_results(
    target_folder: String,
    url: String,
    date: Option<String>,
    age_group: Option<String>,
    style: Option<String>,
    level: Option<String>,
    download_html: bool,
    output_format: &str,
) -> PyResult<()> {
    if output_format != "json" {
        return Err(pyo3::exceptions::PyValueError::new_err(format!("Unsupported output format: {}", output_format)));
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create tokio runtime: {}", e))
    })?;

    let parser = crate::sources::get_source_for_url(&url)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("No parser found for URL: {}", url)))?;

    let mut scraper = crate::crawler::client::Scraper::new();

    let mut manifest = crate::crawler::manifest::Manifest::from_target_folder(Path::new(&target_folder));

    rt.block_on(async {
        // 1. Determine if URL is event index or single competition
        let competition_links = scraper.get_competition_links(&url).await.unwrap_or_default();

        let urls_to_process = if competition_links.is_empty() {
            vec![url.clone()]
        } else {
            competition_links
        };

        for comp_url in urls_to_process {
            if manifest.is_processed(&comp_url) {
                log::info!("Skipping already processed URL: {}", comp_url);
                continue;
            }

             // Create a temp directory for this competition
             let temp_dir = Path::new(&target_folder).join(format!("tmp_download_{}", sanitize_name(&comp_url)));
             if temp_dir.exists() {
                 let _ = fs::remove_dir_all(&temp_dir);
             }
             if let Err(e) = fs::create_dir_all(&temp_dir) {
                 log::error!("Failed to create temp directory {:?}: {}", temp_dir, e);
                 continue;
             }

             let download_res = scraper.download_competition_files(&comp_url, &temp_dir).await;
             if let Err(e) = download_res {
                 log::error!("Failed to download competition from {}: {}", comp_url, e);
                 let _ = fs::remove_dir_all(&temp_dir);
                 continue;
             }

             // 2. Parse
             let event_res = crate::sources::dtv_native::extract_event_data(&temp_dir.to_string_lossy());
             let mut event = match event_res {
                 Ok(e) => e,
                 Err(e) => {
                     log::error!("Failed to parse competition from {}: {}", comp_url, e);
                     let _ = fs::remove_dir_all(&temp_dir);
                     continue;
                 }
             };

             event.source_url = Some(comp_url.clone());

             let year = event.date.map(|d| d.format("%Y").to_string()).unwrap_or_else(|| "0000".to_string());
             let sanitized_event_name = sanitize_name(&event.name);
             let event_folder_name = format!("{}_{}", sanitized_event_name, year);
             let event_path = Path::new(&target_folder).join(&event_folder_name);
             if let Err(e) = fs::create_dir_all(&event_path) {
                 log::error!("Failed to create event directory {:?}: {}", event_path, e);
                 let _ = fs::remove_dir_all(&temp_dir);
                 continue;
             }

             // 3. Apply overrides and Validate
             let competitions = event.competitions_list.clone();
             let mut event_metadata = event.clone();
             event_metadata.competitions_list = vec![];

             for mut comp in competitions {
                 if let Some(ref d_str) = date {
                     if let Some(d) = parser.parse_date(d_str) {
                         event_metadata.date = Some(d);
                         comp.min_dances = crate::models::validation::get_min_dances_for_level(&comp.level, &d);
                     }
                 }
                 if let Some(ref ag_str) = age_group {
                     if let Some(ag) = crate::i18n::map_age_group(ag_str) { comp.age_group = ag; }
                 }
                 if let Some(ref s_str) = style {
                     if let Some(s) = crate::i18n::map_discipline(s_str) { comp.style = s; }
                 }
                 if let Some(ref l_str) = level {
                     if let Some(l) = Level::from_str(l_str).ok() { comp.level = l; }
                 }

                 let comp_id = format!("{:?}_{:?}_{:?}", comp.age_group, comp.level, comp.style);
                 let sanitized_comp_id = sanitize_name(&comp_id);

                 // Math Check & Fidelity Gate
                 let mut single_comp_event = event_metadata.clone();
                 single_comp_event.competitions_list = vec![comp];

                 if !crate::models::validation::validate_event_fidelity(&single_comp_event) {
                     log::error!("CRITICAL_VALIDATION_ERROR: Competition {} failed fidelity gate or math check", comp_id);
                     continue;
                 }

                 // Mark as processed in manifest (for this run's deduplication)
                 manifest.mark_processed(comp_url.clone());

                 // 4. Save JSON
                 let json_path = event_path.join(format!("{}.json", sanitized_comp_id));
                 match serde_json::to_string_pretty(&single_comp_event) {
                    Ok(json_data) => {
                        if let Err(e) = fs::write(json_path, json_data) {
                            log::error!("Failed to write JSON file: {}", e);
                        }
                    }
                    Err(e) => log::error!("Failed to serialize event: {}", e),
                 }

                 // 5. Handle raw HTML
                 if download_html {
                     let raw_path = event_path.join("raw").join(&sanitized_comp_id);
                     if let Ok(_) = fs::create_dir_all(&raw_path) {
                         if let Ok(entries) = fs::read_dir(&temp_dir) {
                             for entry in entries.flatten() {
                                 let dest = raw_path.join(entry.file_name());
                                 let _ = fs::copy(entry.path(), dest);
                             }
                         }
                     }
                 }
             }

             // Cleanup temp dir
             let _ = fs::remove_dir_all(&temp_dir);
        }
        Ok(())
    })
}

#[pymodule]
fn _dancing_datacollection(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load_competition_results, m)?)?;
    Ok(())
}
