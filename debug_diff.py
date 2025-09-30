import pathlib
import difflib
from dancing_datacollection.html_canonicalize import canonical_deck_html, canonical_tabges_html, canonical_erg_html, canonical_ergwert_html

def _load(dirpath: pathlib.Path, name: str) -> str:
    p = dirpath / name
    assert p.exists(), f"Missing {name} in {dirpath}"
    return p.read_text(encoding="utf-8")

def main():
    base = pathlib.Path(__file__).parent / "tests" / "51-1105_ot_hgr2dstd"

    # Choose which test case to debug
    # test_case = "deck"
    test_case = "tabges"
    # test_case = "erg"
    # test_case = "ergwert"

    if test_case == "deck":
        html_file = "deck.htm"
        golden_file = "deck.golden.htm"
        canonical_func = canonical_deck_html
    elif test_case == "tabges":
        html_file = "tabges.htm"
        golden_file = "tabges.golden.htm"
        canonical_func = canonical_tabges_html
    elif test_case == "erg":
        html_file = "erg.htm"
        golden_file = "erg.golden.htm"
        canonical_func = canonical_erg_html
    elif test_case == "ergwert":
        html_file = "ergwert.htm"
        golden_file = "ergwert.golden.htm"
        canonical_func = canonical_ergwert_html
    else:
        raise ValueError(f"Unknown test case: {test_case}")


    html_input = _load(base, html_file)
    golden_output = _load(base, golden_file)

    canonical_output = canonical_func(html_input)

    sm = difflib.SequenceMatcher(None, canonical_output, golden_output)

    print(f"--- Diff for {test_case} ---")
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag != 'equal':
            print(f'{tag:7} a[{i1}:{i2}] --> b[{j1}:{j2}]')
            print("--- Got ---")
            print(f'{canonical_output[i1-40:i2+40]!r}')
            print("--- Expected ---")
            print(f'{golden_output[j1-40:j2+40]!r}')
            print("-" * 20)
    print("--- End Diff ---")

if __name__ == "__main__":
    main()