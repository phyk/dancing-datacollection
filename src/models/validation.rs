use std::collections::{BTreeMap, HashSet};
use crate::models::{Round, RoundData, Dance, Judge, Competition};
use crate::models::skating::{calculate_dance_ranks, calculate_final_ranks, verify_wdsf_score};


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

    for judge in expected_judges {
        for &bib in participants {
            let bib_str = bib.to_string();
            let data_present = match &round.data {
                RoundData::Marking { marking_crosses } => {
                    marking_crosses.get(&judge.code)
                        .and_then(|jm| jm.get(&bib_str))
                        .map(|bm| dances.iter().all(|d| bm.contains_key(d)))
                        .unwrap_or(false)
                }
                RoundData::DTV { dtv_ranks } => {
                    dtv_ranks.get(&judge.code)
                        .and_then(|jm| jm.get(&bib_str))
                        .map(|bm| dances.iter().all(|d| bm.contains_key(d)))
                        .unwrap_or(false)
                }
                RoundData::WDSF { wdsf_scores } => {
                    wdsf_scores.get(&judge.code)
                        .map(|jm| jm.contains_key(&bib_str))
                        .unwrap_or(false)
                }
            };
            if !data_present {
                return false;
            }
        }
    }
    true
}

/// Checks whether the competition extracted reproduces the downloaded sources (Fidelity Gate).
pub fn validate_competition_fidelity(comp: &Competition) -> bool {
    if comp.officials.judges.len() < 3 || comp.participants.is_empty() || comp.rounds.is_empty() {
        return false;
    }
    if (comp.dances.len() as u32) < comp.min_dances {
        return false;
    }

    let last_round = comp.rounds.last().unwrap();
    match &last_round.data {
        RoundData::Marking { .. } => return false, // Last round must be a scoring round
        _ => {}
    }

    let mut round_participant_sets = Vec::new();
    for round in &comp.rounds {
        let mut round_participants = HashSet::new();
        match &round.data {
            RoundData::Marking { marking_crosses } => {
                for jm in marking_crosses.values() {
                    for b in jm.keys() {
                        round_participants.insert(b.parse::<u32>().unwrap_or(0));
                    }
                }
            }
            RoundData::DTV { dtv_ranks } => {
                for jm in dtv_ranks.values() {
                    for b in jm.keys() {
                        round_participants.insert(b.parse::<u32>().unwrap_or(0));
                    }
                }
            }
            RoundData::WDSF { wdsf_scores } => {
                for jm in wdsf_scores.values() {
                    for b in jm.keys() {
                        round_participants.insert(b.parse::<u32>().unwrap_or(0));
                    }
                }
            }
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
            let current_is_redance = crate::i18n::is_redance(&comp.rounds[i].name);
            let prev_is_redance = crate::i18n::is_redance(&comp.rounds[i - 1].name);

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
                if crate::i18n::is_redance(&comp.rounds[i].name) {
                    continue;
                }
                if rank <= round_set.len() as u32 && !round_set.contains(&participant.bib_number) {
                    return false;
                }
            }
        }
    }

    // Skating Math Verification
    if let Some(last_round) = comp.rounds.last() {
        if let RoundData::DTV { dtv_ranks } = &last_round.data {
            let dtv_marks = dtv_ranks;
            let mut dance_marks = BTreeMap::new();
            for dance in &comp.dances {
                let mut jm_for_dance = BTreeMap::new();
                for (j_code, bib_map) in dtv_marks {
                    let mut marks = BTreeMap::new();
                    for (bib_str, d_map) in bib_map {
                        if let Ok(bib) = bib_str.parse::<u32>() {
                            if let Some(&m) = d_map.get(dance) {
                                marks.insert(bib, m);
                            }
                        }
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
                        if calc != expected {
                            return false;
                        }
                    }
                }
            }
        }
    }

    true
}

