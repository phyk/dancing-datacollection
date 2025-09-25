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
    }

def test_canonical_deck_idempotent(html_51):
    original_html = html_51["deck"]
    canonical_once = canonical_deck_html(original_html)
    canonical_twice = canonical_deck_html(canonical_once)
    assert canonical_once == canonical_twice

def test_canonical_tabges_idempotent(html_51):
    original_html = html_51["tabges"]
    canonical_once = canonical_tabges_html(original_html)
    canonical_twice = canonical_tabges_html(canonical_once)
    assert canonical_once == canonical_twice

def test_canonical_erg_idempotent(html_51):
    original_html = html_51["erg"]
    canonical_once = canonical_erg_html(original_html)
    canonical_twice = canonical_erg_html(canonical_once)
    assert canonical_once == canonical_twice

def test_canonical_ergwert_idempotent(html_51):
    original_html = html_51["ergwert"]
    canonical_once = canonical_ergwert_html(original_html)
    canonical_twice = canonical_ergwert_html(canonical_once)
    assert canonical_once == canonical_twice