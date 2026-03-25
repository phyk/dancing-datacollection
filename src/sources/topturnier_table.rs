use scraper::{ElementRef, Selector};
use crate::models::{Dance, Round, RoundData, Officials, WDSFScore};
use std::collections::BTreeMap;
use regex::Regex;
use std::sync::LazyLock;

static RE_SCORE: LazyLock<Regex> = LazyLock::new(|| Regex::new(crate::assets::PATTERN_SCORE).unwrap());
static RE_BIB_PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(crate::assets::PATTERN_BIB_PARENS).unwrap());

#[derive(Debug, Clone, Default)]
pub struct TableGrid {
    pub rows: Vec<Vec<String>>,
    pub width: usize,
    pub height: usize,
}

impl TableGrid {
    pub fn from_element(table: ElementRef) -> Self {
        let mut grid: BTreeMap<usize, BTreeMap<usize, String>> = BTreeMap::new();
        let mut max_w = 0;
        let mut max_h = 0;

        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel = Selector::parse("td, th").unwrap();

        for (r_idx, tr) in table.select(&tr_sel).enumerate() {
            let mut c_idx = 0;
            let cells: Vec<_> = tr.select(&td_sel).collect();
            for i in 0..cells.len() {
                let td = cells[i];
                let is_phantom = td.value().attr("class").is_some_and(|c| c.split_whitespace().any(|cls| cls == "td2ww"));

                while grid.get(&r_idx).and_then(|row| row.get(&c_idx)).is_some() {
                    c_idx += 1;
                }

                let rowspan = td.value().attr("rowspan").and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                let colspan = td.value().attr("colspan").and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                let mut content = extract_text(td);

                // Enrich td2w with following td2ww abbreviation if exists
                if td.value().attr("class").is_some_and(|c| c.split_whitespace().any(|cls| cls == "td2w")) && i + 1 < cells.len() {
                    let next_td = cells[i+1];
                    if next_td.value().attr("class").is_some_and(|c| c.split_whitespace().any(|cls| cls == "td2ww")) {
                        let abbr = extract_text(next_td);
                        if !abbr.is_empty() {
                            content = format!("{} ({})", content, abbr);
                        }
                    }
                }

                if is_phantom {
                    continue;
                }

                for dr in 0..rowspan {
                    for dc in 0..colspan {
                        grid.entry(r_idx + dr)
                            .or_default()
                            .insert(c_idx + dc, content.clone());
                        max_w = max_w.max(c_idx + dc + 1);
                        max_h = max_h.max(r_idx + dr + 1);
                    }
                }
                c_idx += colspan;
            }
        }

        let mut rows = Vec::with_capacity(max_h);
        for r in 0..max_h {
            let mut row = Vec::with_capacity(max_w);
            for c in 0..max_w {
                let val = grid.get(&r).and_then(|row| row.get(&c)).cloned().unwrap_or_default();
                row.push(val);
            }
            rows.push(row);
        }

        TableGrid {
            rows,
            width: max_w,
            height: max_h,
        }
    }
}

