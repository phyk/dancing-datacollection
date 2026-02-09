//! Assets for DTV Competition Ruleset and Internationalization
//! This file contains hardcoded constants and mappings.

use crate::models::{Dance, AgeGroup, Style, Level};

pub const AGE_GROUP_MAPPINGS: &[(&str, AgeGroup)] = &[
    ("Hgr", AgeGroup::Adult), ("Hgr.", AgeGroup::Adult), ("Hauptgruppe", AgeGroup::Adult), ("Adult", AgeGroup::Adult), ("Adults", AgeGroup::Adult), ("Rising Stars", AgeGroup::Adult),
    ("Hgr.II", AgeGroup::Adult2), ("Hgr II", AgeGroup::Adult2), ("Hauptgruppe II", AgeGroup::Adult2),
    ("Sen", AgeGroup::Senior), ("Sen.", AgeGroup::Senior), ("Mas.", AgeGroup::Senior),
    ("Sen.I", AgeGroup::Sen1), ("Mas.I", AgeGroup::Sen1), ("Senioren I", AgeGroup::Sen1),
    ("Sen.II", AgeGroup::Sen2), ("Mas.II", AgeGroup::Sen2), ("Senioren II", AgeGroup::Sen2),
    ("Sen.III", AgeGroup::Sen3), ("Mas.III", AgeGroup::Sen3), ("Senioren III", AgeGroup::Sen3),
    ("Sen.IV", AgeGroup::Sen4), ("Mas.IV", AgeGroup::Sen4), ("Senioren IV", AgeGroup::Sen4),
    ("Sen.V", AgeGroup::Sen5), ("Mas.V", AgeGroup::Sen5), ("Senioren V", AgeGroup::Sen5),
    ("Kinder I", AgeGroup::Juv1), ("Kin.I", AgeGroup::Juv1),
    ("Kinder II", AgeGroup::Juv2), ("Kin.II", AgeGroup::Juv2),
    ("Junioren I", AgeGroup::Jun1), ("Jun.I", AgeGroup::Jun1),
    ("Junioren II", AgeGroup::Jun2), ("Jun.II", AgeGroup::Jun2),
    ("Jugend", AgeGroup::Youth), ("Jug.", AgeGroup::Youth),
];

pub const AGE_GROUP_ID_MAPPINGS: &[(&str, AgeGroup)] = &[
    ("juv_1", AgeGroup::Juv1),
    ("juv_2", AgeGroup::Juv2),
    ("jun_1", AgeGroup::Jun1),
    ("jun_2", AgeGroup::Jun2),
    ("youth", AgeGroup::Youth),
    ("adult", AgeGroup::Adult),
    ("adult_2", AgeGroup::Adult2),
    ("sen_1", AgeGroup::Sen1),
    ("sen_2", AgeGroup::Sen2),
    ("sen_3", AgeGroup::Sen3),
    ("sen_4", AgeGroup::Sen4),
    ("sen_5", AgeGroup::Sen5),
    ("senior", AgeGroup::Senior),
];

pub const STYLE_MAPPINGS: &[(&str, Style)] = &[
    ("Standard", Style::Standard),
    ("Latein", Style::Latein),
    ("Latin", Style::Latein),
];

pub const STYLE_ID_MAPPINGS: &[(&str, Style)] = &[
    ("std", Style::Standard),
    ("standard", Style::Standard),
    ("lat", Style::Latein),
    ("latin", Style::Latein),
    ("latein", Style::Latein),
];

pub const DANCE_ABBREVIATIONS: &[(Dance, &[&str])] = &[
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

pub const LEVEL_MAPPINGS: &[(&str, Level)] = &[
    ("E", Level::E),
    ("D", Level::D),
    ("C", Level::C),
    ("B", Level::B),
    ("A", Level::A),
    ("S", Level::S),
];

pub const ROLE_MAPPINGS: &[(&str, &str)] = &[
    ("Turnierleiter", "responsible_person"),
    ("Beisitzer", "assistant"),
];

pub const MONTH_MAPPINGS: &[(&str, u32)] = &[
    ("jan", 1), ("januar", 1),
    ("feb", 2), ("februar", 2),
    ("mar", 3), ("märz", 3),
    ("apr", 4), ("april", 4),
    ("may", 5), ("mai", 5),
    ("jun", 6), ("juni", 6),
    ("jul", 7), ("juli", 7),
    ("aug", 8), ("august", 8),
    ("sep", 9), ("september", 9),
    ("oct", 10), ("oktober", 10),
    ("nov", 11), ("november", 11),
    ("dec", 12), ("dezember", 12),
];

pub const ROUND_NAME_MAPPINGS: &[(&str, &str)] = &[
    ("vorrunde", "Vorrunde"),
    ("zwischenrunde", "Zwischenrunde"),
    ("endrunde", "Endrunde"),
    ("finale", "Endrunde"),
    ("final", "Endrunde"),
];

pub const REDANCE_MARKERS: &[&str] = &["redance", "hoffnung", "h-lauf"];

pub const ORGANIZER_MARKERS: &[&str] = &["Veranstalter"];
pub const HOSTING_CLUB_MARKERS: &[&str] = &["Ausrichter"];
