//! DTV Competition Ruleset and Internationalization
//! This file contains the hardcoded "DTV Competition Ruleset" and string mappings.
//! It must be updated if national regulations change.

use crate::models::{AgeGroup, Style, Level, Dance};
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

const AGE_GROUP_MAPPINGS: &[(&str, &str)] = &[
    ("Hgr", "adult"), ("Hgr.", "adult"), ("Hauptgruppe", "adult"), ("Adult", "adult"), ("Adults", "adult"), ("Rising Stars", "adult"),
    ("Hgr.II", "adult_2"), ("Hgr II", "adult_2"), ("Hauptgruppe II", "adult_2"),
    ("Sen", "senior"), ("Sen.", "senior"), ("Mas.", "senior"),
    ("Sen.I", "sen_1"), ("Mas.I", "sen_1"), ("Senioren I", "sen_1"),
    ("Sen.II", "sen_2"), ("Mas.II", "sen_2"), ("Senioren II", "sen_2"),
    ("Sen.III", "sen_3"), ("Mas.III", "sen_3"), ("Senioren III", "sen_3"),
    ("Sen.IV", "sen_4"), ("Mas.IV", "sen_4"), ("Senioren IV", "sen_4"),
    ("Sen.V", "sen_5"), ("Mas.V", "sen_5"), ("Senioren V", "sen_5"),
    ("Kinder I", "juv_1"), ("Kin.I", "juv_1"),
    ("Kinder II", "juv_2"), ("Kin.II", "juv_2"),
    ("Kin.", "juv"),
    ("Junioren I", "jun_1"), ("Jun.I", "jun_1"),
    ("Junioren II", "jun_2"), ("Jun.II", "jun_2"),
    ("Jugend", "youth"), ("Jug.", "youth"),
];

const STYLE_MAPPINGS: &[(&str, &str)] = &[
    ("Standard", "std"),
    ("Latein", "lat"),
    ("Latin", "lat"),
];

const DANCE_ABBREVIATIONS: &[(Dance, &[&str])] = &[
    (Dance::SlowWaltz, &["SW", "LW", "WALZER"]),
    (Dance::Tango, &["TG", "TANGO"]),
    (Dance::VienneseWaltz, &["VW", "WIENER", "WW"]),
    (Dance::SlowFoxtrot, &["SF", "SLOW", "FOX"]),
    (Dance::Quickstep, &["QS", "QU", "QUICK"]),
    (Dance::ChaChaCha, &["CC", "CHA"]),
    (Dance::Samba, &["SB", "SA", "SAMBA"]),
    (Dance::Rumba, &["RB", "RU", "RUMBA"]),
    (Dance::PasoDoble, &["PD", "PASO"]),
    (Dance::Jive, &["JV", "JI", "JIVE"]),
];

pub struct I18n {}

impl I18n {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    pub fn map_age_group(&self, s: &str) -> Option<AgeGroup> {
        AGE_GROUP_MAPPINGS.iter()
            .find(|&&(k, _)| k == s)
            .and_then(|&(_, id)| AgeGroup::from_id(id))
    }

    pub fn map_discipline(&self, s: &str) -> Option<Style> {
        STYLE_MAPPINGS.iter()
            .find(|&&(k, _)| k == s)
            .and_then(|&(_, id)| Style::from_id(id))
    }

    pub fn map_role(&self, s: &str) -> Option<String> {
        match s {
            "Turnierleiter" => Some("responsible_person".to_string()),
            "Beisitzer" => Some("assistant".to_string()),
            _ => None,
        }
    }

    pub fn map_month(&self, mon_str: &str) -> Option<u32> {
        match mon_str.to_lowercase().as_str() {
            "jan" | "januar" => Some(1),
            "feb" | "februar" => Some(2),
            "mar" | "märz" => Some(3),
            "apr" | "april" => Some(4),
            "may" | "mai" => Some(5),
            "jun" | "juni" => Some(6),
            "jul" | "juli" => Some(7),
            "aug" | "august" => Some(8),
            "sep" | "september" => Some(9),
            "oct" | "oktober" => Some(10),
            "nov" | "november" => Some(11),
            "dec" | "dezember" => Some(12),
            _ => None,
        }
    }

    pub fn parse_dances(&self, s: &str) -> Vec<Dance> {
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
                // Heuristic: check if we are in a Latin or Standard context
                // but for now let's just match as it was in dtv_native.rs
                dances.push(dance);
            }
        }

        // Special handling for broad disciplines if no specific dances found
        if dances.is_empty() {
            if s_up.contains("STANDARD") {
                dances = vec![Dance::SlowWaltz, Dance::Tango, Dance::VienneseWaltz, Dance::SlowFoxtrot, Dance::Quickstep];
            } else if s_up.contains("LATEIN") || s_up.contains("LATIN") {
                dances = vec![Dance::Samba, Dance::ChaChaCha, Dance::Rumba, Dance::PasoDoble, Dance::Jive];
            }
        }

        // Sort to ensure consistent order (optional, but good for stability)
        dances.sort_by_key(|&d| d as u32);
        dances.dedup();
        dances
    }

    pub fn age_group_keys(&self) -> Vec<&'static str> {
        AGE_GROUP_MAPPINGS.iter().map(|&(k, _)| k).collect()
    }

    pub fn style_keys(&self) -> Vec<&'static str> {
        STYLE_MAPPINGS.iter().map(|&(k, _)| k).collect()
    }
}