pub fn extract_text(el: ElementRef) -> String {
    let mut text = String::new();

    for node in el.children() {
        if let Some(element) = ElementRef::wrap(node) {
            let name = element.value().name();
            if name == "br" {
                text.push('\n');
            } else if element.value().attr("class").is_some_and(|c| {
                c.contains("tooltip") && !c.contains("tooltipnar")
            }) {
                continue;
            } else {
                text.push_str(&extract_text(element));
            }
        } else if let Some(t) = node.value().as_text() {
            text.push_str(t);
        }
    }
    text.replace('\u{a0}', " ").trim().to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Rank,
    Participant,
    Bib,
    Round,
    Mark { dance: Dance, judge: String },
    Dance(Dance),
    Sum,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableOrientation {
    Horizontal,
}

#[derive(Debug, Clone)]
pub struct IntermediateMark {
    pub dance: Dance,
    pub judge: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct IntermediateResult {
    pub bib: u32,
    pub rank: Option<String>,
    pub marks_by_round: BTreeMap<String, Vec<IntermediateMark>>,
}

pub fn identify_orientation(_grid: &TableGrid) -> TableOrientation {
    TableOrientation::Horizontal
}

pub fn identify_columns(grid: &TableGrid) -> Vec<ColumnType> {
    let mut col_types = vec![ColumnType::Unknown; grid.width];
    let mut col_dances = vec![None; grid.width];

    // 1. Identify dances and basic column types from header rows (0 and 1)
    for r in 0..grid.height.min(2) {
        for c in 0..grid.width {
            let val = &grid.rows[r][c];
            if val.is_empty() { continue; }

            // Identify basic columns
            if col_types[c] == ColumnType::Unknown {
                if crate::i18n::is_rank_column_marker(val) {
                    col_types[c] = ColumnType::Rank;
                } else if crate::i18n::is_participant_marker(val) {
                    col_types[c] = ColumnType::Participant;
                } else if crate::i18n::is_bib_column_marker(val) {
                    col_types[c] = ColumnType::Bib;
                } else if crate::i18n::is_round_column_marker(val) {
                    col_types[c] = ColumnType::Round;
                } else if crate::i18n::is_sum_column_marker(val) {
                    col_types[c] = ColumnType::Sum;
                }
            }

            // Identify dances
            let ds = crate::i18n::parse_dances_no_fallback(val);
            if !ds.is_empty() {
                if col_dances[c].is_none() {
                    col_dances[c] = Some(ds[0]);
                }
            }
        }
    }

    // 2. Identify Judges and specialized Sums in dance columns
    for c in 0..grid.width {
        if let Some(dance) = col_dances[c] {
            if col_types[c] == ColumnType::Unknown {
                // Check header rows for judge codes or sum markers
                for r in 0..grid.height.min(2) {
                    let val = &grid.rows[r][c];
                    if val.is_empty() { continue; }

                    if crate::i18n::is_sum_column_marker(val) {
                        col_types[c] = ColumnType::Sum;
                        break;
                    }

                    // A judge code in row 1 under a dance header in row 0
                    if val.len() <= 3 && crate::i18n::parse_dances_no_fallback(val).is_empty() {
                        col_types[c] = ColumnType::Mark { dance, judge: val.clone() };
                        break;
                    }
                }
            }
        }
    }

    // 2b. If we still have Unknown columns that have a dance, check row 1 again for ANY text that might be a judge code
    // This handles cases where judge codes might not be all uppercase (rare but possible) or other oddities.
    for c in 0..grid.width {
        if let Some(dance) = col_dances[c] {
            if col_types[c] == ColumnType::Unknown && grid.height > 1 {
                 let val = &grid.rows[1][c];
                 if !val.is_empty() && !crate::i18n::is_sum_column_marker(val) && val.len() <= 3 {
                      col_types[c] = ColumnType::Mark { dance, judge: val.clone() };
                 }
            }
        }
    }

    // 3. Defaults and Fallbacks
    let has_bib_col = col_types.iter().any(|t| matches!(t, ColumnType::Bib));

    for c in 0..grid.width {
        if col_types[c] == ColumnType::Unknown {
            // Participant/Bib fallback
            let row_to_check = if grid.height > 2 { 2 } else { 0 };
            if !has_bib_col && grid.height > row_to_check && RE_BIB_PARENS.is_match(&grid.rows[row_to_check][c]) {
                 col_types[c] = ColumnType::Participant;
            }

            // Collapsed dance column fallback
            if let Some(dance) = col_dances[c] {
                if col_types[c] == ColumnType::Unknown {
                    col_types[c] = ColumnType::Dance(dance);
                }
            }
        }
    }

    col_types
}

pub fn extract_data(grid: &TableGrid) -> Vec<IntermediateResult> {
    extract_horizontal(grid)
}

fn extract_horizontal(grid: &TableGrid) -> Vec<IntermediateResult> {
    let col_types = identify_columns(grid);
    let mut results = Vec::new();

    let bib_idx = col_types.iter().position(|t| matches!(t, ColumnType::Bib));
    let participant_idx = col_types.iter().position(|t| matches!(t, ColumnType::Participant));
    let rank_idx = col_types.iter().position(|t| matches!(t, ColumnType::Rank));
    let round_idx = col_types.iter().position(|t| matches!(t, ColumnType::Round));

    if bib_idx.is_none() && participant_idx.is_none() {
        return results;
    }

    let start_row = (0..grid.height).find(|&r| {
        if let Some(idx) = bib_idx {
            let val = grid.rows[r][idx].trim();
            !val.is_empty() && val.chars().all(|c| c.is_ascii_digit())
        } else if let Some(idx) = participant_idx {
            RE_BIB_PARENS.is_match(&grid.rows[r][idx])
        } else {
            false
        }
    }).unwrap_or(grid.height);

    for r in start_row..grid.height {
        let bib = if let Some(idx) = bib_idx {
            let bib_raw = grid.rows[r][idx].trim();
            let bib_str = bib_raw.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
            bib_str.parse::<u32>().ok()
        } else if let Some(idx) = participant_idx {
            RE_BIB_PARENS.captures(&grid.rows[r][idx])
                .and_then(|c| c[1].parse::<u32>().ok())
        } else {
            None
        };

        let bib = match bib {
            Some(b) => b,
            None => continue,
        };

        let rank = rank_idx.map(|idx| grid.rows[r][idx].clone());
        let round_vals: Vec<String> = round_idx
            .map(|idx| grid.rows[r][idx].split('\n').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| vec!["Final".to_string()]);

        let mut marks_by_round: BTreeMap<String, Vec<IntermediateMark>> = BTreeMap::new();

        for (c, col_type) in col_types.iter().enumerate() {
            match col_type {
                ColumnType::Mark { dance, judge } => {
                    let cell_val = &grid.rows[r][c];
                    let mark_vals: Vec<&str> = cell_val.split('\n').collect();
                    for (i, round_name) in round_vals.iter().enumerate() {
                        let mark_val = mark_vals.get(i).unwrap_or(&"").trim();
                        marks_by_round.entry(round_name.clone()).or_default().push(IntermediateMark {
                            dance: *dance,
                            judge: Some(judge.clone()),
                            value: mark_val.to_string(),
                        });
                    }
                }
                ColumnType::Dance(dance) => {
                    let cell_val = &grid.rows[r][c];
                    let mark_vals: Vec<&str> = cell_val.split('\n').collect();
                    for (i, round_name) in round_vals.iter().enumerate() {
                        let mark_val = mark_vals.get(i).unwrap_or(&"").trim();
                        marks_by_round.entry(round_name.clone()).or_default().push(IntermediateMark {
                            dance: *dance,
                            judge: None,
                            value: mark_val.to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        results.push(IntermediateResult {
            bib,
            rank,
            marks_by_round,
        });
    }

    results
}

pub fn to_rounds(intermediate: Vec<IntermediateResult>, dances: &[Dance], officials: &Officials) -> Vec<Round> {
    let mut rounds_map: BTreeMap<String, Round> = BTreeMap::new();

    // Pass 1: Determine round types
    let mut round_types: BTreeMap<String, bool> = BTreeMap::new();
    for res in &intermediate {
        for (round_id, marks) in &res.marks_by_round {
            let round_name = crate::i18n::get_round_name_from_id(round_id);
            if marks.iter().any(|m| !crate::i18n::map_wdsf_score_type(&m.value).is_empty()) {
                round_types.insert(round_name, true);
            }
        }
    }

    for res in intermediate {
        for (round_id, marks) in res.marks_by_round {
            let round_name = crate::i18n::get_round_name_from_id(&round_id);
            let is_wdsf = *round_types.get(&round_name).unwrap_or(&false);

            let round = rounds_map.entry(round_name.clone()).or_insert_with(|| {
                let is_final = crate::i18n::is_final_round(&round_name);
                let mut round_dances = Vec::new();
                for m in marks.iter() {
                    if !round_dances.contains(&m.dance) {
                        round_dances.push(m.dance);
                    }
                }
                if round_dances.is_empty() {
                    round_dances = dances.to_vec();
                }
                Round {
                    name: round_name.clone(),
                    order: 0,
                    dances: round_dances,
                    data: if is_wdsf {
                        RoundData::WDSF { wdsf_scores: BTreeMap::new() }
                    } else if is_final {
                        RoundData::DTV { dtv_ranks: BTreeMap::new() }
                    } else {
                        RoundData::Marking { marking_crosses: BTreeMap::new() }
                    },
                }
            });

            for mark in marks {
                if !round.dances.contains(&mark.dance) {
                    round.dances.push(mark.dance);
                }
                match &mut round.data {
                    RoundData::Marking { marking_crosses } => {
                        if let Some(judge) = mark.judge {
                            let is_cross = mark.value.to_lowercase().contains('x') || mark.value == "1";
                            marking_crosses.entry(judge).or_default()
                                .entry(res.bib).or_default()
                                .insert(mark.dance, is_cross);
                        } else {
                            if !mark.value.is_empty() {
                                if mark.value.len() > 1 && mark.value.chars().all(|c| c.is_ascii_digit()) {
                                    for (j_idx, ch) in mark.value.chars().enumerate() {
                                        if let Some(official_judge) = officials.judges.get(j_idx) {
                                            marking_crosses.entry(official_judge.code.clone()).or_default()
                                                .entry(res.bib).or_default()
                                                .insert(mark.dance, ch != '0' && ch != '-');
                                        }
                                    }
                                }
                            }
                        }
                    }
                    RoundData::DTV { dtv_ranks } => {
                        if let Some(judge) = mark.judge {
                            if let Ok(rank) = mark.value.replace(',', ".").parse::<f64>() {
                                dtv_ranks.entry(judge).or_default()
                                    .entry(res.bib).or_default()
                                    .insert(mark.dance, rank.round() as u32);
                            }
                        } else {
                            if mark.value.len() > 1 && mark.value.chars().all(|c| c.is_ascii_digit()) {
                                for (j_idx, ch) in mark.value.chars().enumerate() {
                                    if let Some(official_judge) = officials.judges.get(j_idx) {
                                        if let Some(rank) = ch.to_digit(10) {
                                            dtv_ranks.entry(official_judge.code.clone()).or_default()
                                                .entry(res.bib).or_default()
                                                .insert(mark.dance, rank);
                                        }
                                    }
                                }
                            } else if let Ok(total_rank) = mark.value.replace(',', ".").parse::<f64>() {
                                // Additive fallback: if we only have a single number and NO judges,
                                // it might be the calculated rank. We don't distribute it to all judges
                                // because that's inaccurate. Instead, we do nothing and let other files provide granular marks.
                                let _ = total_rank;
                            }
                        }
                    }
                    RoundData::WDSF { wdsf_scores } => {
                        if let Some(judge) = mark.judge {
                            let sc: Vec<f64> = RE_SCORE.find_iter(&mark.value)
                                .filter_map(|m| m.as_str().replace(',', ".").parse().ok())
                                .collect();
                            let score_types = crate::i18n::map_wdsf_score_type(&mark.value);
                            if !sc.is_empty() && !score_types.is_empty() {
                                let s = wdsf_scores.entry(judge).or_default()
                                    .entry(res.bib).or_default()
                                    .entry(mark.dance).or_insert_with(|| WDSFScore {
                                        technical_quality: 0.0,
                                        movement_to_music: 0.0,
                                        partnering_skills: 0.0,
                                        choreography: 0.0,
                                        total: 0.0,
                                    });

                                for (i, st) in score_types.iter().enumerate() {
                                    let val = if sc.len() == score_types.len() {
                                        sc[i]
                                    } else if sc.len() > i {
                                        sc[i]
                                    } else {
                                        sc[0]
                                    };
                                    match *st {
                                        "technical_quality" => s.technical_quality = val,
                                        "movement_to_music" => s.movement_to_music = val,
                                        "partnering_skills" => s.partnering_skills = val,
                                        "choreography" => s.choreography = val,
                                        "total" => s.total = val,
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut rounds: Vec<Round> = rounds_map.into_values().collect();
    // Heuristic for chronological order: count participants. Final has fewest.
    rounds.sort_by_key(|r| {
         let count = match &r.data {
             RoundData::Marking { marking_crosses } => marking_crosses.values().next().map(|m| m.len()).unwrap_or(0),
             RoundData::DTV { dtv_ranks } => dtv_ranks.values().next().map(|m| m.len()).unwrap_or(0),
             RoundData::WDSF { wdsf_scores } => wdsf_scores.values().next().map(|m| m.len()).unwrap_or(0),
         };
         if crate::i18n::is_final_round(&r.name) {
             1000
         } else {
             100 - (count as i32)
         }
    });
    for (i, r) in rounds.iter_mut().enumerate() {
        r.order = i as u32;
    }
    rounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_table_grid_complex() {
        let html = r#"
            <table>
                <tr>
                    <td rowspan="2">Rank</td>
                    <td rowspan="2">Nr.</td>
                    <td colspan="2">Samba</td>
                </tr>
                <tr>
                    <td>A</td>
                    <td>B</td>
                </tr>
                <tr>
                    <td>1.</td>
                    <td>101</td>
                    <td>1</td>
                    <td>2</td>
                </tr>
            </table>
        "#;
        let doc = Html::parse_document(html);
        let table = doc.select(&Selector::parse("table").unwrap()).next().unwrap();
        let grid = TableGrid::from_element(table);

        assert_eq!(identify_orientation(&grid), TableOrientation::Horizontal);
        let col_types = identify_columns(&grid);
        assert_eq!(col_types[1], ColumnType::Bib);

        let results = extract_data(&grid);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bib, 101);
    }

    #[test]
    fn test_multi_round_extraction() {
        let html = r#"
            <table>
                <tr>
                    <td>Rank</td>
                    <td>Nr</td>
                    <td>R</td>
                    <td colspan="2">Samba</td>
                </tr>
                <tr>
                    <td></td>
                    <td></td>
                    <td></td>
                    <td>A</td>
                    <td>B</td>
                </tr>
                <tr>
                    <td>1.</td>
                    <td>101</td>
                    <td>F<br>S</td>
                    <td>1<br>x</td>
                    <td>2<br>x</td>
                </tr>
            </table>
        "#;
        let doc = Html::parse_document(html);
        let table = doc.select(&Selector::parse("table").unwrap()).next().unwrap();
        let grid = TableGrid::from_element(table);
        let results = extract_data(&grid);

        assert_eq!(results.len(), 1);
        let res = &results[0];
        assert_eq!(res.marks_by_round.len(), 2);
    }

    #[test]
    fn test_tooltip_ignored() {
        let html = r#"<table><tr><td>A<span class="tooltip">Hidden</span>B</td></tr></table>"#;
        let doc = Html::parse_document(html);
        let td = doc.select(&Selector::parse("td").unwrap()).next().unwrap();
        let text = extract_text(td);
        assert_eq!(text, "AB");
    }
}
