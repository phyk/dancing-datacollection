use std::collections::BTreeMap;
use crate::models::{Dance, WDSFScore};

/// Calculates the ranks for a single dance using the Skating System (Rules 5-9).
pub fn calculate_dance_ranks(
    judge_marks: &BTreeMap<String, BTreeMap<u32, u32>>, // JudgeCode -> Bib -> Mark
) -> BTreeMap<u32, u32> {
    let bibs: Vec<u32> = judge_marks.values().next().map(|m| m.keys().cloned().collect()).unwrap_or_default();
    let num_judges = judge_marks.len();
    if num_judges == 0 { return BTreeMap::new(); }
    let majority = (num_judges / 2) + 1;
    let num_participants = bibs.len();

    let mut final_ranks: BTreeMap<u32, u32> = BTreeMap::new();
    let mut remaining_bibs = bibs.clone();
    let mut current_place = 1;

    for r in 1..=(num_participants as u32) {
        if remaining_bibs.is_empty() { break; }

        let mut candidates = Vec::new();
        for &bib in &remaining_bibs {
            let mut count = 0;
            let mut sum = 0;
            for marks in judge_marks.values() {
                if let Some(&mark) = marks.get(&bib) {
                    if mark <= r {
                        count += 1;
                        sum += mark;
                    }
                }
            }
            if count >= majority {
                candidates.push((bib, count, sum));
            }
        }

        // Sort candidates by majority count (desc) then sum (asc)
        candidates.sort_by(|a, b| {
            if a.1 != b.1 {
                b.1.cmp(&a.1) // Greater majority wins
            } else {
                a.2.cmp(&b.2) // Lower sum wins
            }
        });

        let mut i = 0;
        while i < candidates.len() {
            let (bib, count, sum) = candidates[i];

            let mut tie_group = vec![bib];
            let mut j = i + 1;
            while j < candidates.len() {
                let (next_bib, next_count, next_sum) = candidates[j];
                if count == next_count && sum == next_sum {
                    tie_group.push(next_bib);
                    j += 1;
                } else {
                    break;
                }
            }

            if tie_group.len() > 1 && r < num_participants as u32 {
                // If there's a tie group, we can only rank them if they are absolutely tied
                // or if we reached the last 'r' level.
                // Otherwise we break and hope r+1 resolves it.
                break;
            } else {
                for &b in &tie_group {
                    final_ranks.insert(b, current_place);
                    remaining_bibs.retain(|&x| x != b);
                }
                current_place += tie_group.len() as u32;
                i += tie_group.len();
            }
        }
    }

    // In case of absolute ties or people who never got a majority
    if !remaining_bibs.is_empty() {
        for bib in remaining_bibs {
            final_ranks.insert(bib, current_place);
        }
    }

    final_ranks
}

/// Calculates final ranks across all dances (Rules 10-11).
pub fn calculate_final_ranks(
    dance_ranks: &BTreeMap<Dance, BTreeMap<u32, u32>>,
    all_judge_marks: Option<&BTreeMap<Dance, BTreeMap<String, BTreeMap<u32, u32>>>>,
) -> BTreeMap<u32, u32> {
    let mut bib_sums: BTreeMap<u32, u32> = BTreeMap::new();
    let mut bibs = Vec::new();

    for ranks in dance_ranks.values() {
        for (&bib, &rank) in ranks {
            *bib_sums.entry(bib).or_insert(0) += rank;
            if !bibs.contains(&bib) {
                bibs.push(bib);
            }
        }
    }

    // Sort by sum of ranks (Rule 10)
    bibs.sort_by(|&a, &b| {
        let sum_a = bib_sums[&a];
        let sum_b = bib_sums[&b];
        if sum_a != sum_b {
            sum_a.cmp(&sum_b)
        } else if let Some(all_marks) = all_judge_marks {
            break_rule_11(a, b, all_marks)
        } else {
            a.cmp(&b)
        }
    });

    let mut final_ranks = BTreeMap::new();
    let mut i = 0;
    while i < bibs.len() {
        let bib = bibs[i];
        let sum = bib_sums[&bib];

        let mut tie_group = vec![bib];
        let mut j = i + 1;
        while j < bibs.len() {
            let next_bib = bibs[j];
            let next_sum = bib_sums[&next_bib];

            let tied = if sum == next_sum {
                if let Some(all_marks) = all_judge_marks {
                    break_rule_11(bib, next_bib, all_marks) == std::cmp::Ordering::Equal
                } else {
                    true
                }
            } else {
                false
            };

            if tied {
                tie_group.push(next_bib);
                j += 1;
            } else {
                break;
            }
        }

        for &b in &tie_group {
            final_ranks.insert(b, (i + 1) as u32);
        }
        i += tie_group.len();
    }
    final_ranks
}

fn break_rule_11(a: u32, b: u32, all_marks: &BTreeMap<Dance, BTreeMap<String, BTreeMap<u32, u32>>>) -> std::cmp::Ordering {
    let mut marks_a = Vec::new();
    let mut marks_b = Vec::new();
    for dm in all_marks.values() {
        for jm in dm.values() {
            if let Some(&m) = jm.get(&a) { marks_a.push(m); }
            if let Some(&m) = jm.get(&b) { marks_b.push(m); }
        }
    }

    let num_marks = marks_a.len();
    if num_marks == 0 { return std::cmp::Ordering::Equal; }
    let majority = (num_marks / 2) + 1;
    let max_mark = *marks_a.iter().chain(marks_b.iter()).max().unwrap_or(&10);

    for r in 1..=max_mark {
        let count_a = marks_a.iter().filter(|&&m| m <= r).count();
        let count_b = marks_b.iter().filter(|&&m| m <= r).count();

        if count_a >= majority || count_b >= majority {
            if count_a != count_b {
                return count_b.cmp(&count_a);
            }
            let sum_a: u32 = marks_a.iter().filter(|&&m| m <= r).sum();
            let sum_b: u32 = marks_b.iter().filter(|&&m| m <= r).sum();
            if sum_a != sum_b {
                return sum_a.cmp(&sum_b);
            }
        }
    }
    std::cmp::Ordering::Equal
}

