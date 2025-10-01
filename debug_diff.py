import difflib
import logging
import pathlib

from dancing_datacollection.html_canonicalize import canonicalize_html

logging.basicConfig(level=logging.INFO, format="%(message)s")


def _load(dirpath: pathlib.Path, name: str) -> str:
    p = dirpath / name
    if not p.exists():
        message = f"Missing {name} in {dirpath}"
        raise FileNotFoundError(message)
    return p.read_text(encoding="utf-8")


def main() -> None:
    base = pathlib.Path(__file__).parent / "tests" / "51-1105_ot_hgr2dstd"

    test_case = "tabges"

    if test_case == "deck":
        html_file = "deck.htm"
        golden_file = "deck.golden.htm"
    elif test_case == "tabges":
        html_file = "tabges.htm"
        golden_file = "tabges.golden.htm"
    elif test_case == "erg":
        html_file = "erg.htm"
        golden_file = "erg.golden.htm"
    elif test_case == "ergwert":
        html_file = "ergwert.htm"
        golden_file = "ergwert.golden.htm"
    else:
        message = f"Unknown test case: {test_case}"
        raise ValueError(message)

    html_input = _load(base, html_file)
    golden_output = _load(base, golden_file)

    canonical_output = canonicalize_html(html_input)

    sm = difflib.SequenceMatcher(None, canonical_output, golden_output)

    logging.info("--- Diff for %s ---", test_case)
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag != "equal":
            logging.info("%7s a[%d:%d] --> b[%d:%d]", tag, i1, i2, j1, j2)
            logging.info("--- Got ---")
            logging.info("%r", canonical_output[i1 - 40 : i2 + 40])
            logging.info("--- Expected ---")
            logging.info("%r", golden_output[j1 - 40 : j2 + 40])
            logging.info("-" * 20)
    logging.info("--- End Diff ---")


if __name__ == "__main__":
    main()