use crate::models::skating::{
    calculate_dance_ranks, calculate_final_ranks, calculate_wdsf_dance_score, dance_marks,
    verify_wdsf_score, wdsf_judge_scores,
};
use crate::models::{Competition, Dance, Judge, Participant, Round, RoundData, WDSFScore};
use std::collections::BTreeMap;

/// Helper method to check if all expected data is present in a round.
fn is_round_complete(round: &Round, expected_judges: &[Judge], dances: &[Dance]) -> bool {
    let round_participants = round.data.participant_bibs();
    if round_participants.is_empty() {
        return false;
    }

    for judge in expected_judges {
        // In WDSF, judges might only judge specific dances or rounds.
        // We only expect completeness if the judge has provided ANY data in this round.
        if !round.data.has_judge(&judge.code) {
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

    let Some(last_round) = comp.rounds.last() else {
        return false;
    };
    if let RoundData::Marking { .. } = &last_round.data {
        return false; // Last round must be a scoring round
    }

    let all_valid_bibs: std::collections::BTreeSet<u32> =
        comp.participants.iter().map(|p| p.bib_number).collect();
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
            // Redance must be a subset of the previous round. Normal rounds are not
            // checked against the previous round because couples may be seeded in.
            if crate::i18n::is_redance(&comp.rounds[i].name) && !current_set.is_subset(prev_set) {
                return false;
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
                    if i + 1 < comp.rounds.len()
                        && crate::i18n::is_redance(&comp.rounds[i + 1].name)
                        && round_participant_sets[i + 1].contains(&participant.bib_number)
                    {
                        in_redance = true;
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

    // Placement verification for the final round: DTV (skating system) and WDSF (score totals)
    match &last_round.data {
        RoundData::DTV { dtv_ranks } => {
            let mut all_dance_marks = BTreeMap::new();
            for &dance in &comp.dances {
                all_dance_marks.insert(dance, dance_marks(dtv_ranks, dance));
            }

            let dance_ranks: BTreeMap<Dance, BTreeMap<u32, f64>> = all_dance_marks
                .iter()
                .map(|(&dance, marks)| (dance, calculate_dance_ranks(marks)))
                .collect();

            let final_calc_ranks = calculate_final_ranks(&dance_ranks, Some(&all_dance_marks));
            let final_bibs = last_round.data.participant_bibs();
            for p in &comp.participants {
                let Some(expected) = p.final_rank else {
                    continue;
                };
                if !final_bibs.contains(&p.bib_number) {
                    continue;
                }
                match final_calc_ranks.get(&p.bib_number) {
                    Some(&calc) if calc == expected => {}
                    _ => return false,
                }
            }
        }
        RoundData::WDSF { wdsf_scores } => {
            if !verify_wdsf_placements(&comp.participants, last_round, wdsf_scores) {
                return false;
            }
        }
        RoundData::Marking { .. } => {}
    }

    true
}

/// Checks that the final ranks of WDSF finalists are ordered by their total score,
/// i.e. the sum over the round's dances of `calculate_wdsf_dance_score`.
/// Totals within 0.001 of each other are treated as ties and may appear in any order.
fn verify_wdsf_placements(
    participants: &[Participant],
    round: &Round,
    wdsf_scores: &BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, WDSFScore>>>,
) -> bool {
    let finalists: Vec<(u32, f64)> = participants
        .iter()
        .filter_map(|p| {
            let rank = p.final_rank?;
            let mut total = 0.0;
            let mut has_data = false;
            for &dance in &round.dances {
                let judge_scores = wdsf_judge_scores(wdsf_scores, p.bib_number, dance);
                if !judge_scores.is_empty() {
                    total += calculate_wdsf_dance_score(&judge_scores);
                    has_data = true;
                }
            }
            has_data.then_some((rank, total))
        })
        .collect();

    // A strictly higher total must never have a worse (numerically greater) rank.
    finalists.iter().all(|&(rank_a, total_a)| {
        finalists
            .iter()
            .all(|&(rank_b, total_b)| total_a <= total_b + 0.001 || rank_a <= rank_b)
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IdentityType;

    type WdsfScores = BTreeMap<String, BTreeMap<u32, BTreeMap<Dance, WDSFScore>>>;

    fn participant(bib: u32, rank: u32) -> Participant {
        Participant {
            identity_type: IdentityType::Couple,
            name_one: format!("P{}", bib),
            bib_number: bib,
            name_two: None,
            affiliation: None,
            final_rank: Some(rank),
        }
    }

    fn wdsf_round(tq_by_bib: &[(u32, f64)]) -> (Round, WdsfScores) {
        let mut scores: WdsfScores = BTreeMap::new();
        for &(bib, tq) in tq_by_bib {
            scores
                .entry("A".to_string())
                .or_default()
                .entry(bib)
                .or_default()
                .insert(
                    Dance::Samba,
                    WDSFScore {
                        technical_quality: tq,
                        total: tq,
                        ..WDSFScore::default()
                    },
                );
        }
        let round = Round {
            name: "Final".to_string(),
            order: 0,
            dances: vec![Dance::Samba],
            data: RoundData::WDSF {
                wdsf_scores: scores.clone(),
            },
        };
        (round, scores)
    }

    #[test]
    fn test_wdsf_placements() {
        let (round, scores) = wdsf_round(&[(1, 9.0), (2, 8.0)]);
        let ok = [participant(1, 1), participant(2, 2)];
        assert!(verify_wdsf_placements(&ok, &round, &scores));
        let swapped = [participant(1, 2), participant(2, 1)];
        assert!(!verify_wdsf_placements(&swapped, &round, &scores));

        let (tied_round, tied_scores) = wdsf_round(&[(1, 8.0), (2, 8.0)]);
        assert!(verify_wdsf_placements(&swapped, &tied_round, &tied_scores));
    }
}
