pub mod assets;
pub mod crawler;
pub mod i18n;
pub mod models;
pub mod sources;

use crate::models::{sanitize_name, AgeGroup, Level, Style};
use chrono::NaiveDate;
use pyo3::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Typed, already-validated filters for `run_pipeline`.
/// Parsing happens up front so invalid user input fails fast instead of silently matching nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Filters {
    pub date: Option<NaiveDate>,
    pub age_group: Option<AgeGroup>,
    pub style: Option<Style>,
    pub level: Option<Level>,
}

/// Parses raw filter strings into typed `Filters`; shared by the Python API and the CLI.
/// Dates accept `YYYY-MM-DD` (the documented format) and fall back to the DTV `dd.mm.yyyy` style.
/// The error message names the offending parameter and value.
pub fn parse_filters(
    date: Option<&str>,
    age_group: Option<&str>,
    style: Option<&str>,
    level: Option<&str>,
) -> Result<Filters, String> {
    let date = date
        .map(|s| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .or_else(|| crate::sources::dtv_native::parse_date(s))
                .ok_or_else(|| format!("Invalid date '{}': expected YYYY-MM-DD", s))
        })
        .transpose()?;
    let age_group = age_group
        .map(|s| {
            crate::i18n::map_age_group(s)
                .or_else(|| crate::i18n::parse_age_group(s))
                .ok_or_else(|| format!("Invalid age_group '{}'", s))
        })
        .transpose()?;
    let style = style
        .map(|s| {
            crate::i18n::map_discipline(s)
                .or_else(|| crate::i18n::parse_style(s))
                .ok_or_else(|| format!("Invalid style '{}'", s))
        })
        .transpose()?;
    let level = level
        .map(|s| crate::i18n::parse_level(s).ok_or_else(|| format!("Invalid level '{}'", s)))
        .transpose()?;
    Ok(Filters {
        date,
        age_group,
        style,
        level,
    })
}

/// Orchestrator to load, parse, validate, and store competition results.
///
/// This function follows a multi-stage process:
/// 1. Discovery & Crawling: Identifies competition links from the provided URL, respecting robots.txt and using a manifest for deduplication.
/// 2. Filtering: Applies optional case-insensitive filters for date, age group, style, and level.
/// 3. Fidelity Validation (Safety Shield): Ensures structural integrity, including judge counts and skating system math verification.
/// 4. Structured Storage: Saves validated results as JSON files in a directory hierarchy: {Event_Name}_{Year}/{AgeGroup}_{Level}_{Style}_{YYYY-MM-DD}.json.
///
/// # Arguments
/// * `target_folder` - The base directory where the results and optional raw files will be stored.
/// * `url` - The URL of the competition or event index page to process.
/// * `date` - Optional date filter or override in `YYYY-MM-DD` format (e.g., "2024-05-01").
/// * `age_group` - Optional filter for a specific age group (e.g., "Adult", "Sen I").
/// * `style` - Optional filter for a specific dance style ("Standard" or "Latein").
/// * `level` - Optional filter for a specific skill level (e.g., "D", "C", "B", "A", "S").
/// * `download_html` - If True, archives raw HTML source files in a 'raw' subfolder.
/// * `output_format` - The serialization format for the output files (only "json" is supported).
///
/// Invalid filter values raise `ValueError`. Progress and errors are reported through
/// Python's `logging` module (logger names start with `dancing_datacollection`).
#[pyfunction]
#[pyo3(signature = (target_folder, url, date=None, age_group=None, style=None, level=None, download_html=true, output_format="json"))]
#[allow(clippy::too_many_arguments)]
fn load_competition_results(
    py: Python<'_>,
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
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unsupported output format: {}",
            output_format
        )));
    }

    let filters = parse_filters(
        date.as_deref(),
        age_group.as_deref(),
        style.as_deref(),
        level.as_deref(),
    )
    .map_err(pyo3::exceptions::PyValueError::new_err)?;

    py.allow_threads(|| run_pipeline(Path::new(&target_folder), &url, &filters, download_html))
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:#}", e)))?;
    Ok(())
}

