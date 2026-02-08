//! Assets for DTV Competition Ruleset and Internationalization
//! This file contains hardcoded constants and mappings.

use crate::models::Dance;

pub const AGE_GROUP_MAPPINGS: &[(&str, &str)] = &[
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

pub const STYLE_MAPPINGS: &[(&str, &str)] = &[
    ("Standard", "std"),
    ("Latein", "lat"),
    ("Latin", "lat"),
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

pub const LEVEL_MAPPINGS: &[(&str, &str)] = &[
    ("E", "E"),
    ("D", "D"),
    ("C", "C"),
    ("B", "B"),
    ("A", "A"),
    ("S", "S"),
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
