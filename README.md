# Dancing Competition Data Collection

## Overview
This Python application downloads, parses, and standardizes data from dancing competitions, storing results in a structured, analysis-ready format. It is designed for extensibility and reliability, following the detailed specification in `project-brief.md`.

## Quickstart
1. **Install dependencies:**
   ```sh
   uv pip install -r requirements.txt
   ```
2. **Configure URLs:**
   - Edit `config.toml` to add base URLs of competition events (see below).
3. **Run the application:**
   ```sh
   uvx python -m dancing_datacollection
   ```

## Configuration File
- The configuration file (`config.toml`) lists base URLs to check for new competitions.
- Example:
  ```toml
  [sources]
  urls = [
    "https://hessen-tanzt.de/media/ht2024/"
  ]
  ```

## Output Structure
- Output is organized as:
  ```
  data/
    2024-HessenTanzt/
      judges.parquet
      scores.parquet
      final_scoring.parquet
      ...
  logs/
    app.log
    error.log
  ```

## Further Details
See `project-brief.md` for the full specification, requirements, and architecture.
