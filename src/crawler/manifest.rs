use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Default)]
pub struct Manifest {
    pub processed_ids: HashSet<String>,
}

impl Manifest {
    pub fn load(path: &Path) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_processed(&self, id: &str) -> bool {
        self.processed_ids.contains(id)
    }

    pub fn mark_processed(&mut self, id: String) {
        self.processed_ids.insert(id);
    }
}
