use crate::assets::*;
use crate::models::{
    AgeGroup, CommitteeMember, Competition, Dance, IdentityType, Judge, Level, Officials,
    Participant, Round, RoundData, Style, WDSFScore,
};
use crate::sources::ParsingError;
use anyhow::Result;
use chrono::NaiveDate;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

// --- Selectors & Regex ---
static SEL_TR: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_TR).unwrap());
static SEL_TD: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_TD).unwrap());
static SEL_SPAN: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_SPAN).unwrap());
static SEL_I: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_I).unwrap());
static SEL_TITLE: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_TITLE).unwrap());
static RE_BIB_PARENS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(PATTERN_BIB_PARENS).unwrap());
static RE_SCORE: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN_SCORE).unwrap());
static RE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN_DATE).unwrap());

// --- Utilities ---
fn clean(s: &str) -> String {
    s.trim().to_string()
}
fn txt(el: &ElementRef) -> String {
    el.text().collect::<String>().trim().to_string()
}

pub fn parse_date(s: &str) -> Option<NaiveDate> {
    let caps = RE_DATE.captures(s)?;
    let d = caps[1].parse::<u32>().ok()?;
    let m_str = &caps[2];
    let y = caps[3].parse::<i32>().ok()?;
    let m = m_str
        .parse::<u32>()
        .ok()
        .or_else(|| crate::i18n::map_month(m_str))?;
    NaiveDate::from_ymd_opt(y, m, d)
}

pub fn parse_metadata(
    html: &str,
) -> (
    Option<String>,
    Option<NaiveDate>,
    Option<String>,
    Option<String>,
) {
    let doc = Html::parse_document(html);
    let (mut org, mut club) = (None, None);
    if let Some(meta_author) = doc
        .select(&Selector::parse(SELECTOR_META_AUTHOR).unwrap())
        .next()
    {
        org = meta_author.value().attr("content").map(|s| s.to_string());
    }

    for row in doc.select(&SEL_TR) {
        let cells: Vec<_> = row.select(&SEL_TD).collect();
        if cells.len() >= 2 {
            let k = txt(&cells[0]);
            let v = txt(&cells[1]);
            if crate::i18n::is_organizer_marker(&k) {
                org = Some(v);
            } else if crate::i18n::is_hosting_club_marker(&k) {
                club = Some(v);
            }
        }
    }
    let name = doc
        .select(&Selector::parse(SELECTOR_EVENT_HEAD).unwrap())
        .next()
        .map(|el| txt(&el));
    let date = doc
        .select(&Selector::parse(SELECTOR_COMP_HEAD).unwrap())
        .next()
        .map(|el| txt(&el))
        .and_then(|d| parse_date(&d));
    (name, date, org, club)
}

fn parse_participant_row(row: ElementRef) -> Result<Participant, ParsingError> {
    let rank = row
        .select(&Selector::parse(SELECTOR_PARTICIPANT_RANK).unwrap())
        .next()
        .and_then(|el| txt(&el).trim_end_matches('.').parse::<u32>().ok());
    let data_cells: Vec<_> = row
        .select(&Selector::parse(SELECTOR_PARTICIPANT_DATA).unwrap())
        .collect();
    if data_cells.is_empty() {
        return Err(ParsingError::ParsingError("NoData".into()));
    }
    let (mut bib, mut name, mut club) = (0, String::new(), None);
    if data_cells.len() >= 2 {
        if let Ok(b) = txt(&data_cells[0]).parse::<u32>() {
            bib = b;
            name = txt(&data_cells[1]);
            club = data_cells.get(2).map(|c| txt(c));
        }
    }
    if bib == 0 {
        let full = txt(&data_cells[0]);
        club = data_cells[0].select(&SEL_I).next().map(|el| txt(&el));
        let name_bib = if let Some(ref c) = club {
            full.replace(c, "").trim().to_string()
        } else {
            full
        };
        if let Some(caps) = RE_BIB_PARENS.captures(&name_bib) {
            bib = caps[1].parse().unwrap_or(0);
            name = RE_BIB_PARENS.replace_all(&name_bib, "").trim().to_string();
        } else {
            name = name_bib;
        }
    }
    if bib == 0 {
        return Err(ParsingError::MissingRequiredData("Bib".into()));
    }
    let (it, n1, n2) = if name.contains(" / ") {
        let p: Vec<_> = name.split(" / ").collect();
        (
            IdentityType::Couple,
            p[0].trim().into(),
            Some(p[1].trim().into()),
        )
    } else {
        (IdentityType::Solo, name, None)
    };
    if n1.is_empty() {
        return Err(ParsingError::MissingRequiredData("Name".into()));
    }
    Ok(Participant {
        identity_type: it,
        name_one: n1,
        bib_number: bib,
        name_two: n2,
        affiliation: club,
        final_rank: rank,
    })
}

