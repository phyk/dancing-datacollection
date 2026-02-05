use crate::PyEvent;
use pyo3::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Manages the persistence of competition data in multiple formats.
#[pyclass]
pub struct StorageManager {
    base_path: PathBuf,
}

#[pymethods]
impl StorageManager {
    /// Creates a new StorageManager with the given base directory.
    #[new]
    pub fn new(base_path: String) -> Self {
        let path = PathBuf::from(base_path);
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        Self { base_path: path }
    }

    /// Saves an Event to disk in both JSONL and Postcard binary formats.
    pub fn save_event(&self, py_event: &PyEvent) -> PyResult<()> {
        let event = &py_event.0;
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

        // Manifest Update
        self.update_manifest(&event.name, &event_dir.to_string_lossy())?;

        Ok(())
    }
}

impl StorageManager {
    fn sanitize_name(&self, name: &str) -> String {
        let mut s: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        s.truncate(64);
        s
    }

    fn update_manifest(&self, event_name: &str, event_path: &str) -> PyResult<()> {
        let manifest_path = self.base_path.join("manifest.json");
        let mut manifest: serde_json::Value = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!("Failed to read manifest: {}", e))
            })?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        if let Some(obj) = manifest.as_object_mut() {
            obj.insert(event_name.to_string(), serde_json::json!(event_path));
        }

        let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to serialize manifest: {}", e))
        })?;
        fs::write(manifest_path, manifest_json).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!("Failed to write manifest file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    #[test]
    fn test_serialization_roundtrip() {
        let event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: Some("Organizer".to_string()),
            hosting_club: None,
            competitions_list: vec![Competition {
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
            }],
        };

        let base_dir = "test_storage_roundtrip";
        let manager = StorageManager::new(base_dir.to_string());
        let py_event = PyEvent(event.clone());
        manager.save_event(&py_event).expect("Save failed");

        let sanitized_name = manager.sanitize_name(&event.name);
        let event_dir = PathBuf::from(base_dir).join(sanitized_name);

        // Verify JSONL
        let json_path = event_dir.join("event.jsonl");
        let json_content = fs::read_to_string(&json_path).expect("Read JSON failed");
        let deserialized_json: Event = serde_json::from_str(json_content.trim()).expect("Deserialize JSON failed");
        assert_eq!(deserialized_json, event);

        // Verify Postcard
        let bin_path = event_dir.join("event.bin");
        let bin_content = fs::read(bin_path).expect("Read Bin failed");
        let deserialized_bin: Event = postcard::from_bytes(&bin_content).expect("Deserialize Bin failed");
        assert_eq!(deserialized_bin, event);

        // Verify Manifest
        let manifest_path = PathBuf::from(base_dir).join("manifest.json");
        assert!(manifest_path.exists());
        let manifest_content = fs::read_to_string(&manifest_path).unwrap();
        assert!(manifest_content.contains("Test Event"));

        // Cleanup
        let _ = fs::remove_dir_all(base_dir);
    }
}
