//! Assets for DTV Competition Ruleset and Internationalization
//! This file contains hardcoded constants and mappings.

use crate::models::{AgeGroup, Dance, Level, Style};

pub const AGE_GROUP_MAPPINGS: &[(&str, AgeGroup)] = &[
    ("Hgr", AgeGroup::Adult),
    ("Hgr.", AgeGroup::Adult),
    ("Hauptgruppe", AgeGroup::Adult),
    ("Adult", AgeGroup::Adult),
    ("Adults", AgeGroup::Adult),
    ("Rising Stars", AgeGroup::Adult),
    ("Hgr.II", AgeGroup::Adult2),
    ("Hgr II", AgeGroup::Adult2),
    ("Hauptgruppe II", AgeGroup::Adult2),
    ("Sen", AgeGroup::Senior),
    ("Sen.", AgeGroup::Senior),
    ("Mas.", AgeGroup::Senior),
    ("Sen.I", AgeGroup::Sen1),
    ("Mas.I", AgeGroup::Sen1),
    ("Senioren I", AgeGroup::Sen1),
    ("Sen.II", AgeGroup::Sen2),
    ("Mas.II", AgeGroup::Sen2),
    ("Senioren II", AgeGroup::Sen2),
    ("Sen.III", AgeGroup::Sen3),
    ("Mas.III", AgeGroup::Sen3),
    ("Senioren III", AgeGroup::Sen3),
    ("Sen.IV", AgeGroup::Sen4),
    ("Mas.IV", AgeGroup::Sen4),
    ("Senioren IV", AgeGroup::Sen4),
    ("Sen.V", AgeGroup::Sen5),
    ("Mas.V", AgeGroup::Sen5),
    ("Senioren V", AgeGroup::Sen5),
    ("Kinder I", AgeGroup::Juv1),
    ("Kin.I", AgeGroup::Juv1),
    ("Kinder II", AgeGroup::Juv2),
    ("Kin.II", AgeGroup::Juv2),
    ("Junioren I", AgeGroup::Jun1),
    ("Jun.I", AgeGroup::Jun1),
    ("Junioren II", AgeGroup::Jun2),
    ("Jun.II", AgeGroup::Jun2),
    ("Jugend", AgeGroup::Youth),
    ("Jug.", AgeGroup::Youth),
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
    (Dance::SlowWaltz, &["SW", "LW", "WALZER", "WALTZ"]),
    (Dance::Tango, &["TG", "TANGO"]),
    (Dance::VienneseWaltz, &["VW", "WIENER", "WW"]),
    (Dance::SlowFoxtrot, &["SF", "SLOW", "FOX"]),
    (Dance::Quickstep, &["QS", "QU", "QUICK"]),
    (Dance::ChaChaCha, &["CC", "CHA", "CCC"]),
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
    ("Master of Ceremony", "responsible_person"),
    ("Beisitzer", "assistant"),
    ("Chairperson", "assistant"),
    ("Protokoll", "assistant"),
];

pub const MONTH_MAPPINGS: &[(&str, u32)] = &[
    ("jan", 1),
    ("januar", 1),
    ("feb", 2),
    ("februar", 2),
    ("mar", 3),
    ("märz", 3),
    ("apr", 4),
    ("april", 4),
    ("may", 5),
    ("mai", 5),
    ("jun", 6),
    ("juni", 6),
    ("jul", 7),
    ("juli", 7),
    ("aug", 8),
    ("august", 8),
    ("sep", 9),
    ("september", 9),
    ("oct", 10),
    ("oktober", 10),
    ("nov", 11),
    ("november", 11),
    ("dec", 12),
    ("dezember", 12),
];

pub const ROUND_NAME_MAPPINGS: &[(&str, &str)] = &[
    ("vorrunde", ROUND_NAME_VORRUNDE),
    ("1st round", ROUND_NAME_VORRUNDE),
    ("1. runde", ROUND_NAME_VORRUNDE),
    ("zwischenrunde", ROUND_NAME_ZWISCHENRUNDE),
    ("2nd round", ROUND_NAME_ZWISCHENRUNDE),
    ("3rd round", ROUND_NAME_ZWISCHENRUNDE),
    ("2. runde", ROUND_NAME_ZWISCHENRUNDE),
    ("3. runde", ROUND_NAME_ZWISCHENRUNDE),
    ("semifinal", ROUND_NAME_SEMIFINAL),
    ("semi-final", ROUND_NAME_SEMIFINAL),
    ("1/2 finale", ROUND_NAME_SEMIFINAL),
    ("quarterfinal", ROUND_NAME_QUARTERFINAL),
    ("1/4 finale", ROUND_NAME_QUARTERFINAL),
    ("endrunde", ROUND_NAME_ENDRUNDE),
    ("finale", ROUND_NAME_ENDRUNDE),
    ("final", ROUND_NAME_ENDRUNDE),
];