#[derive(Clone, Debug, Default)]
struct ColumnMap {
    dance: Option<Dance>,
    judge: Option<String>,
    bib: Option<String>,
    is_sum: bool,
    is_round: bool,
}
enum TableType {
    Horizontal(usize),
    Vertical,
}

fn analyze_table(table: ElementRef) -> (Option<TableType>, Vec<ColumnMap>, usize) {
    let rows: Vec<_> = table.select(&SEL_TR).collect();
    if rows.is_empty() {
        return (None, Vec::new(), 0);
    }
    for (r_idx, row) in rows.iter().enumerate().take(5) {
        let mut bibs = Vec::new();
        for cell in row.select(&SEL_TD) {
            let b = cell.text().next().and_then(|s| {
                let t = s.trim();
                if let Some(caps) = RE_BIB_PARENS.captures(t) {
                    Some(caps[1].to_string())
                } else {
                    let l = t
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>();
                    if !l.is_empty() && l.len() <= 4 && t.len() == l.len() {
                        Some(l)
                    } else {
                        None
                    }
                }
            });
            if let Some(bib) = b {
                if bibs.last() != Some(&bib) {
                    bibs.push(bib);
                }
            }
        }
        if bibs.len() >= 3 {
            let mut map = vec![ColumnMap::default(); row.select(&SEL_TD).count()];
            for (i, cell) in row.select(&SEL_TD).enumerate() {
                map[i].bib = cell.text().next().and_then(|s| {
                    let t = s.trim();
                    if let Some(caps) = RE_BIB_PARENS.captures(t) {
                        Some(caps[1].to_string())
                    } else {
                        let l = t
                            .chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect::<String>();
                        if !l.is_empty() && l.len() <= 4 && t.len() == l.len() {
                            Some(l)
                        } else {
                            None
                        }
                    }
                });
            }
            for row in rows.iter().take(r_idx) {
                if let Some(d) = crate::i18n::parse_dances(&txt(row)).first() {
                    for c in &mut map {
                        if c.bib.is_some() {
                            c.dance = Some(*d);
                        }
                    }
                }
            }
            return (Some(TableType::Vertical), map, r_idx + 1);
        }
    }
    for c_idx in 0..5 {
        let mut bibs = Vec::new();
        for row in rows.iter().take(40) {
            let b = row
                .select(&SEL_TD)
                .nth(c_idx)
                .and_then(|c| c.text().next())
                .and_then(|s| {
                    let t = s.trim();
                    if crate::i18n::is_bib_column_marker(t) || crate::i18n::is_rank_column_marker(t)
                    {
                        return None;
                    }
                    if let Some(caps) = RE_BIB_PARENS.captures(t) {
                        Some(caps[1].to_string())
                    } else {
                        let l = t
                            .chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect::<String>();
                        if !l.is_empty() && l.len() <= 4 && t.len() == l.len() {
                            Some(l)
                        } else {
                            None
                        }
                    }
                });
            if let Some(bib) = b {
                if bibs.last() != Some(&bib) {
                    bibs.push(bib);
                }
            }
        }
        if bibs.len() >= 2
            && (bibs.iter().any(|b| b.len() >= 3)
                || rows
                    .iter()
                    .take(5)
                    .any(|r| crate::i18n::is_participant_marker(&txt(r))))
        {
            let start = rows
                .iter()
                .position(|r| {
                    r.select(&SEL_TD).nth(c_idx).is_some_and(|c| {
                        let t = txt(&c);
                        t.contains(&bibs[0])
                            || (t.len() <= 4
                                && t.chars().all(|ch| ch.is_ascii_digit())
                                && t == bibs[0])
                    })
                })
                .unwrap_or(0);
            let mut map = vec![ColumnMap::default(); 100];
            let mut reserved = vec![0; 100];
            for row in rows.iter().take(start) {
                let mut col_ptr = 0;
                for cell in row.select(&SEL_TD) {
                    while col_ptr < 100 && reserved[col_ptr] > 0 {
                        col_ptr += 1;
                    }
                    if col_ptr >= 100 {
                        break;
                    }
                    let span = cell
                        .value()
                        .attr("colspan")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(1);
                    let rspan = cell
                        .value()
                        .attr("rowspan")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(1);
                    let t = txt(&cell);
                    let ds = crate::i18n::parse_dances(&t);
                    let j = if !t.is_empty()
                        && t.chars().next().unwrap().is_ascii_uppercase()
                        && (t.len() == 1 || !t.chars().nth(1).unwrap().is_ascii_lowercase())
                        && ds.is_empty()
                        && !crate::i18n::is_rank_column_marker(&t)
                    {
                        let code = t
                            .chars()
                            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                            .collect::<String>();
                        if code.is_empty() {
                            None
                        } else {
                            Some(code)
                        }
                    } else {
                        None
                    };
                    for k in 0..span {
                        let idx = col_ptr + k;
                        if idx < 100 {
                            if !ds.is_empty() {
                                map[idx].dance = Some(ds[0]);
                            }
                            if let Some(ref ref_j) = j {
                                map[idx].judge = Some(ref_j.clone());
                            }
                            if crate::i18n::is_sum_column_marker(&t) {
                                map[idx].is_sum = true;
                            }
                            if crate::i18n::is_round_column_marker(&t) {
                                map[idx].is_round = true;
                            }
                            if rspan > 1 {
                                reserved[idx] = rspan;
                            }
                        }
                    }
                    col_ptr += span;
                }
                for r in reserved.iter_mut() {
                    if *r > 0 {
                        *r -= 1;
                    }
                }
            }
            return (Some(TableType::Horizontal(c_idx)), map, start);
        }
    }
    (None, Vec::new(), 0)
}

