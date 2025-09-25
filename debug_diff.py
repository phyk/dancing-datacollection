import difflib
import pathlib
from dancing_datacollection.html_canonicalize import (
    canonical_deck_html,
    canonical_tabges_html,
    canonical_erg_html,
    canonical_ergwert_html,
)

def _load(dirpath: pathlib.Path, name: str) -> str:
    p = dirpath / name
    assert p.exists(), f"Missing {name} in {dirpath}"
    return p.read_text(encoding="utf-8")

def run_diff(test_name: str, test_file: str, golden_file: str, canonical_func):
    base = pathlib.Path(__file__).parent / "tests" / "51-1105_ot_hgr2dstd"
    html_content = _load(base, test_file)
    golden_content = _load(base, golden_file)

    canonical_output = canonical_func(html_content)

    diff = difflib.unified_diff(
        golden_content.splitlines(keepends=True),
        canonical_output.splitlines(keepends=True),
        fromfile=f"golden/{golden_file}",
        tofile=f"generated/{test_file}",
    )

    print(f"--- Diff for {test_name} ---")
    diff_output = "".join(diff)
    if diff_output:
        print(diff_output)
    else:
        print("No differences found.")
    print("-" * 20)

if __name__ == "__main__":
    run_diff("deck", "deck.htm", "deck.golden.htm", canonical_deck_html)
    run_diff("tabges", "tabges.htm", "tabges.golden.htm", canonical_tabges_html)
    run_diff("erg", "erg.htm", "erg.golden.htm", canonical_erg_html)
    run_diff("ergwert", "ergwert.htm", "ergwert.golden.htm", canonical_ergwert_html)