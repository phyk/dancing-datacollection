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
                for k in 1..candidates.len() {
                    if candidates[k].1 == c0 && candidates[k].2 == s0 {
                        tied.push(candidates[k].0);
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
                for k in 1..tied.len() {
                    let mut identical = true;
                    for check_r in r..=num_participants as u32 {
                        if get_count_sum(tied[k], check_r, judge_marks)
                            != get_count_sum(winner_bib, check_r, judge_marks)
                        {
                            identical = false;
                            break;
                        }
                    }
                    if identical {
                        winners.push(tied[k]);
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
        assert_eq!(ranks[&101], 1.0);
        assert_eq!(ranks[&102], 2.0);
    }
}
