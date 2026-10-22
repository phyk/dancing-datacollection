//! Lookup and normalization functions over the tables in `crate::assets`
//! (age groups, styles, levels, dances, months, roles, round names, column markers),
//! plus the DTV minimum-dance-count rule (`get_min_dances`), which must be
//! updated if national regulations change.

use crate::assets::*;
use crate::models::{AgeGroup, Dance, Level, Style};
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use std::sync::LazyLock;

/// A dance together with its compiled `(alias, regex)` matchers.
type DanceMatchers = (Dance, Vec<(&'static str, Regex)>);

/// Precompiled per-alias regexes for `DANCE_ABBREVIATIONS`.
/// Each alias uses a strict match to avoid greedy matching (e.g. Walzer vs Wiener Walzer);
/// spaces inside an alias may be any run of separators, entities or tags.
static DANCE_REGEXES: LazyLock<Vec<DanceMatchers>> = LazyLock::new(|| {
    DANCE_ABBREVIATIONS
        .iter()
        .map(|&(dance, aliases)| {
            let regexes = aliases
                .iter()
                .map(|&a| {
                    let escaped = regex::escape(a);
                    let flexible = escaped.replace(r"\ ", r"([\s/.,()<>]|&nbsp;|<[^>]+>)*");
                    let pattern = format!(
                        r"(^|[\s/.,()<>])({}|{})([\s/.,()<>]|&nbsp;|$)",
                        escaped, flexible
                    );
                    (
                        a,
                        Regex::new(&pattern).expect("invalid dance alias pattern"),
                    )
                })
                .collect();
            (dance, regexes)
        })
        .collect()
});

/// Returns the minimum number of dances required for a given level and date.
/// Implements the 2026 rule transition logic.
pub fn get_min_dances(level: Level, date: NaiveDate) -> u32 {
    let is_2026_or_later = date.year() >= 2026;
    match (level, is_2026_or_later) {
        (Level::D, true) => 4,
        (Level::D, false) => 3,
        (Level::C, true) => 5,
        (Level::C, false) => 4,
        (Level::E, _) => 3,
        (Level::B | Level::A | Level::S, _) => 5,
    }
}

pub fn parse_level(s: &str) -> Option<Level> {
    LEVEL_MAPPINGS
        .iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn parse_style(s: &str) -> Option<Style> {
    STYLE_ID_MAPPINGS
        .iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn parse_age_group(s: &str) -> Option<AgeGroup> {
    AGE_GROUP_ID_MAPPINGS
        .iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn map_age_group(s: &str) -> Option<AgeGroup> {
    AGE_GROUP_MAPPINGS
        .iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn map_discipline(s: &str) -> Option<Style> {
    STYLE_MAPPINGS
        .iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn map_role(s: &str) -> Option<String> {
    ROLE_MAPPINGS
        .iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, id)| id.to_string())
}

pub fn map_month(mon_str: &str) -> Option<u32> {
    let lower = mon_str.to_lowercase();
    MONTH_MAPPINGS
        .iter()
        .find(|&&(k, _)| k == lower)
        .map(|&(_, m)| m)
}

pub fn parse_dances_no_fallback(s: &str) -> Vec<Dance> {
    let mut dances = Vec::new();
    let s_up = s.to_uppercase();

    for (dance, aliases) in DANCE_REGEXES.iter() {
        if aliases.iter().any(|(a, re)| {
            if *a == "SF" {
                re.is_match(&s_up) && !s_up.contains("WDSF")
            } else {
                re.is_match(&s_up)
            }
        }) {
            dances.push(*dance);
        }
    }

    dances.sort();
    dances.dedup();
    dances
}

pub fn parse_dances(s: &str) -> Vec<Dance> {
    let mut dances = parse_dances_no_fallback(s);
    let s_up = s.to_uppercase();

    if dances.is_empty() {
        if s_up.contains(STYLE_MARKER_STANDARD) {
            dances = STANDARD_DANCES.to_vec();
        } else if s_up.contains(STYLE_MARKER_LATEIN) || s_up.contains(STYLE_MARKER_LATIN) {
            dances = LATIN_DANCES.to_vec();
        }
    }

    dances.sort();
    dances.dedup();
    dances
}

pub fn parse_round_name(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    for &(marker, canonical) in ROUND_NAME_MAPPINGS {
        if lower.contains(marker) {
            if canonical == ROUND_NAME_ZWISCHENRUNDE {
                if lower.contains("1.") || lower.contains("erste") {
                    return Some(format!("1. {}", ROUND_NAME_ZWISCHENRUNDE));
                } else if lower.contains("2.") || lower.contains("zweite") {
                    return Some(format!("2. {}", ROUND_NAME_ZWISCHENRUNDE));
                } else if lower.contains("3.") || lower.contains("dritte") {
                    return Some(format!("3. {}", ROUND_NAME_ZWISCHENRUNDE));
                }
            }
            return Some(canonical.to_string());
        }
    }
    None
}

pub fn get_round_name_from_id(p: &str) -> String {
    if is_final_id(p) {
        ROUND_NAME_ENDRUNDE.to_string()
    } else if let Ok(n) = p.parse::<u32>() {
        if n == 1 {
            ROUND_NAME_VORRUNDE.to_string()
        } else if n > 1 {
            format!("{}. {}", n - 1, ROUND_NAME_ZWISCHENRUNDE)
        } else {
            p.to_string()
        }
    } else if let Some(name) = parse_round_name(p) {
        name
    } else {
        p.to_string()
    }
}

pub fn is_final_round(name: &str) -> bool {
    let lower = name.to_lowercase();
    FINAL_ROUND_MARKERS.iter().any(|&m| lower.contains(m))
}

pub fn is_final_id(s: &str) -> bool {
    s.trim() == FINAL_ID_MARKER
}

pub fn is_redance(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    REDANCE_MARKERS.iter().any(|&m| name_lower.contains(m))
}

pub fn is_organizer_marker(s: &str) -> bool {
    let lower = s.to_lowercase();
    ORGANIZER_MARKERS.iter().any(|&m| lower.contains(m))
}

pub fn is_hosting_club_marker(s: &str) -> bool {
    let lower = s.to_lowercase();
    HOSTING_CLUB_MARKERS.iter().any(|&m| lower.contains(m))
}

pub fn age_group_keys() -> Vec<&'static str> {
    AGE_GROUP_MAPPINGS.iter().map(|&(k, _)| k).collect()
}

pub fn style_keys() -> Vec<&'static str> {
    STYLE_MAPPINGS.iter().map(|&(k, _)| k).collect()
}

pub fn is_bib_column_marker(s: &str) -> bool {
    let lower = s.to_lowercase();
    BIB_COLUMN_MARKERS.iter().any(|&m| lower == m)
}

pub fn is_rank_column_marker(s: &str) -> bool {
    let lower = s.to_lowercase();
    RANK_COLUMN_MARKERS.iter().any(|&m| {
        if m == "rank" || m == "platz" {
            lower == m
        } else {
            lower.contains(m)
        }
    })
}

pub fn is_sum_column_marker(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.contains("pz") || SUM_COLUMN_MARKERS.iter().any(|&m| lower == m)
}

pub fn is_round_column_marker(s: &str) -> bool {
    let lower = s.to_lowercase();
    ROUND_COLUMN_MARKERS.iter().any(|&m| lower == m)
}

pub fn is_participant_marker(s: &str) -> bool {
    let lower = s.to_lowercase();
    PARTICIPANT_MARKERS.iter().any(|&m| lower.contains(m))
}

pub fn map_wdsf_score_type(line: &str) -> Vec<&'static str> {
    let lower = line.to_lowercase();
    let mut results = Vec::new();

    // Find all occurrences and their positions
    let mut found = Vec::new();
    for &(marker, id) in WDSF_SCORE_TYPES {
        let mut start = 0;
        while let Some(pos) = lower[start..].find(marker) {
            found.push((start + pos, id));
            start += pos + marker.len();
        }
    }

    // Sort by position and collect IDs
    found.sort_by_key(|&(pos, _)| pos);
    for (_, id) in found {
        results.push(id);
    }
    results
}

pub fn clean_competition_title(title: &str) -> String {
    let mut title_clean = title.to_string();
    for s in TITLE_CLEANUP_STRINGS {
        title_clean = title_clean.replace(s, "");
    }
    if let Some(pos) = title_clean.find(" - ") {
        title_clean = title_clean[pos + 3..].to_string();
    }
    title_clean.trim().to_string()
}

pub fn is_level_s_marker(up: &str) -> bool {
    LEVEL_S_MARKERS.iter().any(|&m| up.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_month_german_and_english() {
        assert_eq!(map_month("okt"), Some(10));
        assert_eq!(map_month("Okt"), Some(10));
        assert_eq!(map_month("Oktober"), Some(10));
        assert_eq!(map_month("october"), Some(10));
        assert_eq!(map_month("Dez"), Some(12));
        assert_eq!(map_month("December"), Some(12));
        assert_eq!(map_month("mär"), Some(3));
        assert_eq!(map_month("MÄR"), Some(3));
        assert_eq!(map_month("Mrz"), Some(3));
        assert_eq!(map_month("maerz"), Some(3));
        assert_eq!(map_month("März"), Some(3));
        assert_eq!(map_month("march"), Some(3));
        assert_eq!(map_month("june"), Some(6));
        assert_eq!(map_month("July"), Some(7));
        assert_eq!(map_month("January"), Some(1));
        assert_eq!(map_month("february"), Some(2));
        assert_eq!(map_month("Sept"), Some(9));
        assert_eq!(map_month("foo"), None);
    }

    #[test]
    fn test_date_pattern() {
        let re = Regex::new(PATTERN_DATE).unwrap();
        let cases = [
            ("12. Okt. 2024", ("12", "Okt", "2024")),
            ("05.07.2025", ("05", "07", "2025")),
            ("05/07/2025", ("05", "07", "2025")),
            ("5.Mär.2025", ("5", "Mär", "2025")),
            ("12.Oktober.2024", ("12", "Oktober", "2024")),
        ];
        for (input, (d, m, y)) in cases {
            let caps = re.captures(input).unwrap();
            assert_eq!((&caps[1], &caps[2], &caps[3]), (d, m, y), "{}", input);
        }
    }

    #[test]
    fn test_map_age_group_spaced_aliases() {
        assert_eq!(map_age_group("Sen I"), Some(AgeGroup::Sen1));
        assert_eq!(map_age_group("sen ii"), Some(AgeGroup::Sen2));
        assert_eq!(map_age_group("Mas III"), Some(AgeGroup::Sen3));
        assert_eq!(map_age_group("Hgr. II"), Some(AgeGroup::Adult2));
        assert_eq!(map_age_group("Junior I"), Some(AgeGroup::Jun1));
        assert_eq!(map_age_group("Youth"), Some(AgeGroup::Youth));
    }

    #[test]
    fn test_parse_dances() {
        assert_eq!(
            parse_dances_no_fallback("SW TG QS"),
            vec![Dance::SlowWaltz, Dance::Tango, Dance::Quickstep]
        );
        assert_eq!(
            parse_dances_no_fallback("Wiener Walzer"),
            vec![Dance::VienneseWaltz]
        );
        assert!(parse_dances_no_fallback("WDSF SF").is_empty());
        assert_eq!(parse_dances("Hgr. C Standard"), STANDARD_DANCES.to_vec());
        let mut latin = LATIN_DANCES.to_vec();
        latin.sort();
        assert_eq!(parse_dances("Mas.I C Latein"), latin);
    }

    #[test]
    fn test_case_insensitive_markers() {
        assert!(is_organizer_marker("Veranstalter:"));
        assert!(is_organizer_marker("ORGANIZER"));
        assert!(is_hosting_club_marker("Hosting club"));
        assert!(is_hosting_club_marker("ausrichter"));
        assert!(is_sum_column_marker("PZ"));
        assert!(is_sum_column_marker("Total"));
        assert_eq!(
            map_role("turnierleiter").as_deref(),
            Some("responsible_person")
        );
    }
}
