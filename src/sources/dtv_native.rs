use crate::assets::*;
use crate::models::{
    CommitteeMember, Competition, Dance, IdentityType, Judge, Level, Officials, Participant, Round,
    Style,
};
use crate::sources::topturnier_table::{self, RE_BIB_PARENS};
use crate::sources::ParsingError;
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
static SEL_TABLE: LazyLock<Selector> = LazyLock::new(|| Selector::parse(SELECTOR_TABLE).unwrap());
static SEL_META_AUTHOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(SELECTOR_META_AUTHOR).unwrap());
static SEL_EVENT_HEAD: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(SELECTOR_EVENT_HEAD).unwrap());
static SEL_COMP_HEAD: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(SELECTOR_COMP_HEAD).unwrap());
static SEL_PARTICIPANT_RANK: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(SELECTOR_PARTICIPANT_RANK).unwrap());
static SEL_PARTICIPANT_DATA: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(SELECTOR_PARTICIPANT_DATA).unwrap());
static SEL_OFFICIAL_ROLE: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(SELECTOR_OFFICIAL_ROLE).unwrap());
static SEL_OFFICIAL_DATA: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(SELECTOR_OFFICIAL_DATA).unwrap());
static SEL_CANONICAL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(SELECTOR_CANONICAL).unwrap());
static RE_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN_DATE).unwrap());
static RE_RANK: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN_RANK).unwrap());

// --- Utilities ---
fn txt(el: &ElementRef) -> String {
    el.text().collect::<String>().trim().to_string()
}

/// TopTurnier puts the club on the lines after the name: "Name\nClub".
fn split_name_club(s: &str) -> (String, Option<String>) {
    match s.split_once('\n') {
        Some((n, c)) => (
            n.trim().to_string(),
            Some(c.replace('\n', " ").trim().to_string()),
        ),
        None => (s.to_string(), None),
    }
}