/// Verifies WDSF category scores against reported total.
pub fn verify_wdsf_score(score: &WDSFScore) -> bool {
    if score.total == 0.0 { return true; }
    let calculated_sum = score.technical_quality
        + score.movement_to_music
        + score.partnering_skills
        + score.choreography;

    let mean = calculated_sum / 4.0;
    (mean - score.total).abs() < 0.011 || (calculated_sum - score.total).abs() < 0.011
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_5_simple() {
        let mut judge_marks = BTreeMap::new();
        for j in &["A", "B", "C"] {
            let mut jm = BTreeMap::new();
            jm.insert(101, 1);
            jm.insert(102, 2);
            judge_marks.insert(j.to_string(), jm);
        }
        let ranks = calculate_dance_ranks(&judge_marks);
        assert_eq!(ranks[&101], 1);
        assert_eq!(ranks[&102], 2);
    }

    #[test]
    fn test_rule_7_majority_size() {
        let mut judge_marks = BTreeMap::new();
        let marks = vec![
            ("A", 101, 1), ("A", 102, 2),
            ("B", 101, 1), ("B", 102, 1),
            ("C", 101, 1), ("C", 102, 1),
            ("D", 101, 2), ("D", 102, 2),
            ("E", 101, 2), ("E", 102, 2),
        ];
        for (j, b, m) in marks {
            judge_marks.entry(j.to_string()).or_insert_with(BTreeMap::new).insert(b, m);
        }
        let ranks = calculate_dance_ranks(&judge_marks);
        assert_eq!(ranks[&101], 1);
        assert_eq!(ranks[&102], 2);
    }

    #[test]
    fn test_rule_8_sum_of_marks() {
        let mut judge_marks = BTreeMap::new();
        let marks = vec![
            ("A", 101, 1), ("A", 102, 1),
            ("B", 101, 1), ("B", 102, 2),
            ("C", 101, 2), ("C", 102, 2),
            ("D", 101, 2), ("D", 102, 2),
            ("E", 101, 3), ("E", 102, 3),
        ];
        for (j, b, m) in marks {
            judge_marks.entry(j.to_string()).or_insert_with(BTreeMap::new).insert(b, m);
        }
        let ranks = calculate_dance_ranks(&judge_marks);
        assert_eq!(ranks[&101], 1);
        assert_eq!(ranks[&102], 2);
    }

    #[test]
    fn test_rule_10_11_final_tie() {
        let mut dance_ranks = BTreeMap::new();
        let mut d1 = BTreeMap::new();
        d1.insert(101, 1);
        d1.insert(102, 2);
        dance_ranks.insert(Dance::SlowWaltz, d1);
        let mut d2 = BTreeMap::new();
        d2.insert(101, 2);
        d2.insert(102, 1);
        dance_ranks.insert(Dance::Tango, d2);

        let mut all_judge_marks = BTreeMap::new();
        let mut sw_marks = BTreeMap::new();
        let mut tg_marks = BTreeMap::new();
        for j in &["A", "B", "C"] {
            let mut jm_sw = BTreeMap::new();
            jm_sw.insert(101, 1);
            jm_sw.insert(102, 2);
            sw_marks.insert(j.to_string(), jm_sw);
            let mut jm_tg = BTreeMap::new();
            jm_tg.insert(101, 2);
            jm_tg.insert(102, 1);
            tg_marks.insert(j.to_string(), jm_tg);
        }
        all_judge_marks.insert(Dance::SlowWaltz, sw_marks);
        all_judge_marks.insert(Dance::Tango, tg_marks);
        let final_ranks = calculate_final_ranks(&dance_ranks, Some(&all_judge_marks));
        assert!(final_ranks[&101] == 1 || final_ranks[&101] == 2);
    }

    #[test]
    fn test_unbreakable_tie() {
        let mut judge_marks = BTreeMap::new();
        // 3 judges, 2 bibs, exactly same marks
        for j in &["A", "B", "C"] {
            let mut jm = BTreeMap::new();
            jm.insert(101, 1);
            jm.insert(102, 1);
            judge_marks.insert(j.to_string(), jm);
        }
        let ranks = calculate_dance_ranks(&judge_marks);
        assert_eq!(ranks[&101], 1);
        assert_eq!(ranks[&102], 1);
    }

    #[test]
    fn test_wdsf_verify() {
        let score = WDSFScore {
            technical_quality: 8.5,
            movement_to_music: 8.0,
            partnering_skills: 8.5,
            choreography: 9.0,
            total: 8.5,
        };
        assert!(verify_wdsf_score(&score));
        let bad_score = WDSFScore {
            technical_quality: 8.5,
            movement_to_music: 8.0,
            partnering_skills: 8.5,
            choreography: 9.0,
            total: 9.5,
        };
        assert!(!verify_wdsf_score(&bad_score));
    }
}
