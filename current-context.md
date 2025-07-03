# Project Execution
- All runs of the project are done with `uv run python ...` to ensure the correct environment and dependencies are used.
- All tests are run with `uv run pytest ...` to ensure the correct environment and dependencies are used.

# For the current task summary and HTML file relevance, see current-tasks.md

# Competition Dance Info
- Any competition may contain either the Ballroom dances or the Latin dances.
- **Ballroom dances:** Slow Waltz, Tango, Viennese Waltz, Slow Foxtrott, Quick Step
- **Latin dances:** Samba, Cha cha cha, Rumba, Paso Doble, Jive
- All competitions contain at least 3 and at most 5 dances.

## Test Case 51 (tests/51-1105_ot_hgr2dstd)
- After any change, especially to output format or parsing, you should run this test case.
- All current checks on logs (debug, info, error) and output files (especially participants.parquet) should be verified.
- This is required for the previous change: participants.parquet must have columns name_one, name_two, club, number, with number as integer.

## Test Case 52 (tests/52-1105_ot_hgr2cstd)
- Use this test case to validate output and log consistency for a different competition structure.
- All checks on logs and output files (especially participants.parquet, judges, committee, scores, etc.) should be verified.

## Test Case 53 (tests/53-1105_ot_hgr2bstd)
- Use this test case to validate output and log consistency for a third competition structure.
- All checks on logs and output files (especially participants.parquet, judges, committee, scores, etc.) should be verified.

---

## Reference Test Case: Local Run of Tool (51)

The canonical way to validate the tool's output and logging is to run the tool locally on the 51 test case directory. This ensures that all parsing, output, and logging logic is working as expected on a known-good dataset.

**How to run:**

```sh
uv run python -m dancing_datacollection.main --local-dir tests/51-1105_ot_hgr2dstd
```

**What this does:**
- Processes all relevant .htm files in the test case directory.
- Extracts and saves parquet files for judges, committee, scores, final scoring, and participants.
- Validates the schema of each output file (e.g., participants.parquet must have columns name_one, name_two, club, number, with number as integer).
- Writes detailed logs to the logs/ directory, including info, debug, and error messages.

**Why this matters:**
- This test case is the reference for all output and log validation. Any change to parsing or output logic must be checked against this run to ensure nothing breaks.
- If this test fails (e.g., schema validation errors, missing logs, or incorrect output), the change should not be merged until fixed.

---

# Current Task Summary

## Modularization Status

### Extraction logic still implemented directly in TopTurnierParser (not in submodules):
- extract_committee: Committee extraction is implemented directly in TopTurnierParser, not in a submodule.
- extract_scores: Score extraction is implemented directly in TopTurnierParser, not in a submodule.
- extract_final_scoring: Final scoring extraction is implemented directly in TopTurnierParser, not in a submodule.
- parse_tabges_all, parse_erg_all, parse_deck_all: These are exploratory/debugging methods implemented directly in TopTurnierParser.
- extract_finalists_from_erg: Finalist extraction from erg.htm is implemented directly in TopTurnierParser.
- deduplicate_judges, make_judge: Utility methods for judge handling are implemented directly in TopTurnierParser.

### All participant and judge extraction for erg.htm, ergwert.htm, tabges.htm, wert_er.htm, and deck.htm is now modularized and routed through submodules.

---

# HTML File Relevance
- **deck.htm**: Used to extract judges and committee information.
- **tabges.htm**: Used to extract scores for the first rounds and the final round (per round, per judge, per couple, per dance).
- **erg.htm**: Used to extract participants.
- **ergwert.htm**: Used to extract final scoring.
- **index.htm**, **menu.htm**: Used for navigation, not directly parsed for data.
- **wert_er.htm**: Sometimes contains additional scoring info (rarely used).
- **endrunde.jpg**: Image, not parsed.

# Extraction/Refactoring Template: Judges
- Normalize and validate all judge data (code, name, club) at the dataclass level.
- Implement robust extraction methods for each relevant HTML source (deck, tabges, ergwert, wert_er), ensuring deduplication and error handling.
- Centralize deduplication and safe dataclass creation in utility functions.
- Add type hints and docstrings for maintainability.
- Write pytest-based unit tests for all extraction methods using real test cases, checking type, uniqueness, and non-emptiness.
- Ensure all tests pass in CI and local runs.

# Slow Waltz (Langsamer Walzer) - ergwert.htm header and first data row

Header row (dances):
<TD class="td2w" colspan="6" nowrap>Langsamer Walzer</TD>
<TD class="td2ww">LW</TD>

Judge code row (for Slow Waltz):
<TD class="td2w">AT<span class="tooltip2w">B&auml;rschneider, Marcus</span></TD>
<TD class="td2w">AX<span class="tooltip2w">Block, Robert</span></TD>
<TD class="td2w">BW<span class="tooltip2w">Kirchwehm, Susanne</span></TD>
<TD class="td2w">CJ<span class="tooltip2w">M&auml;ser, Erich</span></TD>
<TD class="td2w">EK<span class="tooltip2w">Landauer, Peter</span></TD>
<TD class="td2www">Su</TD>

First data row (couple 610, Slow Waltz columns):
<TD class="td5w tcol1">1<br>x<br>x<br>x</TD>
<TD class="td5w tcol1">1<br>x<br>x<br>x</TD>
<TD class="td5w tcol1">1<br>x<br>x<br>x</TD>
<TD class="td5w tcol1">3<br>x<br>x<br>x</TD>
<TD class="td5w tcol1">1<br>x<br>x<br>x</TD>
<TD class="td3www">1,0<br>5<br>5<br>5</TD>

# End Slow Waltz context
