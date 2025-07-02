# Current Context (Most Important Items)

## Unified Parsing Architecture
- All parsing (participants, judges, committee, scores, final scoring) is handled by the TopTurnierParser class in parsing_topturnier.py.
- Both local file and URL-based workflows use the same TopTurnierParser instance and its methods.
- The codebase is structured so that new formats can be supported by adding new parser classes inheriting from CompetitionParser.

## Debug Logging
- All parsing logic in TopTurnierParser includes detailed debug logging using the 'parsing_debug' logger.
- Intermediate values for all parsed fields (e.g., event name, couple info, club, number, names, name_one, name_two, judges, committee, scores, final scoring) are logged.
- The debug log (logs/parsing_debug.log) provides a comprehensive trace of the parsing process for debugging and verification.

## Utilities and Code Deduplication
- Repeated parsing logic (soup creation, club/number extraction, name splitting, span extraction) has been moved to parsing_utils.py as utility functions.
- TopTurnierParser uses these utilities to avoid code duplication and centralize parsing helpers.

## Removal of Unused Code
- parsing_participants.py has been removed as it was not used in the current workflow.
- All parsing is now routed through TopTurnierParser.

## Logging Setup
- Logging is configured via `setup_logging` in `parsing_utils.py`.
- Three loggers are set up:
  - **Root logger**: Logs INFO and above to `logs/app.log` and to the console.
  - **Error logger** (`logging.getLogger('error')`): Logs ERROR and above to `logs/error.log`.
  - **Parsing debug logger** (`logging.getLogger('parsing_debug')`): Logs DEBUG and above to `logs/parsing_debug.log`.
- Log files are created in the `logs/` directory. The setup ensures handlers are not duplicated if called multiple times.

## Usage in Code
- `main.py` calls `setup_logging()` at startup.
- Most modules use `import logging` and log via either the root logger, a module logger, or the named loggers above.
- `main.py` uses both the root logger and the error logger for general and error messages, respectively.
- `parsing_participants.py` uses the `parsing_debug` logger for detailed debug output during parsing.
- `output.py` logs info and errors about file output and schema validation.
- All logs and errors are written to the `logs/` directory as per the README and `.gitignore`.

## Summary
- Logging is centralized and multi-level (INFO, ERROR, DEBUG), with separate files for general, error, and parsing debug logs.
- Logging is used throughout the codebase for status, error, and debug information.

## parsing_participants.py (Investigation)
- Contains functions to extract participants, judges, committee, and scores from HTML using BeautifulSoup.
- Uses the 'parsing_debug' logger for detailed debug output throughout all parsing functions.
- The main function `extract_participants` currently raises an Exception ('TEST: extract_participants called') at the top, so it does not execute any parsing logic. This is likely for testing error handling or as a placeholder.
- After the exception, the function contains detailed debug logging for every parsing step, including row/cell content, name/club/number extraction, and final participant construction.
- Other functions (`extract_judges`, `extract_committee`, `extract_scores`, `extract_scores_from_tabges`) are implemented and use debug logging extensively to trace parsing steps and extracted data.
- The logging approach is consistent: each function logs its start, key parsing steps, and summary (e.g., number of items found).
- The current state means that participant extraction will always fail with an exception until the test line is removed or replaced. 