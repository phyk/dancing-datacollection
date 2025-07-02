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

# HTML File Relevance
- **deck.htm**: Used to extract judges and committee information.
- **tabges.htm**: Used to extract scores for the first rounds and the final round (per round, per judge, per couple, per dance).
- **erg.htm**: Used to extract participants.
- **ergwert.htm**: Used to extract final scoring.
- **index.htm**, **menu.htm**: Used for navigation, not directly parsed for data.
- **wert_er.htm**: Sometimes contains additional scoring info (rarely used).
- **endrunde.jpg**: Image, not parsed. 