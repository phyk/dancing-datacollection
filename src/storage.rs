use crate::models::Competition;
use std::fs;
use std::path::PathBuf;

/// Manages the persistence of competition data in multiple formats.
pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    /// Creates a new StorageManager with the given base directory.
    pub fn new(base_path: String) -> Self {
        let path = PathBuf::from(base_path);
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        Self { base_path: path }
    }

    /// Saves a Competition to disk in both JSONL and Postcard binary formats.
    pub fn save_event(&self, event: &Competition) -> anyhow::Result<()> {
        let sanitized_name = self.sanitize_name(&event.name);
        let event_dir = self.base_path.join(sanitized_name);
        fs::create_dir_all(&event_dir).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to create event directory: {}", e))
        })?;

        // JSONL Support: Each event must be a single line of JSON.
        let json_path = event_dir.join("event.jsonl");
        let json_data = serde_json::to_string(event).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to serialize to JSON: {}", e))
        })?;
        fs::write(json_path, format!("{}\n", json_data)).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to write JSONL file: {}", e))
        })?;

        // Postcard Support: Use the postcard crate for a dense binary export.
        let bin_path = event_dir.join("event.bin");
        let bin_data = postcard::to_stdvec(event).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to serialize to Postcard: {}", e))
        })?;
        fs::write(bin_path, bin_data).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to write binary file: {}", e))
        })?;

        Ok(())
    }
}

impl StorageManager {
    fn sanitize_name(&self, name: &str) -> String {
        crate::models::sanitize_name(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    #[test]
    fn test_serialization_roundtrip() {
        let event = Competition {
            name: "Test Event".to_string(),
            date: None,
            organizer: Some("Organizer".to_string()),
            hosting_club: None,
            source_url: None,
            level: Level::S,
            age_group: AgeGroup::Adult,
            style: Style::Standard,
            dances: vec![Dance::SlowWaltz, Dance::Tango],
            min_dances: 2,
            officials: Officials {
                responsible_person: None,
                assistant: None,
                judges: vec![Judge {
                    code: "A".to_string(),
                    name: "Judge A".to_string(),
                    club: None,
                }],
            },
            participants: vec![Participant {
                identity_type: IdentityType::Couple,
                name_one: "Dancer One".to_string(),
                bib_number: 101,
                name_two: Some("Dancer Two".to_string()),
                affiliation: None,
                final_rank: Some(1),
            }],
            rounds: vec![],
        };

        let base_dir = "test_storage_roundtrip";
        let manager = StorageManager::new(base_dir.to_string());
        manager.save_event(&event).expect("Save failed");

        let sanitized_name = manager.sanitize_name(&event.name);
        let event_dir = PathBuf::from(base_dir).join(sanitized_name);

        // Verify JSONL
        let json_path = event_dir.join("event.jsonl");
        let json_content = fs::read_to_string(&json_path).expect("Read JSON failed");
        let deserialized_json: Competition =
            serde_json::from_str(json_content.trim()).expect("Deserialize JSON failed");
        assert_eq!(deserialized_json, event);

        // Verify Postcard
        let bin_path = event_dir.join("event.bin");
        let bin_content = fs::read(bin_path).expect("Read Bin failed");
        let deserialized_bin: Competition =
            postcard::from_bytes(&bin_content).expect("Deserialize Bin failed");
        assert_eq!(deserialized_bin, event);

        // Cleanup
        let _ = fs::remove_dir_all(base_dir);
    }
}
