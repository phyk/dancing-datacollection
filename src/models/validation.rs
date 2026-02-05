use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::models::{Level, Event, Round, Dance, Judge};

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

/// Helper method to check if all expected data is present in a round.
fn is_round_complete(
    round: &Round,
    expected_judges: &[Judge],
    participants: &[u32],
    dances: &[Dance],
) -> bool {
    if participants.is_empty() {
        return false;
    }

    let mut has_any_data = false;

    // Check marking crosses
    if let Some(ref crosses) = round.marking_crosses {
        has_any_data = true;
        for judge in expected_judges {
            let judge_map = match crosses.get(&judge.code) {
                Some(m) => m,
                None => return false,
            };
            for &bib in participants {
                let bib_map = match judge_map.get(&bib) {
                    Some(m) => m,
                    None => return false,
                };
                for dance in dances {
                    if !bib_map.contains_key(dance) {
                        return false;
                    }
                }
            }
        }
    }

    // Check DTV ranks
    if let Some(ref ranks) = round.dtv_ranks {
        has_any_data = true;
        for judge in expected_judges {
            let judge_map = match ranks.get(&judge.code) {
                Some(m) => m,
                None => return false,
            };
            for &bib in participants {
                let bib_map = match judge_map.get(&bib) {
                    Some(m) => m,
                    None => return false,
                };
                for dance in dances {
                    if !bib_map.contains_key(dance) {
                        return false;
                    }
                }
            }
        }
    }

    // Check WDSF scores
    if let Some(ref wdsf) = round.wdsf_scores {
        has_any_data = true;
        for judge in expected_judges {
            let judge_map = match wdsf.get(&judge.code) {
                Some(m) => m,
                None => return false,
            };
            for &bib in participants {
                if !judge_map.contains_key(&bib) {
                    return false;
                }
            }
        }
    }

    has_any_data
}

