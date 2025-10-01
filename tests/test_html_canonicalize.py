import pathlib
from typing import Dict

import pytest

from dancing_datacollection.html_canonicalize import canonicalize_html


def _load(dirpath: pathlib.Path, name: str) -> str:
    p = dirpath / name
    assert p.exists(), f"Missing {name} in {dirpath}"
    return p.read_text(encoding="utf-8")


def _load_golden_data(base_dir_name: str) -> Dict[str, str]:
    base = pathlib.Path(__file__).parent / base_dir_name
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


@pytest.fixture(scope="module")
def html_51() -> Dict[str, str]:
    return _load_golden_data("51-1105_ot_hgr2dstd")


@pytest.fixture(scope="module")
def html_52() -> Dict[str, str]:
    return _load_golden_data("52-1105_ot_hgr2cstd")


@pytest.fixture(scope="module")
def html_53() -> Dict[str, str]:
    return _load_golden_data("53-1105_ot_hgr2bstd")


@pytest.mark.parametrize("html_data_fixture", ["html_51", "html_52", "html_53"])
@pytest.mark.parametrize("html_type", ["deck", "tabges", "erg", "ergwert"])
def test_canonicalization(
    html_data_fixture: str, html_type: str, request: pytest.FixtureRequest
) -> None:
    """Helper function to run a golden file test."""
    html_data = request.getfixturevalue(html_data_fixture)
    canonical_output = canonicalize_html(html_data[html_type])
    assert canonical_output == html_data[f"{html_type}_golden"]
