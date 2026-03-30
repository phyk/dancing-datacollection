type JudgeMarksMap = BTreeMap<Dance, BTreeMap<String, BTreeMap<u32, u32>>>;
use crate::models::{Dance, WDSFScore};
use std::collections::BTreeMap;

fn get_count_sum(
    bib: u32,
    r: u32,
    judge_marks: &BTreeMap<String, BTreeMap<u32, u32>>,
) -> (usize, u32) {
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
    (count, sum)
}

/// Calculates the ranks for a single dance using the Skating System (Rules 5-9).
pub fn calculate_dance_ranks(
    judge_marks: &BTreeMap<String, BTreeMap<u32, u32>>, // JudgeCode -> Bib -> Mark
) -> BTreeMap<u32, f64> {
    let bibs: Vec<u32> = judge_marks
        .values()
        .next()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let num_judges = judge_marks.len();
    if num_judges == 0 {
        return BTreeMap::new();
    }
    let majority = (num_judges / 2) + 1;
    let num_participants = bibs.len();

    let mut final_ranks: BTreeMap<u32, f64> = BTreeMap::new();
    let mut remaining = bibs;
    let mut next_place = 1;

    while !remaining.is_empty() {
        let mut found_winner = false;
        for r in 1..=num_participants as u32 {
            let mut candidates = Vec::new();
            for &bib in &remaining {
                let (count, sum) = get_count_sum(bib, r, judge_marks);
                if count >= majority {
                    candidates.push((bib, count, sum));
                }
            }

            if !candidates.is_empty() {
                candidates.sort_by(|a, b| {
                    if b.1 != a.1 {
                        b.1.cmp(&a.1)
                    } else {
                        a.2.cmp(&b.2)
                    }
                });

                let (b0, c0, s0) = candidates[0];
                let mut tied = vec![b0];
                for cand in candidates.iter().skip(1) {
                    if cand.1 == c0 && cand.2 == s0 {
                        tied.push(cand.0);
                    } else {
                        break;
                    }
                }

                if tied.len() > 1 {
                    for next_r in (r + 1)..=num_participants as u32 {
                        tied.sort_by(|&ba, &bb| {
                            let (ca, sa) = get_count_sum(ba, next_r, judge_marks);
                            let (cb, sb) = get_count_sum(bb, next_r, judge_marks);
                            if ca != cb {
                                cb.cmp(&ca)
                            } else if sa != sb {
                                sa.cmp(&sb)
                            } else {
                                std::cmp::Ordering::Equal
                            }
                        });
                        let (c0_new, s0_new) = get_count_sum(tied[0], next_r, judge_marks);
                        let (c1_new, s1_new) = get_count_sum(tied[1], next_r, judge_marks);
                        if c0_new != c1_new || s0_new != s1_new {
                            break;
                        }
                    }
                }

                let winner_bib = tied[0];
                let mut winners = vec![winner_bib];
                for &t_bib in tied.iter().skip(1) {
                    let mut identical = true;
                    for check_r in r..=num_participants as u32 {
                        if get_count_sum(t_bib, check_r, judge_marks)
                            != get_count_sum(winner_bib, check_r, judge_marks)
                        {
                            identical = false;
                            break;
                        }
                    }
                    if identical {
                        winners.push(t_bib);
                    } else {
                        break;
                    }
                }

                let avg_rank = next_place as f64 + (winners.len() as f64 - 1.0) / 2.0;
                for &w in &winners {
                    final_ranks.insert(w, avg_rank);
                    remaining.retain(|&x| x != w);
                }
                next_place += winners.len();
                found_winner = true;
                break;
            }
        }
        if !found_winner {
            break;
        }
    }

    for bib in remaining {
        final_ranks.insert(bib, next_place as f64);
    }

    final_ranks
}