/// Checks whether the competitions extracted reproduce the downloaded sources (Fidelity Gate).
pub fn validate_event_fidelity(event: &Event) -> bool {
    for comp in &event.competitions_list {
        // Fidelity Gate: A competition is invalid if it lacks Officials, Judges, or Results.
        // Integrity Layer: Must have at least 3 judges.
        if comp.officials.judges.len() < 3 {
             return false;
        }
        if comp.participants.is_empty() {
            return false;
        }
        if comp.rounds.is_empty() {
            return false;
        }
        // Verify that the number of dances parsed matches the level's minimum requirement.
        if (comp.dances.len() as u32) < comp.min_dances {
            return false;
        }

        // Completeness Audit: For every Round in comp.rounds
        for round in &comp.rounds {
            let mut round_participants = HashSet::new();

            if let Some(ref map) = round.marking_crosses {
                for judge_map in map.values() {
                    for &bib in judge_map.keys() {
                        round_participants.insert(bib);
                    }
                }
            }
            if let Some(ref map) = round.dtv_ranks {
                for judge_map in map.values() {
                    for &bib in judge_map.keys() {
                        round_participants.insert(bib);
                    }
                }
            }
            if let Some(ref map) = round.wdsf_scores {
                for judge_map in map.values() {
                    for &bib in judge_map.keys() {
                        round_participants.insert(bib);
                    }
                }
            }

            let participants_vec: Vec<u32> = round_participants.into_iter().collect();
            if !is_round_complete(round, &comp.officials.judges, &participants_vec, &comp.dances) {
                return false;
            }
        }
    }
    !event.competitions_list.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Competition, Officials, Participant, IdentityType, Style, AgeGroup};

    fn create_mock_judge(code: &str) -> Judge {
        Judge {
            code: code.to_string(),
            name: format!("Judge {}", code),
            club: None,
        }
    }

    fn create_mock_competition() -> Competition {
        Competition {
            level: Level::D,
            age_group: AgeGroup::Adult,
            style: Style::Standard,
            dances: vec![Dance::SlowWaltz, Dance::Tango],
            min_dances: 2,
            officials: Officials {
                responsible_person: None,
                assistant: None,
                judges: vec![
                    create_mock_judge("A"),
                    create_mock_judge("B"),
                    create_mock_judge("C"),
                ],
            },
            participants: vec![
                Participant {
                    identity_type: IdentityType::Solo,
                    name_one: "P1".to_string(),
                    bib_number: 101,
                    name_two: None,
                    affiliation: None,
                    final_rank: None,
                },
            ],
            rounds: vec![
                Round {
                    name: "Final".to_string(),
                    marking_crosses: {
                        let mut m = HashMap::new();
                        for j in &["A", "B", "C"] {
                            let mut jm = HashMap::new();
                            let mut bm = HashMap::new();
                            bm.insert(Dance::SlowWaltz, true);
                            bm.insert(Dance::Tango, true);
                            jm.insert(101, bm);
                            m.insert(j.to_string(), jm);
                        }
                        Some(m)
                    },
                    dtv_ranks: None,
                    wdsf_scores: None,
                }
            ],
        }
    }

    #[test]
    fn test_valid_event() {
        let event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![create_mock_competition()],
        };
        assert!(validate_event_fidelity(&event));
    }

    #[test]
    fn test_insufficient_judges() {
        let mut comp = create_mock_competition();
        comp.officials.judges.pop(); // Down to 2
        let event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![comp],
        };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_missing_judge_in_round() {
        let mut comp = create_mock_competition();
        if let Some(ref mut crosses) = comp.rounds[0].marking_crosses {
            crosses.remove("C");
        }
        let event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![comp],
        };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_missing_participant_for_judge() {
        let mut comp = create_mock_competition();
        // Add another participant to the round union
        if let Some(ref mut crosses) = comp.rounds[0].marking_crosses {
            let mut bm = HashMap::new();
            bm.insert(Dance::SlowWaltz, true);
            bm.insert(Dance::Tango, true);
            crosses.get_mut("A").unwrap().insert(102, bm);
        }
        // Now Judge B and C are missing participant 102
        let event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![comp],
        };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_missing_dance_for_participant() {
        let mut comp = create_mock_competition();
        if let Some(ref mut crosses) = comp.rounds[0].marking_crosses {
            crosses.get_mut("A").unwrap().get_mut(&101).unwrap().remove(&Dance::Tango);
        }
        let event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![comp],
        };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_wdsf_scores_completeness() {
        let mut comp = create_mock_competition();
        comp.rounds[0].marking_crosses = None;
        comp.rounds[0].wdsf_scores = {
            let mut m = HashMap::new();
            for j in &["A", "B", "C"] {
                let mut jm = HashMap::new();
                jm.insert(101, crate::models::WDSFScore {
                    technical_quality: 10.0,
                    movement_to_music: 10.0,
                    partnering_skills: 10.0,
                    choreography: 10.0,
                });
                m.insert(j.to_string(), jm);
            }
            Some(m)
        };

        let event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![comp.clone()],
        };
        assert!(validate_event_fidelity(&event));

        // Corrupt it
        let mut corrupt_comp = comp.clone();
        if let Some(ref mut wdsf) = corrupt_comp.rounds[0].wdsf_scores {
            wdsf.get_mut("A").unwrap().remove(&101);
        }
        let corrupt_event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![corrupt_comp],
        };
        assert!(!validate_event_fidelity(&corrupt_event));
    }

    #[test]
    fn test_dtv_ranks_completeness() {
        let mut comp = create_mock_competition();
        comp.rounds[0].marking_crosses = None;
        comp.rounds[0].dtv_ranks = {
            let mut m = HashMap::new();
            for j in &["A", "B", "C"] {
                let mut jm = HashMap::new();
                let mut bm = HashMap::new();
                bm.insert(Dance::SlowWaltz, 1);
                bm.insert(Dance::Tango, 2);
                jm.insert(101, bm);
                m.insert(j.to_string(), jm);
            }
            Some(m)
        };

        let event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![comp.clone()],
        };
        assert!(validate_event_fidelity(&event));

        // Corrupt it - missing dance
        let mut corrupt_comp = comp.clone();
        if let Some(ref mut ranks) = corrupt_comp.rounds[0].dtv_ranks {
            ranks.get_mut("A").unwrap().get_mut(&101).unwrap().remove(&Dance::Tango);
        }
        let corrupt_event = Event {
            name: "Test Event".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            competitions_list: vec![corrupt_comp],
        };
        assert!(!validate_event_fidelity(&corrupt_event));
    }
}