/// Pure-Rust pipeline behind `load_competition_results` and the CLI `download` command.
/// Per-competition failures are logged and skipped; returns the number of competitions saved.
pub fn run_pipeline(
    target_folder: &Path,
    url: &str,
    filters: &Filters,
    download_html: bool,
) -> anyhow::Result<usize> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| anyhow::anyhow!("Failed to create tokio runtime: {}", e))?;

    let mut scraper = crate::crawler::client::Scraper::new();
    let mut manifest = crate::crawler::manifest::Manifest::from_target_folder(target_folder);
    let mut saved = 0usize;

    rt.block_on(async {
        // 1. Determine if URL is event index or single competition
        let competition_links = match scraper.get_competition_links(url).await {
            Ok(links) => links,
            Err(e) => {
                log::warn!(
                    "Failed to fetch competition index {}: {}; treating it as a single competition",
                    url,
                    e
                );
                Vec::new()
            }
        };

        let urls_to_process = if competition_links.is_empty() {
            vec![url.to_string()]
        } else {
            competition_links
        };

        for (index, comp_url) in urls_to_process.into_iter().enumerate() {
            if manifest.is_processed(&comp_url) {
                log::info!("Skipping already processed URL: {}", comp_url);
                continue;
            }

            // Index-based name: sanitized URLs are truncated and could collide.
            let temp_dir = target_folder.join(format!("tmp_download_{}", index));
            if temp_dir.exists() {
                let _ = fs::remove_dir_all(&temp_dir);
            }
            if let Err(e) = fs::create_dir_all(&temp_dir) {
                log::error!("Failed to create temp directory {:?}: {}", temp_dir, e);
                continue;
            }

            let download_res = scraper
                .download_competition_files(&comp_url, &temp_dir)
                .await;
            if let Err(e) = download_res {
                log::error!("NETWORK_ERROR: Skipping competition {}: {:#}", comp_url, e);
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
            if !apply_filters_and_overrides(&mut comp, filters) {
                let _ = fs::remove_dir_all(&temp_dir);
                continue;
            }

            let comp_id = format!("{:?}_{:?}_{:?}", comp.age_group, comp.level, comp.style);

            // Math Check & Fidelity Gate (The Safety Shield)
            if !crate::models::validation::validate_competition_fidelity(&comp) {
                log::error!(
                    "CRITICAL_VALIDATION_ERROR: Competition {} failed fidelity gate or math check",
                    comp_id
                );
                let _ = fs::remove_dir_all(&temp_dir);
                continue;
            }

            // 4. Folder structure: {Event_Name}_{Year}/
            let year = comp
                .date
                .map(|d| d.format("%Y").to_string())
                .unwrap_or_else(|| "0000".to_string());
            let sanitized_event_name = sanitize_name(&strip_floor_note(&comp.name));
            let event_folder_name = format!("{}_{}", sanitized_event_name, year);
            let event_path = target_folder.join(&event_folder_name);
            if let Err(e) = fs::create_dir_all(&event_path) {
                log::error!("Failed to create event directory {:?}: {}", event_path, e);
                let _ = fs::remove_dir_all(&temp_dir);
                continue;
            }

            // 5. File naming: {Age}_{Level}_{Style}_{Date}.json
            let (output_id, json_path) = output_path(&event_path, &comp, &comp_url);
            let json_data = match serde_json::to_string_pretty(&comp) {
                Ok(d) => d,
                Err(e) => {
                    log::error!("Failed to serialize competition: {}", e);
                    let _ = fs::remove_dir_all(&temp_dir);
                    continue;
                }
            };
            if let Err(e) = fs::write(&json_path, json_data) {
                log::error!("Failed to write JSON file {:?}: {}", json_path, e);
                let _ = fs::remove_dir_all(&temp_dir);
                continue;
            }

            // Only count and mark as processed once the JSON is on disk.
            manifest.mark_processed(comp_url.clone());
            saved += 1;
            log::info!("Saved {} to {:?}", comp_url, json_path);

            // 6. Handle raw HTML
            if download_html {
                let raw_path = event_path.join("raw").join(&output_id);
                match fs::create_dir_all(&raw_path) {
                    Ok(_) => {
                        if let Ok(entries) = fs::read_dir(&temp_dir) {
                            for entry in entries.flatten() {
                                let dest = raw_path.join(entry.file_name());
                                let _ = fs::copy(entry.path(), dest);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create raw HTML directory {:?}: {}", raw_path, e)
                    }
                }
            }

            // Cleanup temp dir
            let _ = fs::remove_dir_all(&temp_dir);
        }
    });

    Ok(saved)
}

/// Drops quoted floor/heat notes (e.g. `"Fläche 3/4"`) so heats of one competition share a folder.
fn strip_floor_note(name: &str) -> String {
    static RE_QUOTED: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r#""[^"]*""#).unwrap());
    RE_QUOTED.replace_all(name, "").trim().to_string()
}

/// Picks `{Age}_{Level}_{Style}_{Date}` so the same competition always maps to the same file,
/// independent of processing order. A second competition with identical class and date (rare)
/// additionally gets its source folder name, which is stable across runs.
fn output_path(
    event_path: &Path,
    comp: &crate::models::Competition,
    comp_url: &str,
) -> (String, PathBuf) {
    #[derive(serde::Deserialize)]
    struct SourceOnly {
        source_url: Option<String>,
    }

    let date = comp
        .date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "undated".to_string());
    let base_id = sanitize_name(&format!(
        "{:?}_{:?}_{:?}_{}",
        comp.age_group, comp.level, comp.style, date
    ));
    let path = event_path.join(format!("{}.json", base_id));
    let free_or_ours = !path.exists()
        || fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<SourceOnly>(&s).ok())
            .and_then(|c| c.source_url)
            .is_some_and(|u| u == comp_url);
    if free_or_ours {
        return (base_id, path);
    }

    let source_dir = url::Url::parse(comp_url)
        .ok()
        .and_then(|u| {
            u.path_segments()
                // Last directory segment, e.g. `61-1005_ot_hgrdlat` of `.../61-1005_ot_hgrdlat/index.htm`
                .and_then(|s| {
                    s.rev()
                        .find(|seg| !seg.is_empty() && !seg.contains('.'))
                        .map(str::to_string)
                })
        })
        .unwrap_or_else(|| comp_url.to_string());
    let id = sanitize_name(&format!("{}_{}", base_id, source_dir));
    log::warn!(
        "{}.json in {:?} belongs to a different source; saving {} as {}.json",
        base_id,
        event_path,
        comp_url,
        id
    );
    let path = event_path.join(format!("{}.json", id));
    (id, path)
}

