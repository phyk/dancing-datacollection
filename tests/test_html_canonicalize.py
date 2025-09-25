import pathlib
import pytest
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

def _run_golden_test(html, golden_html, canonical_func):
    """Helper function to run a golden file test."""
    canonical_output = canonical_func(html)
    assert canonical_output == golden_html

def test_canonical_deck(html_51):
    _run_golden_test(
        html_51["deck"],
        html_51["deck_golden"],
        canonical_deck_html,
    )

def test_canonical_tabges(html_51):
    _run_golden_test(
        html_51["tabges"],
        html_51["tabges_golden"],
        canonical_tabges_html,
    )

def test_canonical_erg(html_51):
    _run_golden_test(
        html_51["erg"],
        html_51["erg_golden"],
        canonical_erg_html,
    )

def test_canonical_ergwert(html_51):
    _run_golden_test(
        html_51["ergwert"],
        html_51["ergwert_golden"],
        canonical_ergwert_html,
    )