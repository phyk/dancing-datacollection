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
    match s.to_uppercase().as_str() {
        "E" => Some(Level::E),
        "D" => Some(Level::D),
        "C" => Some(Level::C),
        "B" => Some(Level::B),
        "A" => Some(Level::A),
        "S" => Some(Level::S),
        _ => None,
    }
}

pub fn parse_style(s: &str) -> Option<Style> {
    match s.to_lowercase().as_str() {
        "std" | "standard" => Some(Style::Standard),
        "lat" | "latin" | "latein" => Some(Style::Latein),
        _ => None,
    }
}

pub fn parse_age_group(s: &str) -> Option<AgeGroup> {
    match s.to_lowercase().as_str() {
        "juv_1" => Some(AgeGroup::Juv1),
        "juv_2" => Some(AgeGroup::Juv2),
        "jun_1" => Some(AgeGroup::Jun1),
        "jun_2" => Some(AgeGroup::Jun2),
        "youth" => Some(AgeGroup::Youth),
        "adult" => Some(AgeGroup::Adult),
        "adult_2" => Some(AgeGroup::Adult2),
        "sen_1" => Some(AgeGroup::Sen1),
        "sen_2" => Some(AgeGroup::Sen2),
        "sen_3" => Some(AgeGroup::Sen3),
        "sen_4" => Some(AgeGroup::Sen4),
        "sen_5" => Some(AgeGroup::Sen5),
        "senior" => Some(AgeGroup::Senior),
        _ => None,
    }
}

pub fn map_age_group(s: &str) -> Option<AgeGroup> {
    AGE_GROUP_MAPPINGS.iter()
        .find(|&&(k, _)| k == s)
        .and_then(|&(_, id)| parse_age_group(id))
}

pub fn map_discipline(s: &str) -> Option<Style> {
    STYLE_MAPPINGS.iter()
        .find(|&&(k, _)| k == s)
        .and_then(|&(_, id)| parse_style(id))
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
        if s_up.contains("STANDARD") {
            dances = vec![Dance::SlowWaltz, Dance::Tango, Dance::VienneseWaltz, Dance::SlowFoxtrot, Dance::Quickstep];
        } else if s_up.contains("LATEIN") || s_up.contains("LATIN") {
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
            if marker == "zwischenrunde" {
                 if lower.contains("1.") || lower.contains("erste") {
                      return Some("1. Zwischenrunde".to_string());
                 } else if lower.contains("2.") || lower.contains("zweite") {
                      return Some("2. Zwischenrunde".to_string());
                 } else if lower.contains("3.") || lower.contains("dritte") {
                      return Some("3. Zwischenrunde".to_string());
                 }
            }
            return Some(canonical.to_string());
        }
    }
    None
}

pub fn get_round_name_from_id(p: &str) -> String {
    if p == "F" {
        "Endrunde".to_string()
    } else if let Ok(n) = p.parse::<u32>() {
        if n == 1 {
            "Vorrunde".to_string()
        } else if n > 1 {
            format!("{}. Zwischenrunde", n - 1)
        } else {
            p.to_string()
        }
    } else {
        p.to_string()
    }
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
