use scraper::{ElementRef, Selector};
use crate::models::Dance;
use std::collections::BTreeMap;

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
            for td in tr.select(&td_sel) {
                while grid.get(&r_idx).and_then(|row| row.get(&c_idx)).is_some() {
                    c_idx += 1;
                }

                let rowspan = td.value().attr("rowspan").and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                let colspan = td.value().attr("colspan").and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                let content = extract_text(td);

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

fn extract_text(el: ElementRef) -> String {
    let mut text = String::new();

    for node in el.children() {
        if let Some(element) = ElementRef::wrap(node) {
            let name = element.value().name();
            if name == "br" {
                text.push('\n');
            } else if element.value().attr("class").map_or(false, |c| {
                c.contains("tooltip")
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
    Sum,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct IntermediateMark {
    pub dance: Dance,
    pub judge: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct IntermediateResult {
    pub bib: String,
    pub rank: Option<String>,
    pub marks_by_round: BTreeMap<String, Vec<IntermediateMark>>,
}

pub fn identify_columns(grid: &TableGrid) -> Vec<ColumnType> {
    let mut col_types = vec![ColumnType::Unknown; grid.width];
    let mut col_dances = vec![None; grid.width];

    for r in 0..grid.height.min(5) {
        for c in 0..grid.width {
            let val = &grid.rows[r][c];
            if val.is_empty() { continue; }

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
            } else {
                let ds = crate::i18n::parse_dances_no_fallback(val);
                if !ds.is_empty() {
                    col_dances[c] = Some(ds[0]);
                }
            }
        }
    }

    for c in 0..grid.width {
        if col_types[c] == ColumnType::Unknown {
            if let Some(dance) = col_dances[c] {
                for r in 0..grid.height.min(5) {
                    let val = &grid.rows[r][c];
                    if val.is_empty() { continue; }

                    if val.len() <= 3 && val.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()) {
                        if crate::i18n::parse_dances_no_fallback(val).is_empty() {
                            col_types[c] = ColumnType::Mark { dance, judge: val.clone() };
                            break;
                        }
                    }
                }
            }
        }
    }

    col_types
}

pub fn extract_ergwert(grid: &TableGrid) -> Vec<IntermediateResult> {
    let col_types = identify_columns(grid);
    let mut results = Vec::new();

    let bib_idx = col_types.iter().position(|t| matches!(t, ColumnType::Bib));
    let rank_idx = col_types.iter().position(|t| matches!(t, ColumnType::Rank));
    let round_idx = col_types.iter().position(|t| matches!(t, ColumnType::Round));

    if bib_idx.is_none() {
        return results;
    }
    let bib_idx = bib_idx.unwrap();

    let start_row = (0..grid.height).find(|&r| {
        let val = grid.rows[r][bib_idx].trim();
        !val.is_empty() && val.chars().all(|c| c.is_ascii_digit())
    }).unwrap_or(grid.height);

    for r in start_row..grid.height {
        let bib_raw = grid.rows[r][bib_idx].trim();
        let bib = bib_raw.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
        if bib.is_empty() {
            continue;
        }

        let rank = rank_idx.map(|idx| grid.rows[r][idx].clone());
        let round_vals: Vec<String> = round_idx
            .map(|idx| grid.rows[r][idx].split('\n').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| vec!["Final".to_string()]);

        let mut marks_by_round: BTreeMap<String, Vec<IntermediateMark>> = BTreeMap::new();

        for (c, col_type) in col_types.iter().enumerate() {
            if let ColumnType::Mark { dance, judge } = col_type {
                let cell_val = &grid.rows[r][c];
                let mark_vals: Vec<&str> = cell_val.split('\n').collect();

                for (i, round_name) in round_vals.iter().enumerate() {
                    if let Some(&mark_val) = mark_vals.get(i) {
                        let mark_val = mark_val.trim();
                        if !mark_val.is_empty() && mark_val != "-" {
                            marks_by_round.entry(round_name.clone()).or_default().push(IntermediateMark {
                                dance: *dance,
                                judge: judge.clone(),
                                value: mark_val.to_string(),
                            });
                        }
                    }
                }
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
                    <td rowspan="2">No</td>
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

        assert_eq!(grid.width, 4);
        assert_eq!(grid.height, 3);
        assert_eq!(grid.rows[0][0], "Rank");
        assert_eq!(grid.rows[1][0], "Rank");
        assert_eq!(grid.rows[0][2], "Samba");
        assert_eq!(grid.rows[0][3], "Samba");
        assert_eq!(grid.rows[1][2], "A");
        assert_eq!(grid.rows[1][3], "B");

        let col_types = identify_columns(&grid);
        assert_eq!(col_types[0], ColumnType::Rank);
        assert_eq!(col_types[1], ColumnType::Bib);
        assert!(matches!(col_types[2], ColumnType::Mark { dance: Dance::Samba, judge: ref j } if j == "A"));
        assert!(matches!(col_types[3], ColumnType::Mark { dance: Dance::Samba, judge: ref j } if j == "B"));

        let results = extract_ergwert(&grid);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bib, "101");
        assert_eq!(results[0].rank, Some("1.".to_string()));
        let marks = results[0].marks_by_round.get("Final").unwrap();
        assert_eq!(marks.len(), 2);
    }

    #[test]
    fn test_multi_round_extraction() {
        let html = r#"
            <table>
                <tr>
                    <td>Rank</td>
                    <td>No</td>
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
        let results = extract_ergwert(&grid);

        assert_eq!(results.len(), 1);
        let res = &results[0];
        assert_eq!(res.marks_by_round.len(), 2);

        let final_marks = res.marks_by_round.get("F").unwrap();
        assert_eq!(final_marks.len(), 2);
        assert_eq!(final_marks[0].value, "1");

        let semi_marks = res.marks_by_round.get("S").unwrap();
        assert_eq!(semi_marks.len(), 2);
        assert_eq!(semi_marks[0].value, "x");
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
