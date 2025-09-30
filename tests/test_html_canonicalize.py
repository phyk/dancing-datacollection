import pathlib
import pytest
from dancing_datacollection.html_canonicalize import canonicalize_html

def _load(dirpath: pathlib.Path, name: str) -> str:
    p = dirpath / name
    assert p.exists(), f"Missing {name} in {dirpath}"
    return p.read_text(encoding="utf-8")

@pytest.fixture(scope="module")
def html_51():
    base = pathlib.Path(__file__).parent / "51-1105_ot_hgr2dstd"
    return {
        "deck": _load(base, "deck.htm"),
        "tabges": _load(base, "tabges.htm"),
        "erg": _load(base, "erg.htm"),
        "ergwert": _load(base, "ergwert.htm"),
        "deck_golden": _load(base, "deck.golden.htm"),
        "tabges_golden": _load(base, "tabges.golden.htm"),
        "erg_golden": _load(base, "erg.golden.htm"),
        "ergwert_golden": _load(base, "ergwert.golden.htm"),
    }

@pytest.mark.parametrize("html_type", ["deck", "tabges", "erg", "ergwert"])
def test_canonicalization(html_51, html_type):
    """Helper function to run a golden file test."""
    canonical_output = canonicalize_html(html_51[html_type])
    assert canonical_output == html_51[f"{html_type}_golden"]