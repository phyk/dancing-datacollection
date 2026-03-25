use crate::assets::*;
use crate::models::{
    AgeGroup, CommitteeMember, Competition, Dance, IdentityType, Judge, Level, Officials,
    Participant, Round, Style,
};
use crate::sources::ParsingError;
use crate::sources::topturnier_table;
use anyhow::Result;
use chrono::NaiveDate;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

// --- Selectors & Regex ---
static SEL_TR: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_TR).unwrap());
static SEL_TD: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_TD).unwrap());
static SEL_SPAN: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_SPAN).unwrap());
static SEL_I: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_I).unwrap());
static SEL_TITLE: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_TITLE).unwrap());
static RE_BIB_PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN_BIB_PARENS).unwrap());
static RE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN_DATE).unwrap());
static RE_RANK: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN_RANK).unwrap());

// --- Utilities ---
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
        .and_then(|el| {
            RE_RANK
                .captures(&txt(&el))
                .and_then(|c| c[1].parse::<u32>().ok())
        });
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
        let mut name_bib = full;
        if let Some(ref c) = club {
            name_bib = name_bib.replace(c, "").trim().to_string();
        } else if data_cells.len() > 1 {
            let next_cell_text = txt(&data_cells[1]);
            if !crate::i18n::is_bib_column_marker(&next_cell_text) && !next_cell_text.chars().all(|c| c.is_ascii_digit()) {
                club = Some(next_cell_text);
            }
        }

        if let Some(caps) = RE_BIB_PARENS.captures(&name_bib) {
            bib = caps[1].parse().unwrap_or(0);
            name = RE_BIB_PARENS.replace_all(&name_bib, "").trim().to_string();
        } else {
            name = name_bib;
        }

        if club.is_none() && name.contains('\n') {
            let p: Vec<_> = name.splitn(2, '\n').map(|s| s.to_string()).collect();
            name = p[0].trim().to_string();
            club = Some(p[1].replace('\n', " ").trim().to_string());
        }
    }
    if bib == 0 {
        return Err(ParsingError::MissingRequiredData("Bib".into()));
    }
    let (it, n1, n2) = if name.contains(" / ") {
        let p: Vec<_> = name.split(" / ").collect();
        let mut n2_val = p[1].trim().to_string();
        if n2_val.contains('\n') {
            let parts: Vec<_> = n2_val.splitn(2, '\n').map(|s| s.to_string()).collect();
            n2_val = parts[0].trim().to_string();
            if club.is_none() {
                club = Some(parts[1].replace('\n', " ").trim().to_string());
            }
        }
        (IdentityType::Couple, p[0].trim().into(), Some(n2_val))
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

pub fn extract_participants(html: &str) -> Vec<Participant> {
    Html::parse_document(html)
        .select(&SEL_TR)
        .filter_map(|r| parse_participant_row(r).ok())
        .collect()
}

fn merge_round_data(existing: &mut crate::models::RoundData, new: crate::models::RoundData) {
    match (existing, new) {
        (
            crate::models::RoundData::Marking {
                marking_crosses: e_map,
            },
            crate::models::RoundData::Marking {
                marking_crosses: n_map,
            },
        ) => {
            for (judge, n_bib_map) in n_map {
                let e_bib_map = e_map.entry(judge).or_default();
                for (bib, n_dance_map) in n_bib_map {
                    let e_dance_map = e_bib_map.entry(bib).or_default();
                    // Merge dance by dance, prioritizing existing if they are more detailed?
                    // Or just additive.
                    for (dance, val) in n_dance_map {
                        e_dance_map.entry(dance).or_insert(val);
                    }
                }
            }
        }
        (crate::models::RoundData::DTV { dtv_ranks: e_map }, crate::models::RoundData::DTV { dtv_ranks: n_map }) => {
            for (judge, n_bib_map) in n_map {
                let e_bib_map = e_map.entry(judge).or_default();
                for (bib, n_dance_map) in n_bib_map {
                    let e_dance_map = e_bib_map.entry(bib).or_default();
                    for (dance, val) in n_dance_map {
                        e_dance_map.entry(dance).or_insert(val);
                    }
                }
            }
        }
        (crate::models::RoundData::WDSF { wdsf_scores: e_map }, crate::models::RoundData::WDSF { wdsf_scores: n_map }) => {
            for (judge, n_bib_map) in n_map {
                let e_bib_map = e_map.entry(judge).or_default();
                for (bib, n_dance_map) in n_bib_map {
                    let e_dance_map = e_bib_map.entry(bib).or_default();
                    if n_dance_map.len() > e_dance_map.len() {
                        *e_dance_map = n_dance_map;
                    }
                }
            }
        }
        (e, n) => {
            if n.count_entries() > e.count_entries() {
                *e = n;
            }
        }
    }
}

pub fn extract_event_data(data_dir: &str) -> Result<Competition> {
    let dir = Path::new(data_dir);
    let ergwert_h = fs::read_to_string(dir.join("ergwert.htm")).unwrap_or_default();
    let (name, date, mut org, mut club) = parse_metadata(&ergwert_h);
    let title = Html::parse_document(&ergwert_h)
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
                let r_raw = txt(&r_el);
                let r = r_raw.trim_end_matches(':').trim().to_string();

                let spans: Vec<_> = d_el.select(&SEL_SPAN).collect();
                let (n, c) = if spans.len() >= 2 {
                    (txt(&spans[0]), Some(txt(&spans[1])))
                } else if spans.len() == 1 {
                    (txt(&spans[0]), None)
                } else {
                    let full = topturnier_table::extract_text(d_el);
                    if full.contains('\n') {
                        let p: Vec<_> = full.splitn(2, '\n').collect();
                        (p[0].trim().to_string(), Some(p[1].replace('\n', " ").trim().to_string()))
                    } else {
                        (full, None)
                    }
                };

                if crate::i18n::is_organizer_marker(&r) {
                    if comp.organizer.is_none() {
                        comp.organizer = Some(txt(&d_el));
                    }
                } else if crate::i18n::is_hosting_club_marker(&r) {
                    if comp.hosting_club.is_none() {
                        comp.hosting_club = Some(txt(&d_el));
                    }
                } else if let Some(m) = crate::i18n::map_role(&r) {
                    let mem = CommitteeMember {
                        name: n.clone(),
                        club: c.clone(),
                    };
                    if m == "responsible_person" {
                        off.responsible_person = Some(mem);
                    } else {
                        // If there's already an assistant, we might want to keep the "Beisitzer" or "Chairperson"
                        // but for now we just overwrite or could append. SPEC says "Assistant".
                        if off.assistant.is_none() || r.to_lowercase().contains("beisitzer") {
                            off.assistant = Some(mem);
                        }
                    }
                } else if (r.len() <= 3 || r.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()))
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
    comp.participants = extract_participants(&ergwert_h);

    if comp.participants.is_empty() {
        // Fallback: extract participants using the structural table parser if row-based parsing fails
        let doc = Html::parse_document(&ergwert_h);
        for table in doc.select(&Selector::parse(SELECTOR_TABLE).unwrap()) {
            let grid = topturnier_table::TableGrid::from_element(table);
            if topturnier_table::identify_orientation(&grid) == topturnier_table::TableOrientation::Horizontal {
                let col_types = topturnier_table::identify_columns(&grid);
                if let Some(p_idx) = col_types.iter().position(|t| matches!(t, topturnier_table::ColumnType::Participant)) {
                    let bib_idx = col_types.iter().position(|t| matches!(t, topturnier_table::ColumnType::Bib));
                    let rank_idx = col_types.iter().position(|t| matches!(t, topturnier_table::ColumnType::Rank));

                    for r in 1..grid.height {
                         let name = &grid.rows[r][p_idx];
                         if name.is_empty() || crate::i18n::is_participant_marker(name) { continue; }

                         let bib = if let Some(idx) = bib_idx {
                             grid.rows[r][idx].parse().ok()
                         } else {
                             RE_BIB_PARENS.captures(name).and_then(|c| c[1].parse().ok())
                         };

                         let bib = match bib { Some(b) => b, None => continue };

                         let rank = rank_idx.and_then(|idx| {
                             RE_RANK.captures(&grid.rows[r][idx]).and_then(|c| c[1].parse().ok())
                         });

                         let (name_clean, club) = if name.contains('\n') {
                             let p: Vec<_> = name.splitn(2, '\n').collect();
                             (p[0].trim().to_string(), Some(p[1].replace('\n', " ").trim().to_string()))
                         } else {
                             (name.clone(), None)
                         };

                         let (it, n1, n2) = if name_clean.contains(" / ") {
                             let p: Vec<_> = name_clean.split(" / ").collect();
                             (IdentityType::Couple, p[0].trim().into(), Some(p[1].trim().into()))
                         } else {
                             (IdentityType::Solo, name_clean, None)
                         };

                         comp.participants.push(Participant {
                             identity_type: it,
                             name_one: n1,
                             bib_number: bib,
                             name_two: n2,
                             affiliation: club,
                             final_rank: rank,
                         });
                    }
                }
            }
        }
        comp.participants.dedup_by_key(|p| p.bib_number);
    }

    comp.participants.retain(|p| {
        p.bib_number != 0
            && !crate::i18n::is_participant_marker(&p.name_one)
            && !crate::i18n::is_rank_column_marker(&p.name_one)
    });

    let full_style_dances = if comp.style == Style::Standard {
        vec![
            Dance::SlowWaltz,
            Dance::Tango,
            Dance::VienneseWaltz,
            Dance::SlowFoxtrot,
            Dance::Quickstep,
        ]
    } else {
        vec![
            Dance::Samba,
            Dance::ChaChaCha,
            Dance::Rumba,
            Dance::PasoDoble,
            Dance::Jive,
        ]
    };

    let files = ["ergwert.htm"];
    let mut all_rounds: Vec<Round> = Vec::new();
    let mut found_dances = std::collections::HashSet::new();

    for f_name in files {
        if let Ok(content) = fs::read_to_string(dir.join(f_name)) {
            let mut file_dances = crate::i18n::parse_dances_no_fallback(&content);
            file_dances.retain(|d| full_style_dances.contains(d));

            let doc = Html::parse_document(&content);
            for table in doc.select(&Selector::parse(SELECTOR_TABLE).unwrap()) {
                 let grid = topturnier_table::TableGrid::from_element(table);
                 let intermediate = topturnier_table::extract_data(&grid);

                 let col_types = topturnier_table::identify_columns(&grid);
                 let mut table_dances = Vec::new();
                 for ct in col_types {
                     if let topturnier_table::ColumnType::Mark { dance, .. } = ct {
                         if !table_dances.contains(&dance) { table_dances.push(dance); }
                     } else if let topturnier_table::ColumnType::Dance(dance) = ct {
                         if !table_dances.contains(&dance) { table_dances.push(dance); }
                     }
                 }
                 table_dances.retain(|d| full_style_dances.contains(d));
                 for d in &table_dances {
                     found_dances.insert(*d);
                 }

                 let file_rounds = topturnier_table::to_rounds(intermediate, &table_dances, &comp.officials);
                 for fr in file_rounds {
                    if let Some(existing) = all_rounds.iter_mut().find(|r| r.name == fr.name) {
                        merge_round_data(&mut existing.data, fr.data);
                    } else {
                        all_rounds.push(fr);
                    }
                }
            }
        }
    }

    let mut all_dances_vec: Vec<_> = found_dances
        .iter()
        .cloned()
        .filter(|d| full_style_dances.contains(d))
        .collect();
    all_dances_vec.sort_by_key(|&d| d as u32);

    // Filter dances that actually appear in scoring rounds.
    let scoring_dances: std::collections::HashSet<Dance> = all_rounds.iter()
        .flat_map(|r| {
             let mut ds = std::collections::HashSet::new();
             match &r.data {
                 crate::models::RoundData::Marking { marking_crosses } => {
                     for j_map in marking_crosses.values() {
                         for p_map in j_map.values() {
                             for d in p_map.keys() { ds.insert(*d); }
                         }
                     }
                 }
                 crate::models::RoundData::DTV { dtv_ranks } => {
                     for j_map in dtv_ranks.values() {
                         for p_map in j_map.values() {
                             for d in p_map.keys() { ds.insert(*d); }
                         }
                     }
                 }
                 crate::models::RoundData::WDSF { wdsf_scores } => {
                     for j_map in wdsf_scores.values() {
                         for p_map in j_map.values() {
                             for d in p_map.keys() { ds.insert(*d); }
                         }
                     }
                 }
             }
             ds.into_iter()
        })
        .collect();
    all_dances_vec.retain(|d| scoring_dances.contains(d));

    comp.dances = all_dances_vec;

    for r in &mut all_rounds {
        match &mut r.data {
            crate::models::RoundData::DTV { dtv_ranks } => {
                for j_map in dtv_ranks.values_mut() {
                    for p_map in j_map.values_mut() {
                        p_map.retain(|d, _| comp.dances.contains(d));
                    }
                }
            }
            crate::models::RoundData::Marking { marking_crosses } => {
                for j_map in marking_crosses.values_mut() {
                    for p_map in j_map.values_mut() {
                        p_map.retain(|d, _| comp.dances.contains(d));
                    }
                }
            }
            _ => {}
        }
    }

    all_rounds.retain(|r| r.data.count_entries() > 0);
    all_rounds.sort_by(|a, b| {
        let a_final = crate::i18n::is_final_round(&a.name);
        let b_final = crate::i18n::is_final_round(&b.name);
        if a_final && !b_final {
            std::cmp::Ordering::Greater
        } else if !a_final && b_final {
            std::cmp::Ordering::Less
        } else {
            a.order.cmp(&b.order)
        }
    });

    comp.rounds = all_rounds;

    comp.participants.sort_by_key(|p| p.bib_number);
    comp.officials.judges.sort_by(|a, b| a.code.cmp(&b.code));

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

    #[test]
    fn test_parse_participants() {
        let html = r#"<table><TR><TD class="td3r">1.</TD><TD class="td5">Jonathan Kummetz / Elisabeth Findeiß (610)<BR><i>1. TC Rot-Gold Bayreuth</i></TD></TR></table>"#;
        let p = parse_participant_row(Html::parse_document(html).select(&SEL_TR).next().unwrap())
            .unwrap();
        assert_eq!(p.bib_number, 610);
        assert_eq!(p.name_one, "Jonathan Kummetz");
    }
}
