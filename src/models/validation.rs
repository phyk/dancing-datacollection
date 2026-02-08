use std::collections::{BTreeMap, HashSet};
use crate::models::{Level, Event, Round, Dance, Judge, Competition};
use crate::models::skating::{calculate_dance_ranks, calculate_final_ranks, verify_wdsf_score};

fn is_redance(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.contains("redance")
        || name_lower.contains("hoffnung")
        || name_lower.contains("h-lauf")
}

pub fn get_min_dances_for_level(
    level: &Level,
    date: &chrono::NaiveDate,
) -> u32 {
    crate::i18n::get_min_dances(*level, *date)
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
        if !validate_competition_fidelity(comp) {
            return false;
        }
    }
    !event.competitions_list.is_empty()
}

fn validate_competition_fidelity(comp: &Competition) -> bool {
    if comp.officials.judges.len() < 3 || comp.participants.is_empty() || comp.rounds.is_empty() {
        return false;
    }
    if (comp.dances.len() as u32) < comp.min_dances {
        return false;
    }

    let last_round = comp.rounds.last().unwrap();
    if last_round.dtv_ranks.is_none() && last_round.wdsf_scores.is_none() {
        return false;
    }

    let mut round_participant_sets = Vec::new();
    for round in &comp.rounds {
        let mut round_participants = HashSet::new();
        if let Some(ref map) = round.marking_crosses {
            for jm in map.values() { for &b in jm.keys() { round_participants.insert(b); } }
        }
        if let Some(ref map) = round.dtv_ranks {
            for jm in map.values() { for &b in jm.keys() { round_participants.insert(b); } }
        }
        if let Some(ref map) = round.wdsf_scores {
            for jm in map.values() { for &b in jm.keys() { round_participants.insert(b); } }
        }

        let participants_vec: Vec<u32> = round_participants.iter().cloned().collect();
        if !is_round_complete(round, &comp.officials.judges, &participants_vec, &comp.dances) {
            return false;
        }

        if !verify_round_math(round) {
            return false;
        }

        round_participant_sets.push(round_participants);
    }

    let round_0_bibs = &round_participant_sets[0];
    for (i, current_set) in round_participant_sets.iter().enumerate() {
        for bib in current_set {
            if !round_0_bibs.contains(bib) { return false; }
        }
        if i > 0 {
            let prev_set = &round_participant_sets[i - 1];
            let current_is_redance = is_redance(&comp.rounds[i].name);
            let prev_is_redance = is_redance(&comp.rounds[i - 1].name);

            if !current_is_redance && !prev_is_redance {
                if !current_set.is_subset(prev_set) || current_set.len() > prev_set.len() { return false; }
            } else if current_is_redance {
                if !current_set.is_subset(prev_set) { return false; }
            }
        }
    }

    for participant in &comp.participants {
        if let Some(rank) = participant.final_rank {
            for (i, round_set) in round_participant_sets.iter().enumerate() {
                if is_redance(&comp.rounds[i].name) { continue; }
                if rank <= round_set.len() as u32 && !round_set.contains(&participant.bib_number) {
                    return false;
                }
            }
        }
    }

    // Skating Math Verification
    if let Some(last_round) = comp.rounds.last() {
        if let Some(ref dtv_marks) = last_round.dtv_ranks {
            let mut dance_marks = BTreeMap::new();
            for dance in &comp.dances {
                let mut jm_for_dance = BTreeMap::new();
                for (j_code, bib_map) in dtv_marks {
                    let mut marks = BTreeMap::new();
                    for (bib, d_map) in bib_map {
                        if let Some(&m) = d_map.get(dance) { marks.insert(*bib, m); }
                    }
                    jm_for_dance.insert(j_code.clone(), marks);
                }
                dance_marks.insert(*dance, jm_for_dance);
            }

            let mut dance_ranks = BTreeMap::new();
            for (dance, marks) in &dance_marks {
                dance_ranks.insert(*dance, calculate_dance_ranks(marks));
            }

            let final_calc_ranks = calculate_final_ranks(&dance_ranks, Some(&dance_marks));
            for p in &comp.participants {
                if let Some(expected) = p.final_rank {
                    if let Some(&calc) = final_calc_ranks.get(&p.bib_number) {
                        if calc != expected { return false; }
                    }
                }
            }
        }
    }

    true
}

