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

Fidelity Gate (Validation):

A competition is invalid if it lacks Officials, Judges, or Results.

Structure Check: If a competition level (e.g., "Standard A") requires 5 dances but only 3 are found, log a PARSING_ERROR and do not save.

Localization Module:

Store German/English aliases for Age Groups and Levels in a separate, accessible module to allow easy language switching.

Input & Output Input: A configuration file (.toml or .yaml) containing source URLs and site-specific parameters.
Output Formats:

Human-Readable: .jsonl (JSON Lines) for easy diffing and inspection.

Optimized Binary: postcard or MessagePack for high-speed, low-disk-space storage.

Abstract Design: ResultSource Trait To ensure efficient abstractions, all scrapers must implement this trait:
Rust

pub trait ResultSource {
    fn name(&self) -> &str;
    fn fetch(&self, url: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn parse(&self, html: &str) -> Result<crate::models::Event, crate::sources::ParsingError>;
}

Error Handling: Every failure must log a specific reason (NETWORK_ERROR, MISSING_REQUIRED_DATA, PARSING_ERROR).

Performance: The binary format must be significantly smaller than the JSONL output.

Documentation: Short "why-not-how" inline docstrings for all public functions.

Python Access: The library must be importable in Python with accessible getters for all competition data.