fn verify_round_math(round: &Round) -> bool {
    if let RoundData::WDSF { wdsf_scores } = &round.data {
        for judge_map in wdsf_scores.values() {
            for score in judge_map.values() {
                if !verify_wdsf_score(score) {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Officials, Participant, IdentityType, Style, AgeGroup, Level};

    fn create_mock_judge(code: &str) -> Judge {
        Judge { code: code.to_string(), name: format!("Judge {}", code), club: None }
    }

    fn create_mock_competition() -> Competition {
        Competition {
            name: "Test Comp".to_string(),
            date: None,
            organizer: None,
            hosting_club: None,
            source_url: None,
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
            participants: vec![Participant {
                identity_type: IdentityType::Solo,
                name_one: "P1".to_string(),
                bib_number: 101,
                name_two: None,
                affiliation: None,
                final_rank: Some(1),
            }],
            rounds: vec![Round {
                name: "Final".to_string(),
                order: 1,
                dances: vec![Dance::SlowWaltz, Dance::Tango],
                data: RoundData::DTV {
                    dtv_ranks: {
                        let mut m = BTreeMap::new();
                        for j in &["A", "B", "C"] {
                            let mut jm = BTreeMap::new();
                            let mut bm = BTreeMap::new();
                            bm.insert(Dance::SlowWaltz, 1);
                            bm.insert(Dance::Tango, 1);
                            jm.insert(101.to_string(), bm);
                            m.insert(j.to_string(), jm);
                        }
                        m
                    },
                },
            }],
        }
    }

    #[test]
    fn test_valid_competition() {
        let comp = create_mock_competition();
        assert!(validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_insufficient_judges() {
        let mut comp = create_mock_competition();
        comp.officials.judges.pop();
        assert!(!validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_missing_judge_in_round() {
        let mut comp = create_mock_competition();
        if let RoundData::DTV { ref mut dtv_ranks } = comp.rounds[0].data {
            dtv_ranks.remove("C");
        }
        assert!(!validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_missing_participant_for_judge() {
        let mut comp = create_mock_competition();
        if let RoundData::DTV { ref mut dtv_ranks } = comp.rounds[0].data {
            let mut bm = BTreeMap::new();
            bm.insert(Dance::SlowWaltz, 1);
            bm.insert(Dance::Tango, 1);
            dtv_ranks.get_mut("A").unwrap().insert(102.to_string(), bm);
        }
        assert!(!validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_missing_dance_for_participant() {
        let mut comp = create_mock_competition();
        if let RoundData::DTV { ref mut dtv_ranks } = comp.rounds[0].data {
            dtv_ranks
                .get_mut("A")
                .unwrap()
            .get_mut("101")
                .unwrap()
                .remove(&Dance::Tango);
        }
        assert!(!validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_wdsf_scores_completeness() {
        let mut comp = create_mock_competition();
        comp.rounds[0] = Round {
            name: "Final".to_string(),
            order: 1,
            dances: vec![Dance::SlowWaltz, Dance::Tango],
            data: RoundData::WDSF {
                wdsf_scores: {
                    let mut m = BTreeMap::new();
                    for j in &["A", "B", "C"] {
                        let mut jm = BTreeMap::new();
                        jm.insert(
                            101.to_string(),
                            crate::models::WDSFScore {
                                technical_quality: 10.0,
                                movement_to_music: 10.0,
                                partnering_skills: 10.0,
                                choreography: 10.0,
                                total: 10.0,
                            },
                        );
                        m.insert(j.to_string(), jm);
                    }
                    m
                },
            },
        };
        assert!(validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_dtv_ranks_completeness() {
        let mut comp = create_mock_competition();
        comp.rounds[0] = Round {
            name: "Final".to_string(),
            order: 1,
            dances: vec![Dance::SlowWaltz, Dance::Tango],
            data: RoundData::DTV {
                dtv_ranks: {
                    let mut m = BTreeMap::new();
                    for j in &["A", "B", "C"] {
                        let mut jm = BTreeMap::new();
                        let mut bm = BTreeMap::new();
                        bm.insert(Dance::SlowWaltz, 1);
                        bm.insert(Dance::Tango, 2);
                        jm.insert(101.to_string(), bm);
                        m.insert(j.to_string(), jm);
                    }
                    m
                },
            },
        };
        assert!(validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_teleporting_couple() {
        let mut comp = create_mock_competition();
        comp.rounds.insert(
            0,
            Round {
                name: "Vorrunde".to_string(),
                order: 0,
                dances: vec![Dance::SlowWaltz, Dance::Tango],
                data: RoundData::Marking {
                    marking_crosses: {
                        let mut m = BTreeMap::new();
                        for j in &["A", "B", "C"] {
                            let mut jm = BTreeMap::new();
                            let mut bm = BTreeMap::new();
                            bm.insert(Dance::SlowWaltz, true);
                            bm.insert(Dance::Tango, true);
                            jm.insert(101.to_string(), bm);
                            m.insert(j.to_string(), jm);
                        }
                        m
                    },
                },
            },
        );
        if let RoundData::DTV { ref mut dtv_ranks } = comp.rounds[1].data {
            let mut bm = BTreeMap::new();
            bm.insert(Dance::SlowWaltz, 1);
            bm.insert(Dance::Tango, 1);
            dtv_ranks.get_mut("A").unwrap().insert(102.to_string(), bm.clone());
            dtv_ranks.get_mut("B").unwrap().insert(102.to_string(), bm.clone());
            dtv_ranks.get_mut("C").unwrap().insert(102.to_string(), bm);
        }
        assert!(!validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_skipping_round() {
        let mut comp = create_mock_competition();
        comp.rounds.insert(
            0,
            Round {
                name: "Vorrunde".to_string(),
                order: 0,
                dances: vec![Dance::SlowWaltz, Dance::Tango],
                data: RoundData::Marking {
                    marking_crosses: {
                        let mut m = BTreeMap::new();
                        for j in &["A", "B", "C"] {
                            let mut jm = BTreeMap::new();
                            let mut bm = BTreeMap::new();
                            bm.insert(Dance::SlowWaltz, true);
                            bm.insert(Dance::Tango, true);
                            jm.insert(101.to_string(), bm.clone());
                            jm.insert(102.to_string(), bm.clone());
                            m.insert(j.to_string(), jm);
                        }
                        m
                    },
                },
            },
        );
        comp.rounds.insert(
            1,
            Round {
                name: "Semi".to_string(),
                order: 1,
                dances: vec![Dance::SlowWaltz, Dance::Tango],
                data: RoundData::Marking {
                    marking_crosses: {
                        let mut m = BTreeMap::new();
                        for j in &["A", "B", "C"] {
                            let mut jm = BTreeMap::new();
                            let mut bm = BTreeMap::new();
                            bm.insert(Dance::SlowWaltz, true);
                            bm.insert(Dance::Tango, true);
                            jm.insert(102.to_string(), bm);
                            m.insert(j.to_string(), jm);
                        }
                        m
                    },
                },
            },
        );
        assert!(!validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_inconsistent_rank() {
        let mut comp = create_mock_competition();
        comp.participants[0].final_rank = Some(1);
        if let RoundData::DTV { ref mut dtv_ranks } = comp.rounds[0].data {
            for jm in dtv_ranks.values_mut() {
                jm.remove("101");
                let mut bm = BTreeMap::new();
                bm.insert(Dance::SlowWaltz, 1);
                bm.insert(Dance::Tango, 1);
                jm.insert(102.to_string(), bm);
            }
        }
        assert!(!validate_competition_fidelity(&comp));
    }

    #[test]
    fn test_missing_final_anchor() {
        let mut comp = create_mock_competition();
        comp.rounds[0] = Round {
            name: "Final".to_string(),
            order: 1,
            dances: vec![Dance::SlowWaltz, Dance::Tango],
            data: RoundData::Marking {
                marking_crosses: {
                    let mut m = BTreeMap::new();
                    for j in &["A", "B", "C"] {
                        let mut jm = BTreeMap::new();
                        let mut bm = BTreeMap::new();
                        bm.insert(Dance::SlowWaltz, true);
                        bm.insert(Dance::Tango, true);
                        jm.insert(101.to_string(), bm);
                        m.insert(j.to_string(), jm);
                    }
                    m
                },
            },
        };
        assert!(!validate_competition_fidelity(&comp));
    }
}