/// Calculates final ranks across all dances (Rules 10-12).
pub fn calculate_final_ranks(
    dance_ranks: &BTreeMap<Dance, BTreeMap<u32, f64>>,
    all_judge_marks: Option<&JudgeMarksMap>,
) -> BTreeMap<u32, u32> {
    let mut bib_sums: BTreeMap<u32, f64> = BTreeMap::new();
    let mut bibs = Vec::new();

    for ranks in dance_ranks.values() {
        for (&bib, &rank) in ranks {
            *bib_sums.entry(bib).or_insert(0.0) += rank;
            if !bibs.contains(&bib) {
                bibs.push(bib);
            }
        }
    }

    bibs.sort_by(|&a, &b| {
        let sum_a = bib_sums[&a];
        let sum_b = bib_sums[&b];
        if (sum_a - sum_b).abs() > 0.001 {
            sum_a.partial_cmp(&sum_b).unwrap()
        } else {
            let cmp_r11 = break_rule_11_dance_ranks(a, b, dance_ranks);
            if cmp_r11 != std::cmp::Ordering::Equal {
                cmp_r11
            } else if let Some(all_marks) = all_judge_marks {
                break_rule_12(a, b, all_marks)
            } else {
                a.cmp(&b)
            }
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

            let tied = if (sum - next_sum).abs() < 0.001 {
                if break_rule_11_dance_ranks(bib, next_bib, dance_ranks)
                    == std::cmp::Ordering::Equal
                {
                    if let Some(all_marks) = all_judge_marks {
                        break_rule_12(bib, next_bib, all_marks) == std::cmp::Ordering::Equal
                    } else {
                        true
                    }
                } else {
                    false
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

fn break_rule_11_dance_ranks(
    a: u32,
    b: u32,
    dance_ranks: &BTreeMap<Dance, BTreeMap<u32, f64>>,
) -> std::cmp::Ordering {
    let mut ranks_a: Vec<f64> = dance_ranks
        .values()
        .filter_map(|m| m.get(&a).cloned())
        .collect();
    let mut ranks_b: Vec<f64> = dance_ranks
        .values()
        .filter_map(|m| m.get(&b).cloned())
        .collect();
    ranks_a.sort_by(|x, y| x.partial_cmp(y).unwrap());
    ranks_b.sort_by(|x, y| x.partial_cmp(y).unwrap());

    let num_dances = ranks_a.len();
    let majority = (num_dances / 2) + 1;
    let max_rank = ranks_a
        .iter()
        .chain(ranks_b.iter())
        .cloned()
        .fold(0.0, f64::max) as u32;

    for r in 1..=max_rank {
        let count_a = ranks_a.iter().filter(|&&rk| rk <= r as f64).count();
        let count_b = ranks_b.iter().filter(|&&rk| rk <= r as f64).count();

        if count_a >= majority || count_b >= majority {
            if count_a != count_b {
                return count_b.cmp(&count_a);
            }
            let sum_a: f64 = ranks_a.iter().filter(|&&rk| rk <= r as f64).sum();
            let sum_b: f64 = ranks_b.iter().filter(|&&rk| rk <= r as f64).sum();
            if (sum_a - sum_b).abs() > 0.001 {
                return sum_a.partial_cmp(&sum_b).unwrap();
            }
        }
    }
    std::cmp::Ordering::Equal
}

fn break_rule_12(
    a: u32,
    b: u32,
    all_marks: &BTreeMap<Dance, BTreeMap<String, BTreeMap<u32, u32>>>,
) -> std::cmp::Ordering {
    let mut marks_a = Vec::new();
    let mut marks_b = Vec::new();
    for dance_map in all_marks.values() {
        for judge_map in dance_map.values() {
            if let Some(&mark) = judge_map.get(&a) {
                marks_a.push(mark);
            }
            if let Some(&mark) = judge_map.get(&b) {
                marks_b.push(mark);
            }
        }
    }

    let num_marks = marks_a.len();
    if num_marks == 0 {
        return std::cmp::Ordering::Equal;
    }
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
    if score.total == 0.0 {
        return true;
    }
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
        assert!((ranks[&101] - 1.0).abs() < 0.001);
        assert!((ranks[&102] - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_rule_7_majority_size() {
        let mut judge_marks = BTreeMap::new();
        let marks = vec![
            ("A", 101, 1),
            ("A", 102, 2),
            ("B", 101, 1),
            ("B", 102, 1),
            ("C", 101, 1),
            ("C", 102, 1),
            ("D", 101, 2),
            ("D", 102, 2),
            ("E", 101, 2),
            ("E", 102, 2),
        ];
        for (j, b, m) in marks {
            judge_marks
                .entry(j.to_string())
                .or_insert_with(BTreeMap::new)
                .insert(b, m);
        }
        let ranks = calculate_dance_ranks(&judge_marks);
        assert!((ranks[&101] - 1.0).abs() < 0.001);
        assert!((ranks[&102] - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_rule_8_sum_of_marks() {
        let mut judge_marks = BTreeMap::new();
        let marks = vec![
            ("A", 101, 1),
            ("A", 102, 1),
            ("B", 101, 1),
            ("B", 102, 2),
            ("C", 101, 2),
            ("C", 102, 2),
            ("D", 101, 2),
            ("D", 102, 2),
            ("E", 101, 3),
            ("E", 102, 3),
        ];
        for (j, b, m) in marks {
            judge_marks
                .entry(j.to_string())
                .or_insert_with(BTreeMap::new)
                .insert(b, m);
        }
        let ranks = calculate_dance_ranks(&judge_marks);
        assert!((ranks[&101] - 1.0).abs() < 0.001);
        assert!((ranks[&102] - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_rule_10_11_final_tie() {
        let mut dance_ranks = BTreeMap::new();
        let mut d1 = BTreeMap::new();
        d1.insert(101, 1.0);
        d1.insert(102, 2.0);
        dance_ranks.insert(Dance::SlowWaltz, d1);
        let mut d2 = BTreeMap::new();
        d2.insert(101, 2.0);
        d2.insert(102, 1.0);
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
        assert!((ranks[&101] - 1.5).abs() < 0.001);
        assert!((ranks[&102] - 1.5).abs() < 0.001);
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
