pub mod assets;
pub mod crawler;
pub mod i18n;
pub mod models;
pub mod sources;
pub mod storage;

use pyo3::prelude::*;
use crate::models::sanitize_name;
use std::path::Path;
use std::fs;

/// Orchestrator to load, parse, validate, and store competition results.
///
/// This function follows a multi-stage process:
/// 1. Discovery & Crawling: Identifies competition links from the provided URL, respecting robots.txt and using a manifest for deduplication.
/// 2. Filtering: Applies optional case-insensitive filters for date, age group, style, and level.
/// 3. Fidelity Validation (Safety Shield): Ensures structural integrity, including judge counts and skating system math verification.
/// 4. Structured Storage: Saves validated results as JSON files in a directory hierarchy: {Event_Name}_{Year}/{AgeGroup}_{Level}_{Style}.json.
///
/// # Arguments
/// * `target_folder` - The base directory where the results and optional raw files will be stored.
/// * `url` - The URL of the competition or event index page to process.
/// * `date` - Optional date filter or override (e.g., "2024-05-01").
/// * `age_group` - Optional filter for a specific age group (e.g., "Adult", "Sen I").
/// * `style` - Optional filter for a specific dance style ("Standard" or "Latein").
/// * `level` - Optional filter for a specific skill level (e.g., "D", "C", "B", "A", "S").
/// * `download_html` - If True, archives raw HTML source files in a 'raw' subfolder.
/// * `output_format` - The serialization format for the output files (only "json" is supported).
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
            let temp_dir = Path::new(&target_folder).join(format!(
                "tmp_download_{}",
                sanitize_name(&comp_url)
            ));
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
            let comp_res =
                crate::sources::dtv_native::extract_event_data(&temp_dir.to_string_lossy());
            let mut comp = match comp_res {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to parse competition from {}: {}", comp_url, e);
                    let _ = fs::remove_dir_all(&temp_dir);
                    continue;
                }
            };

            comp.source_url = Some(comp_url.clone());

            // 3. Apply overrides and Validate
            // 3a. Filtering Logic
            if let Some(ref ag_filter) = age_group {
                let target = crate::i18n::map_age_group(ag_filter)
                    .or_else(|| crate::i18n::parse_age_group(ag_filter));
                if let Some(t) = target {
                    if comp.age_group != t {
                        let _ = fs::remove_dir_all(&temp_dir);
                        continue;
                    }
                } else {
                    let _ = fs::remove_dir_all(&temp_dir);
                    continue;
                }
            }
            if let Some(ref s_filter) = style {
                let target = crate::i18n::map_discipline(s_filter)
                    .or_else(|| crate::i18n::parse_style(s_filter));
                if let Some(t) = target {
                    if comp.style != t {
                        let _ = fs::remove_dir_all(&temp_dir);
                        continue;
                    }
                } else {
                    let _ = fs::remove_dir_all(&temp_dir);
                    continue;
                }
            }
            if let Some(ref l_filter) = level {
                let target = crate::i18n::parse_level(l_filter);
                if let Some(t) = target {
                    if comp.level != t {
                        let _ = fs::remove_dir_all(&temp_dir);
                        continue;
                    }
                } else {
                    let _ = fs::remove_dir_all(&temp_dir);
                    continue;
                }
            }

            if let Some(ref d_str) = date {
                match crate::sources::dtv_native::parse_date(d_str) {
                    Some(d) => {
                        if let Some(comp_date) = comp.date {
                            if comp_date != d {
                                let _ = fs::remove_dir_all(&temp_dir);
                                continue;
                            }
                        } else {
                            comp.date = Some(d);
                        }
                        comp.min_dances =
                            crate::i18n::get_min_dances(comp.level, d);
                    }
                    None => {
                        log::error!("Provided date filter '{}' could not be parsed.", d_str);
                        let _ = fs::remove_dir_all(&temp_dir);
                        continue;
                    }
                }
            } else if let Some(comp_date) = comp.date {
                // Ensure min_dances is correct for the parsed event date
                comp.min_dances = crate::i18n::get_min_dances(
                    comp.level,
                    comp_date,
                );
            }

            let comp_id = format!("{:?}_{:?}_{:?}", comp.age_group, comp.level, comp.style);
            let sanitized_comp_id = sanitize_name(&comp_id);

            // Math Check & Fidelity Gate (The Safety Shield)
            if !crate::models::validation::validate_competition_fidelity(&comp) {
                log::error!(
                    "CRITICAL_VALIDATION_ERROR: Competition {} failed fidelity gate or math check",
                    comp_id
                );
                let _ = fs::remove_dir_all(&temp_dir);
                continue;
            }

            // Mark as processed in manifest (for this run's deduplication)
            manifest.mark_processed(comp_url.clone());

            // 4. Folder structure: {Event_Name}_{Year}/
            let year = comp
                .date
                .map(|d| d.format("%Y").to_string())
                .unwrap_or_else(|| "0000".to_string());
            let sanitized_event_name = sanitize_name(&comp.name);
            let event_folder_name = format!("{}_{}", sanitized_event_name, year);
            let event_path = Path::new(&target_folder).join(&event_folder_name);
            if let Err(e) = fs::create_dir_all(&event_path) {
                log::error!("Failed to create event directory {:?}: {}", event_path, e);
                let _ = fs::remove_dir_all(&temp_dir);
                continue;
            }

            // 5. File naming: {Age}_{Level}_{Style}.json
            let json_path = event_path.join(format!("{}.json", sanitized_comp_id));
            match serde_json::to_string_pretty(&comp) {
                Ok(json_data) => {
                    if let Err(e) = fs::write(json_path, json_data) {
                        log::error!("Failed to write JSON file: {}", e);
                    }
                }
                Err(e) => log::error!("Failed to serialize competition: {}", e),
            }

            // 6. Handle raw HTML
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