fn verify_round_math(round: &Round) -> bool {
    if let Some(ref wdsf_scores) = round.wdsf_scores {
        for judge_map in wdsf_scores.values() {
            for score in judge_map.values() {
                if !verify_wdsf_score(score) { return false; }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Officials, Participant, IdentityType, Style, AgeGroup};

    fn create_mock_judge(code: &str) -> Judge {
        Judge { code: code.to_string(), name: format!("Judge {}", code), club: None }
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
                judges: vec![create_mock_judge("A"), create_mock_judge("B"), create_mock_judge("C")],
            },
            participants: vec![
                Participant {
                    identity_type: IdentityType::Solo,
                    name_one: "P1".to_string(),
                    bib_number: 101,
                    name_two: None,
                    affiliation: None,
                    final_rank: Some(1),
                },
            ],
            rounds: vec![
                Round {
                    name: "Final".to_string(),
                    marking_crosses: None,
                    dtv_ranks: Some({
                        let mut m = BTreeMap::new();
                        for j in &["A", "B", "C"] {
                            let mut jm = BTreeMap::new();
                            let mut bm = BTreeMap::new();
                            bm.insert(Dance::SlowWaltz, 1);
                            bm.insert(Dance::Tango, 1);
                            jm.insert(101, bm);
                            m.insert(j.to_string(), jm);
                        }
                        m
                    }),
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
            source_url: None,
            competitions_list: vec![create_mock_competition()],
        };
        assert!(validate_event_fidelity(&event));
    }

    #[test]
    fn test_insufficient_judges() {
        let mut comp = create_mock_competition();
        comp.officials.judges.pop();
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_missing_judge_in_round() {
        let mut comp = create_mock_competition();
        if let Some(ref mut ranks) = comp.rounds[0].dtv_ranks { ranks.remove("C"); }
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_missing_participant_for_judge() {
        let mut comp = create_mock_competition();
        if let Some(ref mut ranks) = comp.rounds[0].dtv_ranks {
            let mut bm = BTreeMap::new();
            bm.insert(Dance::SlowWaltz, 1);
            bm.insert(Dance::Tango, 1);
            ranks.get_mut("A").unwrap().insert(102, bm);
        }
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_missing_dance_for_participant() {
        let mut comp = create_mock_competition();
        if let Some(ref mut ranks) = comp.rounds[0].dtv_ranks {
            ranks.get_mut("A").unwrap().get_mut(&101).unwrap().remove(&Dance::Tango);
        }
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_wdsf_scores_completeness() {
        let mut comp = create_mock_competition();
        comp.rounds[0].marking_crosses = None;
        comp.rounds[0].wdsf_scores = {
            let mut m = BTreeMap::new();
            for j in &["A", "B", "C"] {
                let mut jm = BTreeMap::new();
                jm.insert(101, crate::models::WDSFScore {
                    technical_quality: 10.0,
                    movement_to_music: 10.0,
                    partnering_skills: 10.0,
                    choreography: 10.0,
                    total: 10.0,
                });
                m.insert(j.to_string(), jm);
            }
            Some(m)
        };
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(validate_event_fidelity(&event));
    }

    #[test]
    fn test_dtv_ranks_completeness() {
        let mut comp = create_mock_competition();
        comp.rounds[0].marking_crosses = None;
        comp.rounds[0].dtv_ranks = {
            let mut m = BTreeMap::new();
            for j in &["A", "B", "C"] {
                let mut jm = BTreeMap::new();
                let mut bm = BTreeMap::new();
                bm.insert(Dance::SlowWaltz, 1);
                bm.insert(Dance::Tango, 2);
                jm.insert(101, bm);
                m.insert(j.to_string(), jm);
            }
            Some(m)
        };
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(validate_event_fidelity(&event));
    }

    #[test]
    fn test_teleporting_couple() {
        let mut comp = create_mock_competition();
        comp.rounds.insert(0, Round {
            name: "Vorrunde".to_string(),
            marking_crosses: Some({
                let mut m = BTreeMap::new();
                for j in &["A", "B", "C"] {
                    let mut jm = BTreeMap::new();
                    let mut bm = BTreeMap::new();
                    bm.insert(Dance::SlowWaltz, true);
                    bm.insert(Dance::Tango, true);
                    jm.insert(101, bm);
                    m.insert(j.to_string(), jm);
                }
                m
            }),
            dtv_ranks: None,
            wdsf_scores: None,
        });
        if let Some(ref mut ranks) = comp.rounds[1].dtv_ranks {
            let mut bm = BTreeMap::new();
            bm.insert(Dance::SlowWaltz, 1);
            bm.insert(Dance::Tango, 1);
            ranks.get_mut("A").unwrap().insert(102, bm.clone());
            ranks.get_mut("B").unwrap().insert(102, bm.clone());
            ranks.get_mut("C").unwrap().insert(102, bm);
        }
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_skipping_round() {
        let mut comp = create_mock_competition();
        comp.rounds.insert(0, Round {
            name: "Vorrunde".to_string(),
            marking_crosses: Some({
                let mut m = BTreeMap::new();
                for j in &["A", "B", "C"] {
                    let mut jm = BTreeMap::new();
                    let mut bm = BTreeMap::new();
                    bm.insert(Dance::SlowWaltz, true);
                    bm.insert(Dance::Tango, true);
                    jm.insert(101, bm.clone());
                    jm.insert(102, bm.clone());
                    m.insert(j.to_string(), jm);
                }
                m
            }),
            dtv_ranks: None,
            wdsf_scores: None,
        });
        comp.rounds.insert(1, Round {
            name: "Semi".to_string(),
            marking_crosses: Some({
                let mut m = BTreeMap::new();
                for j in &["A", "B", "C"] {
                    let mut jm = BTreeMap::new();
                    let mut bm = BTreeMap::new();
                    bm.insert(Dance::SlowWaltz, true);
                    bm.insert(Dance::Tango, true);
                    jm.insert(102, bm);
                    m.insert(j.to_string(), jm);
                }
                m
            }),
            dtv_ranks: None,
            wdsf_scores: None,
        });
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_inconsistent_rank() {
        let mut comp = create_mock_competition();
        comp.participants[0].final_rank = Some(1);
        if let Some(ref mut ranks) = comp.rounds[0].dtv_ranks {
            for jm in ranks.values_mut() {
                jm.remove(&101);
                let mut bm = BTreeMap::new();
                bm.insert(Dance::SlowWaltz, 1);
                bm.insert(Dance::Tango, 1);
                jm.insert(102, bm);
            }
        }
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(!validate_event_fidelity(&event));
    }

    #[test]
    fn test_missing_final_anchor() {
        let mut comp = create_mock_competition();
        comp.rounds[0].dtv_ranks = None;
        comp.rounds[0].marking_crosses = Some({
            let mut m = BTreeMap::new();
            for j in &["A", "B", "C"] {
                let mut jm = BTreeMap::new();
                let mut bm = BTreeMap::new();
                bm.insert(Dance::SlowWaltz, true);
                bm.insert(Dance::Tango, true);
                jm.insert(101, bm);
                m.insert(j.to_string(), jm);
            }
            m
        });
        let event = Event { name: "Test Event".to_string(), date: None, organizer: None, hosting_club: None, source_url: None, competitions_list: vec![comp] };
        assert!(!validate_event_fidelity(&event));
    }
}
