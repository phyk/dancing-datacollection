# Project Execution
- All runs of the project are done with `uv run python ...` to ensure the correct environment and dependencies are used.

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

# Current Task Summary

# Participant Extraction Test Status & TODOs
- [x] erg.htm: Extraction robust, tests pass
- [x] ergwert.htm: Extraction robust, tests pass
- [ ] tabges.htm: Extraction fails, no participants extracted, needs logic update
- [ ] wert_er.htm: Extraction fails, no participants extracted, needs logic update

Next step: Improve tabges.htm extraction to correctly extract participants based on file structure.
 