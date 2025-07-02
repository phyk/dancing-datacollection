# Project Brief: Dancing Competition Data Collection

## Overview
This project is a Python application designed to download, parse, and standardize data from dancing competitions. The goal is to collect comprehensive competition data from various public websites, process it into a consistent format, and store it efficiently for further analysis or reproduction of competition results.

---

## 1. Purpose & Value
- Automate the collection and standardization of dancing competition data.
- Enable rechecking and recreation of competition rankings from raw data.
- Support research, analytics, and archival use cases.

---

## 2. Data Requirements
### Data to Collect
- **Organization & Committee**: Details such as chairperson and committee members.
- **Participants**: All couples (not individuals), with both partners' details.
- **Judges**: Full list of judges for each competition.
- **Scores & Rounds**:
  - Scores for each round, including which couples advance.
  - Special handling for the final round (different scoring method).
  - All data necessary to reconstruct the official ranking.

---

## 3. Data Sources
- Data is sourced from a collection of public websites.
- Each website provides data as HTML (with embedded JavaScript).
- The relevant data is parseable from a single HTML file per competition.
- Multiple competitions may be held under a single event (base URL); all should be collected.
- No standardized API; URLs must be provided manually.

---

## 4. Input & Configuration
- User maintains a configuration file listing base URLs to check.
- On execution, the application checks all URLs for new competitions.
- If data for a competition is already present, it is skipped.
- Batch processing of multiple events is supported.

---

## 5. Output & Data Storage
- Output is organized in a directory structure:
  - Each competition has its own directory, named as `year-competition` (e.g., `2024-WorldOpen`).
  - Each directory contains multiple Parquet files, each representing a data category (e.g., `judges.parquet`, `scores.parquet`, `final_scoring.parquet`).
- Data is stored using the [Polars](https://www.pola.rs/) library for efficient Parquet handling.

---

## 6. Error Handling & Logging
- The system logs:
  - Number of new competitions parsed per URL location.
  - Progress of URL checking (with a progress bar on the CLI).
  - Results of data integrity checks on the output directory.
- A separate error log records detailed error messages and stack traces for troubleshooting.
- Malformed or missing data is logged with context for later review.

---

## 7. User Interface & Usability
- Command-line interface (CLI) with shallow, informative output.
- Progress bar shown when checking URLs.
- Summary of data checks and parsing results displayed after execution.

---

## 8. Environment & Dependencies
- Python application, runnable with `uvx`.
- Use [`uv`](https://github.com/astral-sh/uv) as the package management tool for dependency management and installation.
- Use standard Python libraries where possible.
- Use `polars` for data handling and Parquet output.
- Avoid unnecessary dependencies for compatibility and simplicity.

---

## 9. Extensibility & Future-Proofing
- Support for multiple HTML formats:
  - Parsing logic should be modular, allowing new parsers to be added for different HTML structures as formats evolve.
- Easy to add new data sources or competition formats in the future.

---

## 10. Testing & Validation
- Automated tests for all parsing logic.
- Data integrity checks for all output files:
  - Ensure all output conforms to a consistent schema.
  - Validate that all required fields are present and correctly typed.
- Include sample input HTML files and expected output for validation.

---

## 11. Naming, Structure, & Documentation
- Output directories: `year-competition` naming convention.
- Parquet files: Named by content (e.g., `judges.parquet`, `scores.parquet`).
- Codebase:
  - Well-documented code with docstrings and inline comments.
  - README file with:
    - CLI usage instructions.
    - Pointers to the most important code modules and their responsibilities.
    - Overview of the directory structure and output format.

---

## 12. Security & Privacy
- All data is public; no special access control or privacy requirements.

---

## 13. Delivery & Milestones
- No specific deadlines or milestones.

---

## 14. Summary
This specification provides a comprehensive guide for developing a robust, extensible, and well-documented Python application for collecting and standardizing dancing competition data. The focus is on reliability, maintainability, and ease of use for both end-users and future developers. 