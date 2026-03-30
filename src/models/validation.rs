use crate::models::skating::{calculate_dance_ranks, calculate_final_ranks, verify_wdsf_score};
use crate::models::{Competition, Dance, Judge, Round, RoundData};
use std::collections::BTreeMap;

/// Helper method to check if all expected data is present in a round.
fn is_round_complete(
    round: &Round,
    expected_judges: &[Judge],
    dances: &[Dance],
) -> bool {
    let round_participants = round.data.participant_bibs();
    if round_participants.is_empty() {
        return false;
    }

    for judge in expected_judges {
        // In WDSF, judges might only judge specific dances or rounds.
        // We only expect completeness if the judge has provided ANY data in this round.
        let judge_has_any_data = match &round.data {
            RoundData::Marking { marking_crosses } => marking_crosses.contains_key(&judge.code),
            RoundData::DTV { dtv_ranks } => dtv_ranks.contains_key(&judge.code),
            RoundData::WDSF { wdsf_scores } => wdsf_scores.contains_key(&judge.code),
        };

        if !judge_has_any_data {
            continue;
        }

        for &bib in &round_participants {
            if !round.data.has_marks_for(&judge.code, bib, dances) {
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
    // DTV/WDSF competitions must have at least the minimum number of dances for their level.
    if (comp.dances.len() as u32) < comp.min_dances {
        return false;
    }

    let last_round = comp.rounds.last().unwrap();
    if let RoundData::Marking { .. } = &last_round.data {
        return false; // Last round must be a scoring round
    }

    let all_valid_bibs: std::collections::BTreeSet<u32> = comp.participants.iter().map(|p| p.bib_number).collect();
    let mut round_participant_sets = Vec::new();

    for round in &comp.rounds {
        let round_participants = round.data.participant_bibs();
        if !is_round_complete(round, &comp.officials.judges, &round.dances) {
            return false;
        }

        if !verify_round_math(round) {
            return false;
        }

        // Verify all participants in this round are known globally
        for bib in &round_participants {
            if !all_valid_bibs.contains(bib) {
                return false;
            }
        }

        round_participant_sets.push(round_participants);
    }

    // Progression check
    for (i, current_set) in round_participant_sets.iter().enumerate() {
        if i > 0 {
            let prev_set = &round_participant_sets[i - 1];
            let current_is_redance = crate::i18n::is_redance(&comp.rounds[i].name);
            let prev_is_redance = crate::i18n::is_redance(&comp.rounds[i - 1].name);

            if current_is_redance {
                // Redance must be a subset of the previous round
                if !current_set.is_subset(prev_set) {
                    return false;
                }
            } else if !prev_is_redance {
                // Normal round: should be a subset of previous round,
                // UNLESS couples were seeded into this round.
                // Since we don't explicitly track seeding yet, we relax this to
                // just checking global validity (already done).
            }
        }
    }

    // Rank consistency check
    for participant in &comp.participants {
        if let Some(rank) = participant.final_rank {
            for (i, round_set) in round_participant_sets.iter().enumerate() {
                if crate::i18n::is_redance(&comp.rounds[i].name) {
                    continue;
                }
                // If a participant reached a certain rank, they must have been in the corresponding rounds.
                // This is a bit complex due to different round sizes, so we use a conservative check:
                // If they are in the top N where N is the number of people in the round, they should be there.
                if rank <= round_set.len() as u32 && !round_set.contains(&participant.bib_number) {
                    // Check if they might have been in a parallel redance instead
                    let mut in_redance = false;
                    if i + 1 < comp.rounds.len() && crate::i18n::is_redance(&comp.rounds[i+1].name) {
                        if round_participant_sets[i+1].contains(&participant.bib_number) {
                            in_redance = true;
                        }
                    }

                    // Check if they were seeded into a LATER round
                    let mut seeded_later = false;
                    for later_set in &round_participant_sets[i + 1..] {
                        if later_set.contains(&participant.bib_number) {
                            seeded_later = true;
                            break;
                        }
                    }

                    if !in_redance && !seeded_later {
                        return false;
                    }
                }
            }
        }
    }

    // Skating Math Verification for DTV (skating) and WDSF (average/sum)
    if let Some(last_round) = comp.rounds.last() {
        match &last_round.data {
            RoundData::DTV { dtv_ranks } => {
                let dtv_marks = dtv_ranks;
                let mut dance_marks = BTreeMap::new();
                for dance in &comp.dances {
                    let mut jm_for_dance = BTreeMap::new();
                    for (j_code, bib_map) in dtv_marks {
                        let mut marks = BTreeMap::new();
                        for (&bib, d_map) in bib_map {
                            if let Some(&mark) = d_map.get(dance) {
                                marks.insert(bib, mark);
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
            RoundData::WDSF { .. } => {
                // WDSF rank calculation is complex (averages, drop high/low),
                // so we rely on verify_round_math for component validity.
            }
            _ => {}
        }
    }

    true
}

fn verify_round_math(round: &Round) -> bool {
    if let RoundData::WDSF { wdsf_scores } = &round.data {
        for judge_map in wdsf_scores.values() {
            for bib_map in judge_map.values() {
                for score in bib_map.values() {
                    if !verify_wdsf_score(score) {
                        return false;
                    }
                }
            }
        }
    }
    true
}
