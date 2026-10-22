//! Raw lookup tables, CSS selectors and regex patterns.
//! This file holds data only (alias tables, marker lists, selectors, patterns);
//! the lookup and normalization logic that uses it lives in `crate::i18n`.

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
    // DTV children's classes are not split into I/II.
    ("Kinder", AgeGroup::Juv),
    ("Kin.", AgeGroup::Juv),
    ("Junioren I", AgeGroup::Jun1),
    ("Jun.I", AgeGroup::Jun1),
    ("Junioren II", AgeGroup::Jun2),
    ("Jun.II", AgeGroup::Jun2),
    ("Jugend", AgeGroup::Youth),
    ("Jug.", AgeGroup::Youth),
    // Spaced and English variants (as used in user-facing filters)
    ("Hgr. II", AgeGroup::Adult2),
    ("Sen I", AgeGroup::Sen1),
    ("Sen II", AgeGroup::Sen2),
    ("Sen III", AgeGroup::Sen3),
    ("Sen IV", AgeGroup::Sen4),
    ("Sen V", AgeGroup::Sen5),
    ("Mas I", AgeGroup::Sen1),
    ("Mas II", AgeGroup::Sen2),
    ("Mas III", AgeGroup::Sen3),
    ("Mas IV", AgeGroup::Sen4),
    ("Mas V", AgeGroup::Sen5),
    ("Senior I", AgeGroup::Sen1),
    ("Senior II", AgeGroup::Sen2),
    ("Senior III", AgeGroup::Sen3),
    ("Senior IV", AgeGroup::Sen4),
    ("Senior V", AgeGroup::Sen5),
    ("Juvenile", AgeGroup::Juv),
    ("Jun I", AgeGroup::Jun1),
    ("Jun II", AgeGroup::Jun2),
    ("Junior I", AgeGroup::Jun1),
    ("Junior II", AgeGroup::Jun2),
    ("Youth", AgeGroup::Youth),
];

pub const AGE_GROUP_ID_MAPPINGS: &[(&str, AgeGroup)] = &[
    ("juv", AgeGroup::Juv),
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

/// All Standard dances in canonical order.
pub const STANDARD_DANCES: &[Dance] = &[
    Dance::SlowWaltz,
    Dance::Tango,
    Dance::VienneseWaltz,
    Dance::SlowFoxtrot,
    Dance::Quickstep,
];

/// All Latin dances in canonical order.
pub const LATIN_DANCES: &[Dance] = &[
    Dance::Samba,
    Dance::ChaChaCha,
    Dance::Rumba,
    Dance::PasoDoble,
    Dance::Jive,
];

pub const DANCE_ABBREVIATIONS: &[(Dance, &[&str])] = &[
    (
        Dance::SlowWaltz,
        &[
            "SW",
            "LW",
            "LANGSAMER WALZER",
            "SLOW WALTZ",
            "ENGLISH WALTZ",
        ],
    ),
    (Dance::Tango, &["TG", "TANGO"]),
    (
        Dance::VienneseWaltz,
        &["VW", "WW", "WIENER WALZER", "VIENNESE WALTZ", "V. WALTZ"],
    ),
    (Dance::SlowFoxtrot, &["SF", "SLOWFOX", "SLOW FOXTROT"]),
    (Dance::Quickstep, &["QS", "QUICKSTEP"]),
    (Dance::ChaChaCha, &["CC", "CHA CHA", "CHA CHA CHA"]),
    (Dance::Samba, &["SB", "SAMBA"]),
    (Dance::Rumba, &["RB", "RUMBA"]),
    (Dance::PasoDoble, &["PD", "PASO DOBLE"]),
    (Dance::Jive, &["JV", "JIVE"]),
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

/// Month names and abbreviations (German and English), lowercase.
pub const MONTH_MAPPINGS: &[(&str, u32)] = &[
    ("jan", 1),
    ("jän", 1),
    ("januar", 1),
    ("jänner", 1),
    ("january", 1),
    ("feb", 2),
    ("februar", 2),
    ("february", 2),
    ("mar", 3),
    ("mär", 3),
    ("mrz", 3),
    ("märz", 3),
    ("maerz", 3),
    ("march", 3),
    ("apr", 4),
    ("april", 4),
    ("may", 5),
    ("mai", 5),
    ("jun", 6),
    ("juni", 6),
    ("june", 6),
    ("jul", 7),
    ("juli", 7),
    ("july", 7),
    ("aug", 8),
    ("august", 8),
    ("sep", 9),
    ("sept", 9),
    ("september", 9),
    ("oct", 10),
    ("okt", 10),
    ("oktober", 10),
    ("october", 10),
    ("nov", 11),
    ("november", 11),
    ("dec", 12),
    ("dez", 12),
    ("dezember", 12),
    ("december", 12),
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

pub const FINAL_ROUND_MARKERS: &[&str] = &["endrunde", "finale", "final", "result of final"];
pub const FINAL_ID_MARKER: &str = "F";

pub const ROUND_NAME_SEMIFINAL: &str = "Semifinal";
pub const ROUND_NAME_QUARTERFINAL: &str = "Quarterfinal";
pub const ROUND_NAME_VORRUNDE: &str = "Vorrunde";
pub const ROUND_NAME_ENDRUNDE: &str = "Endrunde";
pub const ROUND_NAME_ZWISCHENRUNDE: &str = "Zwischenrunde";

pub const STYLE_MARKER_STANDARD: &str = "STANDARD";
pub const STYLE_MARKER_LATEIN: &str = "LATEIN";
pub const STYLE_MARKER_LATIN: &str = "LATIN";

pub const REDANCE_MARKERS: &[&str] = &["redance", "hoffnung", "h-lauf"];

/// Lowercase; matched case-insensitively as substrings.
pub const ORGANIZER_MARKERS: &[&str] = &["veranstalter", "organizer"];
/// Lowercase; matched case-insensitively as substrings.
pub const HOSTING_CLUB_MARKERS: &[&str] = &["ausrichter", "hosting club"];

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
/// Day, month (number or German/English name/abbreviation) and year,
/// e.g. `05.07.2025`, `05/07/2025`, `12. Okt. 2024`, `12.Oktober.2024`, `5.Mär.2025`.
pub const PATTERN_DATE: &str = r"(\d{1,2})[\./]\s*([\p{L}\d]{1,9})[\./]\s*(\d{4})";
pub const PATTERN_RANK: &str = r"(\d+)";

// --- Interpretation Markers ---
pub const BIB_COLUMN_MARKERS: &[&str] = &["nr", "nr.", "startnummer", "no.", "no", "start number"];
pub const RANK_COLUMN_MARKERS: &[&str] = &["pl.", "platz", "rank", "platz von", "platz bis"];
pub const SUM_COLUMN_MARKERS: &[&str] = &["su", "summe", "total", "pz", "sum"];
pub const ROUND_COLUMN_MARKERS: &[&str] = &["r", "round", "runde"];
pub const PARTICIPANT_MARKERS: &[&str] = &[
    "teilnehmer",
    "participant",
    "no. of participants",
    "country",
    "paar",
];
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

pub const WDSF_SCORE_TYPES: &[(&str, &str)] = &[
    ("tq", "technical_quality"),
    ("mm", "movement_to_music"),
    ("ps", "partnering_skills"),
    ("cp", "choreography"),
    ("total", "total"),
];
