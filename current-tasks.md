# Current Task Summary

After changing code, run the following two commands:
- uv tool run ruff check --fix
- uv tool run ruff format
Fix any errors reported by the ruff check, then rerun tests to ensure the changes did not break the tests.

## Modularization Status

### Extraction logic still implemented directly in TopTurnierParser (not in submodules):
- extract_scores: Score extraction is implemented directly in TopTurnierParser, not in a submodule.
- extract_final_scoring: Final scoring extraction is implemented directly in TopTurnierParser, not in a submodule.
- parse_tabges_all, parse_erg_all, parse_deck_all: These are exploratory/debugging methods implemented directly in TopTurnierParser.
- extract_finalists_from_erg: Finalist extraction from erg.htm is implemented directly in TopTurnierParser.
- deduplicate_judges, make_judge: Utility methods for judge handling are implemented directly in TopTurnierParser.

### All participant, judge, and committee extraction for erg.htm, ergwert.htm, tabges.htm, wert_er.htm, and deck.htm is now modularized and routed through submodules.

---

## Modularization Steps

1. **Add Tests with Ground Truth:**
   - For each function to be modularized, first write or update tests that check its output against ground truth data for all relevant test cases.
2. **Move Code to Submodule:**
   - Move the extraction logic from TopTurnierParser to the appropriate submodule (e.g., parsing/ergwert.py, parsing/tabges.py, etc.).
   - Update TopTurnierParser to delegate to the new submodule function.
3. **Check All Relevant Files and Add Extractors:**
   - For each type of data, check all relevant HTML files (e.g., erg.htm, ergwert.htm, tabges.htm, wert_er.htm, deck.htm) to see if the data is present.
   - Ensure that an extractor function exists in the appropriate submodule for each file type where the data can be extracted.
