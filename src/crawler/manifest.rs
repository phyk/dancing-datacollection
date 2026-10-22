use crate::models::Competition;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct Manifest {
    pub processed_ids: HashSet<String>,
}

impl Manifest {
    pub fn from_target_folder(target_folder: &Path) -> Self {
        let mut processed_ids = HashSet::new();
        if !target_folder.exists() {
            return Self { processed_ids };
        }

        let Ok(entries) = fs::read_dir(target_folder) else {
            return Self { processed_ids };
        };

        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            // Leftover scratch directories from (interrupted) downloads.
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("tmp_download_")
            {
                continue;
            }
            let Ok(sub_entries) = fs::read_dir(&dir) else {
                continue;
            };

            let urls = sub_entries
                .flatten()
                .map(|sub_entry| sub_entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
                .filter_map(|path| fs::read_to_string(path).ok())
                .filter_map(|content| serde_json::from_str::<Competition>(&content).ok())
                .filter_map(|event| event.source_url);
            processed_ids.extend(urls);
        }

        Self { processed_ids }
    }

    pub fn is_processed(&self, id: &str) -> bool {
        self.processed_ids.contains(id)
    }

    pub fn mark_processed(&mut self, id: String) {
        self.processed_ids.insert(id);
    }
}
