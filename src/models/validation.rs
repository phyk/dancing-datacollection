use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::{Level, Event};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LevelConfig {
    pub min_dances: Option<u32>,
    pub min_dances_legacy: Option<u32>,
    pub min_dances_2026: Option<u32>,
}

impl LevelConfig {
    pub fn get_min_dances(&self, date: &chrono::NaiveDate) -> u32 {
        use chrono::Datelike;
        if let Some(min) = self.min_dances {
            return min;
        }
        let is_2026_or_later = date.year() >= 2026;
        if is_2026_or_later {
            self.min_dances_2026
                .or(self.min_dances_legacy)
                .unwrap_or(0)
        } else {
            self.min_dances_legacy.unwrap_or(0)
        }
    }
}

pub fn get_min_dances_for_level(
    levels: &Option<HashMap<String, LevelConfig>>,
    level: &Level,
    date: &chrono::NaiveDate,
) -> u32 {
    let level_str = format!("{:?}", level);
    if let Some(levels) = levels {
        if let Some(config) = levels.get(&level_str) {
            return config.get_min_dances(date);
        }
    }
    0
}

/// Checks whether the competitions extracted reproduce the downloaded sources (Fidelity Gate).
pub fn validate_event_fidelity(event: &Event) -> bool {
    for comp in &event.competitions_list {
        // Fidelity Gate: A competition is invalid if it lacks Officials, Judges, or Results.
        if comp.officials.judges.is_empty() {
             return false;
        }
        if comp.participants.is_empty() {
            return false;
        }
        if comp.rounds.is_empty() {
            return false;
        }
    }
    !event.competitions_list.is_empty()
}