/// "A / B" is a couple, anything else a solo dancer.
fn split_couple(name: String) -> (IdentityType, String, Option<String>) {
    if name.contains(" / ") {
        let p: Vec<_> = name.split(" / ").collect();
        (
            IdentityType::Couple,
            p[0].trim().into(),
            Some(p[1].trim().into()),
        )
    } else {
        (IdentityType::Solo, name, None)
    }
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
    if let Some(meta_author) = doc.select(&SEL_META_AUTHOR).next() {
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
    let name = doc.select(&SEL_EVENT_HEAD).next().map(|el| txt(&el));
    let date = doc
        .select(&SEL_COMP_HEAD)
        .next()
        .map(|el| txt(&el))
        .and_then(|d| parse_date(&d));
    (name, date, org, club)
}

fn parse_participant_row(row: ElementRef) -> Result<Participant, ParsingError> {
    let rank = row.select(&SEL_PARTICIPANT_RANK).next().and_then(|el| {
        RE_RANK
            .captures(&txt(&el))
            .and_then(|c| c[1].parse::<u32>().ok())
    });
    let data_cells: Vec<_> = row.select(&SEL_PARTICIPANT_DATA).collect();
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
            if !crate::i18n::is_bib_column_marker(&next_cell_text)
                && !next_cell_text.chars().all(|c| c.is_ascii_digit())
            {
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
            (name, club) = split_name_club(&name);
        }
    }
    if bib == 0 {
        return Err(ParsingError::MissingRequiredData("Bib".into()));
    }
    let (it, n1, mut n2) = split_couple(name);
    // The club can also trail the second partner's name.
    if let Some(n2_val) = n2.as_mut().filter(|n| n.contains('\n')) {
        let (n, c) = split_name_club(n2_val);
        *n2_val = n;
        if club.is_none() {
            club = c;
        }
    }
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
        (
            crate::models::RoundData::DTV { dtv_ranks: e_map },
            crate::models::RoundData::DTV { dtv_ranks: n_map },
        ) => {
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
        (
            crate::models::RoundData::WDSF { wdsf_scores: e_map },
            crate::models::RoundData::WDSF { wdsf_scores: n_map },
        ) => {
            for (judge, n_bib_map) in n_map {
                let e_bib_map = e_map.entry(judge).or_default();
                for (bib, n_dance_map) in n_bib_map {
                    let e_dance_map = e_bib_map.entry(bib).or_default();
                    for (dance, n_score) in n_dance_map {
                        let e_score = e_dance_map.entry(dance).or_default();
                        if n_score.technical_quality > 0.0 {
                            e_score.technical_quality = n_score.technical_quality;
                        }
                        if n_score.movement_to_music > 0.0 {
                            e_score.movement_to_music = n_score.movement_to_music;
                        }
                        if n_score.partnering_skills > 0.0 {
                            e_score.partnering_skills = n_score.partnering_skills;
                        }
                        if n_score.choreography > 0.0 {
                            e_score.choreography = n_score.choreography;
                        }
                        e_score.total = e_score.technical_quality
                            + e_score.movement_to_music
                            + e_score.partnering_skills
                            + e_score.choreography;
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
    let ergwert_doc = Html::parse_document(&ergwert_h);
    let (name, date, mut org, mut club) = parse_metadata(&ergwert_h);
    let title = ergwert_doc
        .select(&SEL_TITLE)
        .next()
        .map(|n| n.inner_html())
        .unwrap_or_default();
    let deck_h = fs::read_to_string(dir.join("deck.htm")).ok();
    let (d_name, d_org, d_club) = match deck_h.as_deref().map(parse_metadata) {
        Some((n, _, o, c)) => (n, o, c),
        None => (None, None, None),
    };

    let name_for_parse = match name {
        Some(ref n) if n.contains(" - ") => n,
        _ => &title,
    };
    // deck.htm repeats the competition name, so it is the fallback before giving up.
    let mut comp = parse_competition_from_title(name_for_parse)
        .or_else(|e| {
            d_name
                .as_deref()
                .map_or(Err(e), parse_competition_from_title)
        })
        .map_err(|_| {
            ParsingError::MissingRequiredData(format!(
                "cannot determine level/age group/style from title {:?}",
                title
            ))
        })?;
    // Prefer the explicit .comphead date over one embedded in the title.
    if let Some(d) = date {
        comp.date = Some(d);
        comp.min_dances = crate::i18n::get_min_dances(comp.level, d);
    }

    if org.is_none() {
        org = d_org;
    }
    if club.is_none() {
        club = d_club;
    }

    if let Some(deck_h) = &deck_h {
        let doc = Html::parse_document(deck_h);
        let mut off = Officials {
            responsible_person: None,
            assistant: None,
            judges: Vec::new(),
        };
        for row in doc.select(&SEL_TR) {
            if let (Some(r_el), Some(d_el)) = (
                row.select(&SEL_OFFICIAL_ROLE).next(),
                row.select(&SEL_OFFICIAL_DATA).next(),
            ) {
                let r_raw = txt(&r_el);
                let r = r_raw.trim_end_matches(':').trim().to_string();

                let spans: Vec<_> = d_el.select(&SEL_SPAN).collect();
                let (n, c) = if spans.len() >= 2 {
                    (txt(&spans[0]), Some(txt(&spans[1])))
                } else if spans.len() == 1 {
                    (txt(&spans[0]), None)
                } else {
                    split_name_club(&topturnier_table::extract_text(d_el))
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
                } else if (r.len() <= 3
                    || r.chars()
                        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()))
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
        for table in ergwert_doc.select(&SEL_TABLE) {
            let grid = topturnier_table::TableGrid::from_element(table);
            let col_types = topturnier_table::identify_columns(&grid);
            if let Some(p_idx) = col_types
                .iter()
                .position(|t| matches!(t, topturnier_table::ColumnType::Participant))
            {
                let bib_idx = col_types
                    .iter()
                    .position(|t| matches!(t, topturnier_table::ColumnType::Bib));
                let rank_idx = col_types
                    .iter()
                    .position(|t| matches!(t, topturnier_table::ColumnType::Rank));

                for r in 1..grid.height {
                    let name = &grid.rows[r][p_idx];
                    if name.is_empty() || crate::i18n::is_participant_marker(name) {
                        continue;
                    }

                    let bib = if let Some(idx) = bib_idx {
                        grid.rows[r][idx].parse().ok()
                    } else {
                        RE_BIB_PARENS.captures(name).and_then(|c| c[1].parse().ok())
                    };

                    let bib = match bib {
                        Some(b) => b,
                        None => continue,
                    };

                    let rank = rank_idx.and_then(|idx| {
                        RE_RANK
                            .captures(&grid.rows[r][idx])
                            .and_then(|c| c[1].parse().ok())
                    });

                    let (name_clean, club) = split_name_club(name);
                    let (it, n1, n2) = split_couple(name_clean);

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
        comp.participants.dedup_by_key(|p| p.bib_number);
    }

    comp.participants.retain(|p| {
        p.bib_number != 0
            && !crate::i18n::is_participant_marker(&p.name_one)
            && !crate::i18n::is_rank_column_marker(&p.name_one)
    });

    let full_style_dances = if comp.style == Style::Standard {
        STANDARD_DANCES
    } else {
        LATIN_DANCES
    };

    let mut all_rounds: Vec<Round> = Vec::new();
    let mut found_dances = std::collections::HashSet::new();

    for table in ergwert_doc.select(&SEL_TABLE) {
        let grid = topturnier_table::TableGrid::from_element(table);
        let col_types = topturnier_table::identify_columns(&grid);
        let intermediate = topturnier_table::extract_data(&grid, &col_types);

        let mut table_dances = Vec::new();
        for ct in &col_types {
            if let topturnier_table::ColumnType::Mark { dance, .. }
            | topturnier_table::ColumnType::Dance(dance) = ct
            {
                if !table_dances.contains(dance) {
                    table_dances.push(*dance);
                }
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

    let mut all_dances_vec: Vec<_> = found_dances
        .iter()
        .cloned()
        .filter(|d| full_style_dances.contains(d))
        .collect();
    all_dances_vec.sort();

    // Filter dances that actually appear in scoring rounds.
    let scoring_dances: std::collections::HashSet<Dance> = all_rounds
        .iter()
        .flat_map(|r| {
            let mut ds = std::collections::HashSet::new();
            match &r.data {
                crate::models::RoundData::Marking { marking_crosses } => {
                    for j_map in marking_crosses.values() {
                        for p_map in j_map.values() {
                            for d in p_map.keys() {
                                ds.insert(*d);
                            }
                        }
                    }
                }
                crate::models::RoundData::DTV { dtv_ranks } => {
                    for j_map in dtv_ranks.values() {
                        for p_map in j_map.values() {
                            for d in p_map.keys() {
                                ds.insert(*d);
                            }
                        }
                    }
                }
                crate::models::RoundData::WDSF { wdsf_scores } => {
                    for j_map in wdsf_scores.values() {
                        for p_map in j_map.values() {
                            for d in p_map.keys() {
                                ds.insert(*d);
                            }
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
            if let Some(link) = doc.select(&SEL_CANONICAL).next() {
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
    let (Some(ag), Some(st), Some(lv)) = (ag, st, lv) else {
        return Err(ParsingError::MissingRequiredData(title.into()));
    };
    let date = parse_date(title);
    // Without a date assume the pre-2026 rule; extract_event_data recomputes once a date is known.
    let min_dances = crate::i18n::get_min_dances(
        lv,
        date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
    );
    Ok(Competition {
        name: title_clean,
        date,
        organizer: None,
        hosting_club: None,
        source_url: None,
        level: lv,
        age_group: ag,
        style: st,
        dances: crate::i18n::parse_dances(title),
        min_dances,
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
