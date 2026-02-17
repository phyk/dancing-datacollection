//! DTV Competition Ruleset and Internationalization
//! This file contains the hardcoded "DTV Competition Ruleset" and string mappings.
//! It must be updated if national regulations change.

use crate::models::{AgeGroup, Style, Level, Dance};
use crate::assets::*;
use chrono::{NaiveDate, Datelike};

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
    LEVEL_MAPPINGS.iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn parse_style(s: &str) -> Option<Style> {
    STYLE_ID_MAPPINGS.iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn parse_age_group(s: &str) -> Option<AgeGroup> {
    AGE_GROUP_ID_MAPPINGS.iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn map_age_group(s: &str) -> Option<AgeGroup> {
    AGE_GROUP_MAPPINGS.iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn map_discipline(s: &str) -> Option<Style> {
    STYLE_MAPPINGS.iter()
        .find(|&&(k, _)| k.eq_ignore_ascii_case(s))
        .map(|&(_, v)| v)
}

pub fn map_role(s: &str) -> Option<String> {
    ROLE_MAPPINGS.iter()
        .find(|&&(k, _)| k == s)
        .map(|&(_, id)| id.to_string())
}

pub fn map_month(mon_str: &str) -> Option<u32> {
    let lower = mon_str.to_lowercase();
    MONTH_MAPPINGS.iter()
        .find(|&&(k, _)| k == lower)
        .map(|&(_, m)| m)
}

pub fn parse_dances(s: &str) -> Vec<Dance> {
    let mut dances = Vec::new();
    let s_up = s.to_uppercase();

    for &(dance, aliases) in DANCE_ABBREVIATIONS {
        if aliases.iter().any(|&a| {
            if a == "SF" {
                s_up.contains("SF") && !s_up.contains("WDSF")
            } else {
                s_up.contains(a)
            }
        }) {
            dances.push(dance);
        }
    }

    if dances.is_empty() {
        if s_up.contains(STYLE_MARKER_STANDARD) {
            dances = vec![Dance::SlowWaltz, Dance::Tango, Dance::VienneseWaltz, Dance::SlowFoxtrot, Dance::Quickstep];
        } else if s_up.contains(STYLE_MARKER_LATEIN) || s_up.contains(STYLE_MARKER_LATIN) {
            dances = vec![Dance::Samba, Dance::ChaChaCha, Dance::Rumba, Dance::PasoDoble, Dance::Jive];
        }
    }

    dances.sort_by_key(|&d| d as u32);
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
    } else {
        p.to_string()
    }
}

pub fn is_generic_round_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    GENERIC_ROUND_MARKERS.iter().any(|&m| lower.contains(m)) || lower.starts_with("round")
}

pub fn should_skip_round(name: &str) -> bool {
    let lower = name.to_lowercase();
    SKIP_ROUND_MARKERS.iter().any(|&m| lower.contains(m))
}

pub fn is_final_round(name: &str) -> bool {
    let lower = name.to_lowercase();
    FINAL_ROUND_MARKERS.iter().any(|&m| lower.contains(m))
}

pub fn is_result_marker(s: &str) -> bool {
    let lower = s.to_lowercase();
    RESULT_MARKERS.iter().any(|&m| lower.contains(m))
}

pub fn is_final_id(s: &str) -> bool {
    s.trim() == FINAL_ID_MARKER
}

pub fn normalize_wdsf_round_name(name: &str, i: usize, num_rounds: usize) -> String {
    let lower = name.to_lowercase();
    if is_final_round(&lower) {
        return ROUND_NAME_FINAL.to_string();
    }

    if num_rounds >= 3 {
        if i == num_rounds - 1 { return ROUND_NAME_FINAL.to_string(); }
        if i == num_rounds - 2 { return ROUND_NAME_SEMIFINAL.to_string(); }
        if i == num_rounds - 3 { return ROUND_NAME_QUARTERFINAL.to_string(); }
    } else if num_rounds == 2 {
        if i == num_rounds - 1 { return ROUND_NAME_FINAL.to_string(); }
        if i == num_rounds - 2 { return ROUND_NAME_SEMIFINAL.to_string(); }
    }

    name.to_string()
}

pub fn get_generic_round_name(i: usize) -> String {
    format!("{} {}", ROUND_NAME_GENERIC_PREFIX, i)
}

pub fn get_result_table_name() -> String {
    ROUND_NAME_RESULT_TABLE.to_string()
}

pub fn is_redance(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    REDANCE_MARKERS.iter().any(|&m| name_lower.contains(m))
}

pub fn is_organizer_marker(s: &str) -> bool {
    ORGANIZER_MARKERS.iter().any(|&m| s.contains(m))
}

pub fn is_hosting_club_marker(s: &str) -> bool {
    HOSTING_CLUB_MARKERS.iter().any(|&m| s.contains(m))
}

pub fn age_group_keys() -> Vec<&'static str> {
    AGE_GROUP_MAPPINGS.iter().map(|&(k, _)| k).collect()
}

pub fn style_keys() -> Vec<&'static str> {
    STYLE_MAPPINGS.iter().map(|&(k, _)| k).collect()
}