/// Helper to apply filters and overrides to a competition. Returns false if filtered out.
fn apply_filters_and_overrides(comp: &mut crate::models::Competition, filters: &Filters) -> bool {
    if filters.age_group.is_some_and(|t| comp.age_group != t) {
        return false;
    }
    if filters.style.is_some_and(|t| comp.style != t) {
        return false;
    }
    if filters.level.is_some_and(|t| comp.level != t) {
        return false;
    }

    // Date Filter & Min Dances Override
    if let Some(d) = filters.date {
        if let Some(comp_date) = comp.date {
            if comp_date != d {
                return false;
            }
        } else {
            comp.date = Some(d);
        }
        comp.min_dances = crate::i18n::get_min_dances(comp.level, d);
    } else if let Some(comp_date) = comp.date {
        // Ensure min_dances is correct for the parsed event date
        comp.min_dances = crate::i18n::get_min_dances(comp.level, comp_date);
    }

    true
}

#[pymodule]
fn _dancing_datacollection(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Forward Rust `log` records to Python's `logging` module.
    pyo3_log::init();
    m.add_function(wrap_pyfunction!(load_competition_results, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_floor_note() {
        assert_eq!(
            strip_floor_note(r#"WDSF OPEN Latin Rising Stars "Fläche 3/4""#),
            "WDSF OPEN Latin Rising Stars"
        );
        assert_eq!(
            strip_floor_note(r#"Hgr.II D Standard "1""#),
            "Hgr.II D Standard"
        );
        assert_eq!(strip_floor_note("Hgr. C Standard"), "Hgr. C Standard");
    }

    #[test]
    fn test_output_path_dated_and_collision() {
        use crate::models::{AgeGroup, Competition, Level, Officials, Style};
        let dir = std::env::temp_dir().join(format!("ddc_output_path_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut comp = Competition {
            name: "Hgr. D Latein".into(),
            date: NaiveDate::from_ymd_opt(2026, 5, 10),
            organizer: None,
            hosting_club: None,
            source_url: Some("https://x.de/ht/61-1005_ot_hgrdlat/index.htm".into()),
            level: Level::D,
            age_group: AgeGroup::Adult,
            style: Style::Latein,
            dances: vec![],
            min_dances: 4,
            officials: Officials {
                responsible_person: None,
                assistant: None,
                judges: vec![],
            },
            participants: vec![],
            rounds: vec![],
        };
        let url_a = comp.source_url.clone().unwrap();
        let (id, path) = output_path(&dir, &comp, &url_a);
        assert_eq!(id, "Adult_D_Latein_2026-05-10");
        fs::write(&path, serde_json::to_string(&comp).unwrap()).unwrap();

        // Rerun of the same source maps to the same file.
        assert_eq!(output_path(&dir, &comp, &url_a).0, id);

        // A different source with identical class and date gets its folder name appended.
        let url_b = "https://x.de/ht/99-1005_ot_hgrdlat/index.htm";
        comp.source_url = Some(url_b.into());
        assert_eq!(
            output_path(&dir, &comp, url_b).0,
            "Adult_D_Latein_2026-05-10_99-1005_ot_hgrdlat"
        );

        comp.date = None;
        assert_eq!(output_path(&dir, &comp, url_b).0, "Adult_D_Latein_undated");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_filters_iso_date() {
        let f = parse_filters(Some("2024-05-01"), None, None, None).unwrap();
        assert_eq!(f.date, NaiveDate::from_ymd_opt(2024, 5, 1));
    }

    #[test]
    fn parse_filters_dotted_date() {
        let f = parse_filters(Some("01.05.2024"), None, None, None).unwrap();
        assert_eq!(f.date, NaiveDate::from_ymd_opt(2024, 5, 1));
    }

    #[test]
    fn parse_filters_hgr_age_group() {
        let f = parse_filters(None, Some("Hgr"), None, None).unwrap();
        assert_eq!(f.age_group, Some(AgeGroup::Adult));
    }

    #[test]
    fn parse_filters_invalid_level() {
        let err = parse_filters(None, None, None, Some("Z")).unwrap_err();
        assert!(err.contains("level") && err.contains("'Z'"), "{}", err);
    }

    #[test]
    fn parse_filters_invalid_date() {
        let err = parse_filters(Some("not-a-date"), None, None, None).unwrap_err();
        assert!(err.contains("date"), "{}", err);
    }

    #[test]
    fn parse_filters_none_is_empty() {
        assert_eq!(
            parse_filters(None, None, None, None),
            Ok(Filters::default())
        );
    }
}
