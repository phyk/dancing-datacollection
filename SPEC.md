Task Specification: dancing-datacollection

Project Overview Goal: Build a high-performance Python library (core in Rust via PyO3) and CLI tool to scrape, parse, and archive dance competition results with 100% data fidelity. Core Value: The data must be an exact, machine-readable reproduction of the online results, enabling full reproduction of the tournament progression.

Tech Stack & Standards Language: Rust (Core logic/Parsing), Python (Bindings/CLI).

Interop: pyo3 and maturin.

Formatting: Strict adherence to rustfmt.

Code Style: "Flat" abstractions. Minimize layers. Use Traits for extensibility rather than deep object hierarchies.

Dependencies: reqwest (HTTP), scraper (HTML parsing), serde (Serialization), postcard (Binary format), clap (CLI).

Data Schema (Core Models) The agent must implement the following hierarchy in Rust.
A. The Event & Competition Event: A high-level container (e.g., "Hessen tanzt").

Fields: name, organizer, hosting_club, competitions_list.

Competition: A specific contest within an event.

Level: Enum [E, D, C, B, A, S].

Age Group: Use internal Localization module for aliases (e.g., Senioren I / Senior 1).

Style: Standard or Latein.

Dances: Dynamic list based on level/style (e.g., Standard S = [SW, TG, VW, SF, QS]).

Officials: Must include "Responsible Person," "Assistant," and "Judges" (2-letter code, name, club).

B. Participants Identity: Support for Solo (single name) or Couple (Lead + Follow).

Affiliation: Club name.

Context: Bib number and Final Rank.

C. Rounds & Scoring Marking Rounds: Store "Crosses" mapped by Judge -> Participant -> Dance.

Final Rounds (Two Flavors):

DTV (National): Ranks (integers) per Judge/Dance.

WDSF (International): Decimal scores across four categories: Technical Quality, Movement to Music, Partnering Skills, Choreography.

Technical Constraints & Logic Robots & Crawling: * Must parse and respect robots.txt.
Implement a "Smart Skip": Check local storage manifest before downloading to prevent re-downloads.

Fidelity Gate (Validation) & Math Check:

A competition is invalid if it lacks Officials, Judges, or Results.

Structure Check: If a competition level (e.g., "Standard A") requires 5 dances but only 3 are found, log a PARSING_ERROR and do not save.

Mandatory Math Check: The library performs a Skating System re-calculation for DTV ranks and verifies WDSF score totals.

**No-Validation-No-Save Policy**: If a competition fails the Fidelity Gate or the Mandatory Math Check, it is logged as a CRITICAL_VALIDATION_ERROR and NOT saved to the archive.

Localization Module:

Store German/English aliases for Age Groups and Levels in a separate, accessible module to allow easy language switching.

Input & Output Input: Direct URL via API or CLI.
Output Formats:

Human-Readable: .json for single competition events.

Optimized Binary: postcard or MessagePack (Internal/Optional).

Standardized Table Assumption: The tool assumes all provided URLs contain DTV/TopTurnier compatible result tables. Domain-based filtering is removed to support any domain using the standard DTV-Native structure. If the HTML structure is unrecognizable, the parser returns a `ParsingError::InvalidTableStructure`.

Abstract Design: ResultSource Trait All scrapers must implement this trait (standardizing on `DtvNative` as the default implementation):
Rust

pub trait ResultSource {
    fn name(&self) -> &str;
    fn fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn parse(&self, html: &str) -> Result<crate::models::Competition, crate::sources::ParsingError>;
    fn parse_date(&self, s: &str) -> Option<chrono::NaiveDate>;
}

Error Handling: Every failure must log a specific reason (NETWORK_ERROR, MISSING_REQUIRED_DATA, PARSING_ERROR, CRITICAL_VALIDATION_ERROR).

Performance: The binary format must be significantly smaller than the JSON output.

Documentation: Short "why-not-how" inline docstrings for all public functions.

Python Access: The library exposes a single high-level entry point, which serves as the **Source of Truth** for the library's operational flow:
- `load_competition_results(target_folder: str, url: str, date: Optional[str] = None, age_group: Optional[str] = None, style: Optional[str] = None, level: Optional[str] = None, download_html: bool = True, output_format: str = "json") -> None`:
  This orchestrator manages the full lifecycle:
  1. **Discovery & Crawling**: Resolves URLs, respects `robots.txt`, and skips already processed competitions via a local manifest.
  2. **Filtering**: Robust, case-insensitive filtering by `date`, `age_group`, `style`, and `level` against canonical IDs.
  3. **Safety Shield**: Every competition must pass the **Fidelity Gate** (structural integrity) and **Skating System Math Check** (rank recalculation) before archival. If `date >= 2026-01-01`, the 2026 dance count standards are automatically enforced.
  4. **Storage**: Archives results in a standardized folder structure: `target_folder/{Event_Name}_{Year}/` with filenames formatted as `{AgeGroup}_{Level}_{Style}.json`. If `download_html` is enabled, raw HTML sources are archived in a `raw/` subfolder.
