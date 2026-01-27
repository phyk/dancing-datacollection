Task Specification: dancing-datacollection
1. Context & Purpose
Goal: Create a high-performance Python library (powered by Rust/PyO3) and CLI tool to scrape, parse, and archive dance competition results. North Star: Data Fidelity. The primary objective is the 100% accurate reproduction of online results into a machine-readable format. If data cannot be verified as accurate/complete, it should not be stored.

2. Tech Stack & Environment
Core: Rust (for performance and safety).

Bindings: pyo3 / maturin to expose the library to Python.

CLI: clap (Rust) or click/typer (Python via the bindings).

Standards: * Formatting: Strict adherence to rustfmt.

Architecture: Flat, efficient abstractions. Avoid "over-engineering" or deep inheritance trees. Prioritize "Data-Oriented Design."

3. Functional Requirements
Input & Configuration
Config File: A .toml or .yaml file containing a list of source URLs (competition websites) and site-specific scraping parameters.

Crawling: * Must respect robots.txt.

Must implement a "Smart Skip" logic: Check local storage before downloading to avoid redundant network calls.

Output & Storage
Dual-Format Support:

Human-Readable: Structured text (e.g., CSV or JSON) for easy inspection.

Optimized Binary: A space-efficient format (e.g., Parquet, MessagePack, or a custom Protobuf-based binary).

Granularity: Data can be split into multiple files (e.g., metadata.json, scores.bin, judges.csv) as long as they are linked by a unique Competition ID.

4. Logic & Business Rules
Fidelity First: Capture every available data point (heats, individual judge scores, penalties, etc.).

Missing Data Policy:

Strict: If "Required Fields" (defined per source) are missing, the competition record is invalid.

Permissive: Non-essential metadata (e.g., sponsor names) can be null.

Efficiency: Use Rust’s multi-threading (e.g., rayon or tokio) for concurrent scraping where the site’s robots.txt allows.

5. Error Handling & Logging
Validation Gate: No data should be written to the final storage if it fails structural validation.

Detailed Failure Logs: Every failure must be categorized and logged with:

NETWORK_ERROR: Connectivity or Timeout.

MISSING_REQUIRED_DATA: Scraper found the page but key elements were absent.

PARSING_ERROR: HTML structure has changed or is malformed.

Recovery: Log the specific URL that failed so it can be retried or debugged manually.

6. Definition of Done (DoD)
[ ] Rust Logic: All parsing and data-handling logic covered by cargo test.

[ ] Python Bindings: Basic smoke tests ensuring the pyo3 module imports and executes in a Python environment.

[ ] Performance: Binary format must show significant disk-space savings over the human-readable format.

[ ] Documentation: Short, "why-not-how" inline docstrings for all public functions.

[ ] CLI: A working download command that accepts a config file path.

7. Implementation Roadmap (Suggested)
Phase 1: Define the core Rust structs for competition data (The Schema).

Phase 2: Implement the Config parser and the robots.txt crawler.

Phase 3: Create the first "Source" scraper.

Phase 4: Build the Binary/CSV serialization layers.

Phase 5: Wrap with pyo3 for Python access.