pub fn extract_participants(html: &str) -> Vec<Participant> {
    Html::parse_document(html)
        .select(&SEL_TR)
        .filter_map(|r| parse_participant_row(r).ok())
        .collect()
}

pub fn extract_round_data(html: &str, dances: &[Dance], officials: &Officials) -> Vec<Round> {
    let doc = Html::parse_document(html);
    let mut rounds: Vec<Round> = Vec::new();
    let global_name = doc
        .select(&Selector::parse(SELECTOR_ROUND_NAME).unwrap())
        .find_map(|e| crate::i18n::parse_round_name(&txt(&e)));

    for table in doc.select(&Selector::parse(SELECTOR_TABLE).unwrap()) {
        let (stype, map, start) = analyze_table(table);
        let rows: Vec<_> = table.select(&SEL_TR).collect();
        match stype {
            Some(TableType::Horizontal(bc)) => {
                let mut s_ids = Vec::new();
                for row in rows.iter().skip(start) {
                    if let Some(c) = map
                        .iter()
                        .position(|m| m.is_round)
                        .and_then(|i| row.select(&SEL_TD).nth(i))
                    {
                        for p in c
                            .inner_html()
                            .to_lowercase()
                            .replace("<br>", "\n")
                            .replace("<br/>", "\n")
                            .split('\n')
                        {
                            let id = clean(&p.replace("&nbsp;", ""));
                            if !id.is_empty() && !s_ids.contains(&id) {
                                s_ids.push(id);
                            }
                        }
                    }
                }
                s_ids.sort_by(|a, b| {
                    let fa = a.starts_with('f');
                    let fb = b.starts_with('f');
                    if fa && !fb {
                        std::cmp::Ordering::Greater
                    } else if !fa && fb {
                        std::cmp::Ordering::Less
                    } else {
                        a.chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0)
                            .cmp(
                                &b.chars()
                                    .filter(|c| c.is_ascii_digit())
                                    .collect::<String>()
                                    .parse::<u32>()
                                    .unwrap_or(0),
                            )
                    }
                });

                for row in rows.iter().skip(start) {
                    let cells = row.select(&SEL_TD).collect::<Vec<_>>();
                    let bib_text = row
                        .select(&SEL_TD)
                        .nth(bc)
                        .map(|c| txt(&c))
                        .unwrap_or_default();
                    let bib_c = RE_BIB_PARENS
                        .captures(&bib_text)
                        .map(|c| c[1].to_string())
                        .unwrap_or(bib_text);
                    if bib_c.is_empty() || !bib_c.chars().all(|c| c.is_ascii_digit()) {
                        continue;
                    }

                    let r_cell = cells.iter().find(|c| {
                        let cl = c.value().attr("class").unwrap_or("");
                        crate::i18n::is_result_cell_class(cl)
                    });
                    let r_idxs: Vec<usize> = r_cell
                        .map(|c| {
                            c.inner_html()
                                .to_lowercase()
                                .replace("<br>", "\n")
                                .replace("<br/>", "\n")
                                .split('\n')
                                .filter_map(|p| {
                                    let t = clean(&p.replace("&nbsp;", ""));
                                    s_ids.iter().position(|sid| sid == &t)
                                })
                                .collect()
                        })
                        .unwrap_or_else(|| vec![0]);

                    let mut col_ptr = 0;
                    for cell in &cells {
                        let span = cell
                            .value()
                            .attr("colspan")
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(1);
                        if let Some(col) = map.get(col_ptr) {
                            if !col.is_sum && (col.dance.is_some() || col.judge.is_some()) {
                                for (l_idx, val) in cell
                                    .inner_html()
                                    .to_lowercase()
                                    .replace("<br>", "\n")
                                    .replace("<br/>", "\n")
                                    .split('\n')
                                    .enumerate()
                                {
                                    let val_stripped = clean(
                                        &scraper::Html::parse_fragment(val)
                                            .root_element()
                                            .text()
                                            .collect::<String>()
                                            .replace("&nbsp;", ""),
                                    );
                                    if val_stripped.is_empty() || val_stripped == "-" {
                                        continue;
                                    }
                                    let &r_pos =
                                        r_idxs.get(l_idx).unwrap_or(r_idxs.first().unwrap_or(&0));
                                    let r_name = if !s_ids.is_empty() {
                                        crate::i18n::get_round_name_from_id(&s_ids[r_pos])
                                    } else {
                                        global_name
                                            .clone()
                                            .unwrap_or_else(crate::i18n::get_result_table_name)
                                    };
                                    let r = match rounds.iter_mut().find(|r| r.name == r_name) {
                                        Some(r) => r,
                                        None => {
                                            rounds.push(Round {
                                                name: r_name,
                                                order: r_pos as u32,
                                                dances: dances.to_vec(),
                                                data: RoundData::Marking {
                                                    marking_crosses: BTreeMap::new(),
                                                },
                                            });
                                            rounds.last_mut().unwrap()
                                        }
                                    };
                                    if let (Some(d), Some(ref j)) = (col.dance, &col.judge) {
                                        if let Ok(rank) = val_stripped.parse::<u32>() {
                                            if let RoundData::Marking { .. } = r.data {
                                                r.data = RoundData::DTV {
                                                    dtv_ranks: BTreeMap::new(),
                                                };
                                            }
                                            if let RoundData::DTV { ref mut dtv_ranks } = r.data {
                                                dtv_ranks
                                                    .entry(j.clone())
                                                    .or_default()
                                                    .entry(bib_c.clone())
                                                    .or_default()
                                                    .insert(d, rank);
                                            }
                                        } else if val_stripped.contains('x') {
                                            if let RoundData::Marking {
                                                ref mut marking_crosses,
                                            } = r.data
                                            {
                                                marking_crosses
                                                    .entry(j.clone())
                                                    .or_default()
                                                    .entry(bib_c.clone())
                                                    .or_default()
                                                    .insert(d, true);
                                            }
                                        }
                                    } else if let Some(dance) = col.dance {
                                        if val_stripped.len() > 1
                                            && val_stripped.chars().all(|c| c.is_ascii_digit())
                                        {
                                            if let RoundData::Marking { .. } = r.data {
                                                r.data = RoundData::DTV {
                                                    dtv_ranks: BTreeMap::new(),
                                                };
                                            }
                                            if let RoundData::DTV { ref mut dtv_ranks } = r.data {
                                                for (j_idx, mark) in
                                                    val_stripped.chars().enumerate()
                                                {
                                                    if let Some(j_obj) = officials.judges.get(j_idx)
                                                    {
                                                        if let Some(m) = mark.to_digit(10) {
                                                            dtv_ranks
                                                                .entry(j_obj.code.clone())
                                                                .or_default()
                                                                .entry(bib_c.clone())
                                                                .or_default()
                                                                .insert(dance, m);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        col_ptr += span;
                    }
                }
            }
            Some(TableType::Vertical) => {
                let (mut curr_name, mut curr_order) = (
                    global_name
                        .clone()
                        .unwrap_or_else(|| crate::i18n::get_round_name_from_pos(1)),
                    1,
                );
                for row in rows.iter().skip(start) {
                    let text = txt(row).to_lowercase();
                    if crate::i18n::is_qualification_marker(&text) {
                        if text.contains("2nd") || text.contains("2.") {
                            curr_order = 2;
                        } else if text.contains("3rd") || text.contains("3.") {
                            curr_order = 3;
                        } else if text.contains("4th") || text.contains("4.") {
                            curr_order = 4;
                        } else if text.contains("1st") || text.contains("1.") {
                            curr_order = 1;
                        }
                        if let Some(n) = crate::i18n::parse_round_name(&text) {
                            if n.contains(ROUND_NAME_ZWISCHENRUNDE)
                                || n == crate::i18n::get_round_name_from_pos(1)
                            {
                                curr_name =
                                    crate::i18n::get_round_name_from_pos(curr_order as usize);
                            } else {
                                curr_name = n;
                            }
                        }
                        continue;
                    }
                    let mut js = Vec::new();
                    if let Some(c) = row.select(&SEL_TD).next() {
                        for line in c
                            .inner_html()
                            .to_lowercase()
                            .replace("<br>", "\n")
                            .replace("<br/>", "\n")
                            .split('\n')
                        {
                            let t = clean(&line.replace("&nbsp;", ""));
                            if let Some(pos) = t.find(')') {
                                let code = t[..pos].trim().to_uppercase();
                                if !code.is_empty()
                                    && code
                                        .chars()
                                        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
                                {
                                    js.push(code);
                                }
                            }
                        }
                    }
                    if js.is_empty() {
                        continue;
                    }
                    let r = match rounds.iter_mut().find(|r| r.name == curr_name) {
                        Some(r) => r,
                        None => {
                            rounds.push(Round {
                                name: curr_name.clone(),
                                order: curr_order,
                                dances: dances.to_vec(),
                                data: RoundData::Marking {
                                    marking_crosses: BTreeMap::new(),
                                },
                            });
                            rounds.last_mut().unwrap()
                        }
                    };
                    for (i, cell) in row.select(&SEL_TD).enumerate() {
                        let bib = match map.get(i).and_then(|m| m.bib.as_ref()) {
                            Some(b) => b,
                            None => continue,
                        };
                        let dance = map[i]
                            .dance
                            .or_else(|| dances.first().cloned())
                            .unwrap_or(Dance::Samba);
                        for (l_idx, val) in cell
                            .inner_html()
                            .to_lowercase()
                            .replace("<br>", "\n")
                            .replace("<br/>", "\n")
                            .split('\n')
                            .enumerate()
                        {
                            let val_stripped = clean(
                                &scraper::Html::parse_fragment(val)
                                    .root_element()
                                    .text()
                                    .collect::<String>()
                                    .replace("&nbsp;", ""),
                            );
                            if val_stripped.is_empty() || val_stripped == "-" {
                                continue;
                            }
                            if let Some(j) = js.get(l_idx) {
                                if let Ok(rank) = val_stripped.parse::<u32>() {
                                    if let RoundData::Marking { .. } = r.data {
                                        r.data = RoundData::DTV {
                                            dtv_ranks: BTreeMap::new(),
                                        };
                                    }
                                    if let RoundData::DTV { ref mut dtv_ranks } = r.data {
                                        dtv_ranks
                                            .entry(j.clone())
                                            .or_default()
                                            .entry(bib.clone())
                                            .or_default()
                                            .insert(dance, rank);
                                    }
                                } else if val_stripped.contains('x')
                                    || (val_stripped.chars().all(|c| c.is_ascii_digit())
                                        && val_stripped.len() == 1)
                                {
                                    if let RoundData::Marking {
                                        ref mut marking_crosses,
                                    } = r.data
                                    {
                                        marking_crosses
                                            .entry(j.clone())
                                            .or_default()
                                            .entry(bib.clone())
                                            .or_default()
                                            .insert(dance, true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    rounds.sort_by_key(|r| r.order);
    rounds.retain(|r| match &r.data {
        RoundData::DTV { dtv_ranks } => !dtv_ranks.is_empty(),
        RoundData::Marking { marking_crosses } => !marking_crosses.is_empty(),
        RoundData::WDSF { wdsf_scores } => !wdsf_scores.is_empty(),
    });
    rounds
}

fn parse_wdsf_scores(
    html: &str,
    dances: &[Dance],
) -> BTreeMap<String, BTreeMap<String, BTreeMap<Dance, WDSFScore>>> {
    let mut res = BTreeMap::new();
    let doc = Html::parse_document(html);
    for table in doc.select(&Selector::parse(SELECTOR_TABLE).unwrap()) {
        let (stype, map, start) = analyze_table(table);
        if stype.is_none() {
            continue;
        }
        for row in table.select(&SEL_TR).skip(start) {
            let cells: Vec<_> = row.select(&SEL_TD).collect();
            let bib_text = match stype {
                Some(TableType::Horizontal(bc)) => row
                    .select(&SEL_TD)
                    .nth(bc)
                    .map(|c| txt(&c))
                    .unwrap_or_default(),
                _ => String::new(),
            };
            let bib_c = RE_BIB_PARENS
                .captures(&bib_text)
                .map(|c| c[1].to_string())
                .unwrap_or(bib_text);
            for (i, cell) in cells.iter().enumerate() {
                if let Some(col) = map.get(i) {
                    if let (Some(dance), Some(ref j)) =
                        (col.dance.or_else(|| dances.first().cloned()), &col.judge)
                    {
                        for line in cell
                            .inner_html()
                            .to_lowercase()
                            .replace("<br>", "\n")
                            .replace("<br/>", "\n")
                            .split('\n')
                        {
                            let sc: Vec<f64> = RE_SCORE
                                .find_iter(line)
                                .filter_map(|m| m.as_str().replace(',', ".").parse().ok())
                                .collect();
                            if !sc.is_empty() {
                                let s = res
                                    .entry(j.clone())
                                    .or_insert_with(BTreeMap::new)
                                    .entry(bib_c.clone())
                                    .or_insert_with(BTreeMap::new)
                                    .entry(dance)
                                    .or_insert_with(|| WDSFScore {
                                        technical_quality: 0.0,
                                        movement_to_music: 0.0,
                                        partnering_skills: 0.0,
                                        choreography: 0.0,
                                        total: 0.0,
                                    });

                                if let Some(score_type) = crate::i18n::map_wdsf_score_type(line) {
                                    match score_type {
                                        "technical_quality" => s.technical_quality = sc[0],
                                        "movement_to_music" => s.movement_to_music = sc[0],
                                        "partnering_skills" => s.partnering_skills = *sc.last().unwrap(),
                                        "choreography" => s.choreography = *sc.last().unwrap(),
                                        "total" => s.total = sc[0],
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
    res
}

pub fn extract_event_data(data_dir: &str) -> Result<Competition> {
    let dir = Path::new(data_dir);
    let erg_h = fs::read_to_string(dir.join("erg.htm")).unwrap_or_default();
    let (name, date, mut org, mut club) = parse_metadata(&erg_h);
    let title = Html::parse_document(&erg_h)
        .select(&SEL_TITLE)
        .next()
        .map(|n| n.inner_html())
        .unwrap_or_default();

    let name_for_parse = if let Some(ref n) = name {
        if n.contains(" - ") {
            n
        } else {
            &title
        }
    } else {
        &title
    };
    let mut comp = match parse_competition_from_title(name_for_parse) {
        Ok(c) => c,
        Err(_) => Competition {
            name: name.clone().unwrap_or_else(|| "TODO".into()),
            date,
            organizer: None,
            hosting_club: None,
            source_url: None,
            level: Level::S,
            age_group: AgeGroup::Adult,
            style: Style::Standard,
            dances: Vec::new(),
            min_dances: 0,
            officials: Officials {
                responsible_person: None,
                assistant: None,
                judges: Vec::new(),
            },
            participants: Vec::new(),
            rounds: Vec::new(),
        },
    };

    if let Ok(deck_h) = fs::read_to_string(dir.join("deck.htm")) {
        let (d_name, _d_date, d_org, d_club) = parse_metadata(&deck_h);
        if org.is_none() {
            org = d_org;
        }
        if club.is_none() {
            club = d_club;
        }
        if name.is_none() {
            if let Some(ref n) = d_name {
                if let Ok(c) = parse_competition_from_title(n) {
                    comp.name = c.name;
                    comp.age_group = c.age_group;
                    comp.style = c.style;
                    comp.level = c.level;
                }
            }
        }

        let doc = Html::parse_document(&deck_h);
        let mut off = Officials {
            responsible_person: None,
            assistant: None,
            judges: Vec::new(),
        };
        for row in doc.select(&SEL_TR) {
            let (role_sel, data_sel) = (
                Selector::parse(SELECTOR_OFFICIAL_ROLE).unwrap(),
                Selector::parse(SELECTOR_OFFICIAL_DATA).unwrap(),
            );
            if let (Some(r_el), Some(d_el)) =
                (row.select(&role_sel).next(), row.select(&data_sel).next())
            {
                let r = txt(&r_el).replace(':', "");
                let n = d_el
                    .select(&SEL_SPAN)
                    .next()
                    .map(|el| txt(&el))
                    .unwrap_or_default();
                let c = d_el.select(&SEL_SPAN).nth(1).map(|el| txt(&el));

                if crate::i18n::is_organizer_marker(&r) {
                    if comp.organizer.is_none() {
                        comp.organizer = Some(txt(&d_el));
                    }
                } else if crate::i18n::is_hosting_club_marker(&r) {
                    if comp.hosting_club.is_none() {
                        comp.hosting_club = Some(txt(&d_el));
                    }
                } else if let Some(m) = crate::i18n::map_role(&r) {
                    let mem = CommitteeMember { name: n, club: c };
                    if m == "responsible_person" {
                        off.responsible_person = Some(mem);
                    } else {
                        off.assistant = Some(mem);
                    }
                } else if (r.len() <= 3 || r.chars().all(|ch| ch.is_ascii_uppercase()))
                    && !n.is_empty()
                {
                    off.judges.push(Judge {
                        code: r,
                        name: n,
                        club: c,
                    });
                }
            }
        }
        comp.officials = off;
    }

    if org.is_none() || club.is_none() {
        if let Ok(index_h) = fs::read_to_string(dir.join("index.htm")) {
            let (_, _, i_org, i_club) = parse_metadata(&index_h);
            if org.is_none() {
                org = i_org;
            }
            if club.is_none() {
                club = i_club;
            }
        }
    }

    if comp.organizer.is_none() {
        comp.organizer = org;
    }
    if comp.hosting_club.is_none() {
        comp.hosting_club = club;
    }
    comp.participants = extract_participants(&erg_h);
    comp.participants.retain(|p| {
        p.bib_number != 0
            && !crate::i18n::is_participant_marker(&p.name_one)
            && !crate::i18n::is_rank_column_marker(&p.name_one)
    });

    let mut scoring_h = fs::read_to_string(dir.join("tabges.htm")).unwrap_or_default();
    if scoring_h.is_empty() {
        scoring_h = fs::read_to_string(dir.join("ergwert.htm")).unwrap_or_default();
    }
    let has_scoring_file = !scoring_h.is_empty();
    if !has_scoring_file {
        scoring_h = erg_h.clone();
    }

    if comp.dances.is_empty() {
        comp.dances = crate::i18n::parse_dances(&scoring_h);
    }
    let mut rounds = extract_round_data(&scoring_h, &comp.dances, &comp.officials);
    if rounds.is_empty() && has_scoring_file {
        rounds = extract_round_data(&erg_h, &comp.dances, &comp.officials);
    }
    // Filter dances based on actually parsed results
    let mut actual_dances = std::collections::HashSet::new();
    for r in &rounds {
        match &r.data {
            RoundData::DTV { dtv_ranks } => {
                for j_marks in dtv_ranks.values() {
                    for p_marks in j_marks.values() {
                        for d in p_marks.keys() {
                            actual_dances.insert(*d);
                        }
                    }
                }
            }
            RoundData::Marking { marking_crosses } => {
                for j_marks in marking_crosses.values() {
                    for p_marks in j_marks.values() {
                        for d in p_marks.keys() {
                            actual_dances.insert(*d);
                        }
                    }
                }
            }
            RoundData::WDSF { wdsf_scores } => {
                for j_marks in wdsf_scores.values() {
                    for p_marks in j_marks.values() {
                        for d in p_marks.keys() {
                            actual_dances.insert(*d);
                        }
                    }
                }
            }
        }
    }
    if !actual_dances.is_empty() {
        comp.dances.retain(|d| actual_dances.contains(d));
        for r in &mut rounds {
            r.dances.retain(|d| actual_dances.contains(d));
        }
    }
    comp.rounds = rounds;

    if scoring_h.contains("TQ") || scoring_h.contains("MM") {
        let scores = parse_wdsf_scores(&scoring_h, &comp.dances);
        if !scores.is_empty() {
            if let Some(r) = comp
                .rounds
                .iter_mut()
                .find(|r| crate::i18n::is_final_round(&r.name))
            {
                r.data = RoundData::WDSF {
                    wdsf_scores: scores,
                };
            } else if let Some(r) = comp.rounds.last_mut() {
                r.data = RoundData::WDSF {
                    wdsf_scores: scores,
                };
            }
        }
    }
    comp.rounds.sort_by_key(|r| r.order);

    if comp.source_url.is_none() {
        if let Ok(index_h) = fs::read_to_string(dir.join("index.htm")) {
            let doc = Html::parse_document(&index_h);
            if let Some(link) = doc
                .select(&Selector::parse(SELECTOR_CANONICAL).unwrap())
                .next()
            {
                comp.source_url = link.value().attr("href").map(|s| s.to_string());
            }
        }
    }

    Ok(comp)
}

pub fn parse_competition_from_title(title: &str) -> Result<Competition, ParsingError> {
    let (mut ag, mut st, mut lv) = (None, None, None);
    let title_clean = crate::i18n::clean_competition_title(title);

    let up = title_clean.to_uppercase();
    let mut age_keys = crate::i18n::age_group_keys();
    age_keys.sort_by_key(|k| k.len());
    age_keys.reverse();
    for k in age_keys {
        if up.contains(&k.to_uppercase()) {
            ag = crate::i18n::map_age_group(k);
            break;
        }
    }
    for k in crate::i18n::style_keys() {
        if up.contains(&k.to_uppercase()) {
            st = crate::i18n::map_discipline(k);
            break;
        }
    }
    for l in ["S", "A", "B", "C", "D", "E"] {
        if up.contains(&format!(" {} ", l)) || up.ends_with(&format!(" {}", l)) {
            lv = crate::i18n::parse_level(l);
            break;
        }
    }
    if lv.is_none() && crate::i18n::is_level_s_marker(&up) {
        lv = Some(Level::S);
    }
    let date = parse_date(title).unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    if ag.is_none() || st.is_none() || lv.is_none() {
        return Err(ParsingError::MissingRequiredData(title.into()));
    }
    Ok(Competition {
        name: title_clean,
        date: Some(date),
        organizer: None,
        hosting_club: None,
        source_url: None,
        level: lv.unwrap(),
        age_group: ag.unwrap(),
        style: st.unwrap(),
        dances: crate::i18n::parse_dances(title),
        min_dances: crate::i18n::get_min_dances(lv.unwrap(), date),
        officials: Officials {
            responsible_person: None,
            assistant: None,
            judges: Vec::new(),
        },
        participants: Vec::new(),
        rounds: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_participants() {
        let html = r#"<table><TR><TD class="td3r">1.</TD><TD class="td5">Jonathan Kummetz / Elisabeth Findeiß (610)<BR><i>1. TC Rot-Gold Bayreuth</i></TD></TR></table>"#;
        let p = parse_participant_row(Html::parse_document(html).select(&SEL_TR).next().unwrap())
            .unwrap();
        assert_eq!(p.bib_number, 610);
        assert_eq!(p.name_one, "Jonathan Kummetz");
    }

    #[test]
    fn test_real_wdsf_world_open_tabges() {
        let html = fs::read_to_string("tests/44-0507_wdsfworldopenlatadult/tabges.htm").unwrap();
        let dances = vec![
            Dance::Samba,
            Dance::ChaChaCha,
            Dance::Rumba,
            Dance::PasoDoble,
            Dance::Jive,
        ];
        let off = Officials {
            responsible_person: None,
            assistant: None,
            judges: (b'A'..=b'X')
                .map(|c| Judge {
                    code: (c as char).to_string(),
                    name: String::new(),
                    club: None,
                })
                .collect(),
        };
        let results = extract_round_data(&html, &dances, &off);
        let found = results.iter().any(|r| match &r.data {
            RoundData::Marking { marking_crosses } => marking_crosses
                .get("A")
                .map_or(false, |m| m.contains_key("284")),
            RoundData::DTV { dtv_ranks } => {
                dtv_ranks.get("A").map_or(false, |m| m.contains_key("284"))
            }
            _ => false,
        });
        assert!(
            found,
            "Could not find marks for bib 284 in any round. Rounds found: {}",
            results.len()
        );
    }
}
