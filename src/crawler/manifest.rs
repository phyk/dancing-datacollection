use std::collections::HashSet;
use std::fs;
use std::path::Path;
use crate::models::Competition;

#[derive(Default)]
pub struct Manifest {
    pub processed_ids: HashSet<String>,
}

impl Manifest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_target_folder(target_folder: &Path) -> Self {
        let mut processed_ids = HashSet::new();
        if !target_folder.exists() {
            return Self { processed_ids };
        }

        if let Ok(entries) = fs::read_dir(target_folder) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                        for sub_entry in sub_entries.flatten() {
                            if sub_entry.path().extension().map_or(false, |ext| ext == "json") {
                                if let Ok(content) = fs::read_to_string(sub_entry.path()) {
                                    if let Ok(event) = serde_json::from_str::<Competition>(&content) {
                                        if let Some(url) = event.source_url {
                                            processed_ids.insert(url);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
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