pub const GENERIC_ROUND_MARKERS: &[&str] = &["runde", "round", "ergebnis", "ranking", "table"];
pub const SKIP_ROUND_MARKERS: &[&str] = &["ranking report", "table of results"];
pub const FINAL_ROUND_MARKERS: &[&str] = &["endrunde", "finale", "final", "result of final"];
pub const RESULT_MARKERS: &[&str] = &["ergebnis", "result"];
pub const FINAL_ID_MARKER: &str = "F";

pub const ROUND_NAME_FINAL: &str = "Final";
pub const ROUND_NAME_SEMIFINAL: &str = "Semifinal";
pub const ROUND_NAME_QUARTERFINAL: &str = "Quarterfinal";
pub const ROUND_NAME_GENERIC_PREFIX: &str = "Round";
pub const ROUND_NAME_RESULT_TABLE: &str = "Result Table";
pub const ROUND_NAME_VORRUNDE: &str = "Vorrunde";
pub const ROUND_NAME_ENDRUNDE: &str = "Endrunde";
pub const ROUND_NAME_ZWISCHENRUNDE: &str = "Zwischenrunde";

pub const STYLE_MARKER_STANDARD: &str = "STANDARD";
pub const STYLE_MARKER_LATEIN: &str = "LATEIN";
pub const STYLE_MARKER_LATIN: &str = "LATIN";

pub const REDANCE_MARKERS: &[&str] = &["redance", "hoffnung", "h-lauf"];

pub const ORGANIZER_MARKERS: &[&str] = &["Veranstalter", "Organizer", "organizer"];
pub const HOSTING_CLUB_MARKERS: &[&str] = &["Ausrichter", "Hosting club", "Hosting Club"];

// --- CSS Selectors ---
pub const SELECTOR_META_AUTHOR: &str = "meta[name='Author'], meta[name='author']";
pub const SELECTOR_EVENT_HEAD: &str = ".eventhead td";
pub const SELECTOR_COMP_HEAD: &str = ".comphead";
pub const SELECTOR_PARTICIPANT_RANK: &str = "td.td3r, td.td3c, td.td3cv";
pub const SELECTOR_PARTICIPANT_DATA: &str = "td.td2c, td.td5, td.td6, td.td2cv, td.td5v, td.td6v";
pub const SELECTOR_ROUND_NAME: &str = ".comphead, h2, td.td1";
pub const SELECTOR_OFFICIAL_ROLE: &str = "td.td2, td.td2r";
pub const SELECTOR_OFFICIAL_DATA: &str = "td.td5";
pub const SELECTOR_CANONICAL: &str = "link[rel='canonical']";
pub const SELECTOR_TR: &str = "tr";
pub const SELECTOR_TD: &str = "td";
pub const SELECTOR_SPAN: &str = "span";
pub const SELECTOR_I: &str = "i";
pub const SELECTOR_TITLE: &str = "title";
pub const SELECTOR_TABLE: &str = "table";

// --- Regex Patterns ---
pub const PATTERN_BIB_PARENS: &str = r"\((\d+)\)";
pub const PATTERN_SCORE: &str = r"(\d+[\.,]\d+)";
pub const PATTERN_DATE: &str = r"(\d{1,2})[\./]([a-zA-Z0-9]{2,3})[\./](\d{4})";
pub const PATTERN_RANK: &str = r"(\d+)";

// --- Interpretation Markers ---
pub const BIB_COLUMN_MARKERS: &[&str] = &["nr", "nr.", "startnummer", "no.", "no", "start number"];
pub const RANK_COLUMN_MARKERS: &[&str] = &["pl.", "platz", "rank", "platz von", "platz bis"];
pub const SUM_COLUMN_MARKERS: &[&str] = &["su", "summe", "total", "pz", "sum", "total"];
pub const ROUND_COLUMN_MARKERS: &[&str] = &["r", "round", "runde"];
pub const PARTICIPANT_MARKERS: &[&str] =
    &["teilnehmer", "participant", "no. of participants", "country", "paar"];
pub const QUALIFICATION_MARKERS: &[&str] =
    &["result of", "qualified for", "ergebnis", "qualifiziert"];
pub const LEVEL_S_MARKERS: &[&str] = &["WDSF", "OPEN"];
pub const TITLE_CLEANUP_STRINGS: &[&str] = &[
    "\"GS\"",
    "\"OS\"",
    "\"MS\"",
    "&#34;GS&#34;",
    "&#34;OS&#34;",
    "&#34;MS&#34;",
    "OT, ",
];
pub const RESULT_CELL_CLASSES: &[&str] = &["td5c", "td5cv", "td3w"];

pub const WDSF_SCORE_TYPES: &[(&str, &str)] = &[
    ("tq", "technical_quality"),
    ("mm", "movement_to_music"),
    ("ps", "partnering_skills"),
    ("cp", "choreography"),
    ("total", "total"),
];
