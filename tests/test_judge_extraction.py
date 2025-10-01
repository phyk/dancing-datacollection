import os
from typing import Dict, List

import pytest

from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.parsing.deck import extract_judges_from_deck
from dancing_datacollection.parsing.ergwert import extract_judges_from_ergwert
from dancing_datacollection.parsing.parsing_utils import get_soup
from dancing_datacollection.parsing.tabges import extract_judges_from_tabges
from dancing_datacollection.parsing.wert_er import extract_judges_from_wert_er


def get_html(path: str) -> str:
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


# Ground truth fixture for judges, now local to this test file
@pytest.fixture(scope="module")
def true_judges() -> Dict[str, List[Judge]]:
    return {
        "51-1105_ot_hgr2dstd": [
            Judge(code="AT", name="Bärschneider, Marcus", club="TSC Blau-Gelb Hagen"),
            Judge(code="AX", name="Block, Robert", club="Schwarz-Rot-Club Wetzlar"),
            Judge(
                code="BW",
                name="Kirchwehm, Susanne",
                club="TSC Ostseebad Schönberg 1984",
            ),
            Judge(code="CJ", name="Mäser, Erich", club="TSC Rot-Gold Büdingen"),
            Judge(
                code="EK",
                name="Landauer, Peter",
                club="Tanzsportgemeinschaft Bavaria, Augsburg",
            ),
        ],
        "52-1105_ot_hgr2cstd": [
            Judge(
                code="AR",
                name="Appel, Hans-Jürgen",
                club="TTC Gelb-Weiss i. Post-SV Hannover",
            ),
            Judge(code="CH", name="Mak, Annabel", club="Grün-Gold-Casino Wuppertal"),
            Judge(code="DC", name="Schöke, Manuel", club="TTC München"),
            Judge(code="DV", name="Becker, Marc", club="TTC Fortis Nova Maintal"),
            Judge(
                code="EY", name="Schwarz, Sonja", club="TSZ Blau-Gold Casino, Darmstadt"
            ),
        ],
        "53-1105_ot_hgr2bstd": [
            Judge(
                code="BI", name="Fleischer, Georg", club="Grün-Gold-Casino Wuppertal"
            ),
            Judge(
                code="CP",
                name="Peinke-Dean, Lutz",
                club="Tanzsportklub Residenz Dresden",
            ),
            Judge(code="DK", name="Wenzel, Harald", club="Rot-Weiss-Klub Kassel"),
            Judge(code="DL", name="Wied, Dr. Andrea", club="Markgräfler TSC, Müllheim"),
            Judge(code="DR", name="Zuber, Dr. Pascal", club="TSC Metropol Hofheim"),
            Judge(code="EL", name="Lein, Roland", club="TC Rot-Gold Würzburg"),
            Judge(code="EU", name="Reher, Thomas", club="TSC Werne"),
        ],
    }


@pytest.mark.parametrize(
    "sample_dir",
    [
        pytest.param(d)
        for d in ["51-1105_ot_hgr2dstd", "52-1105_ot_hgr2cstd", "53-1105_ot_hgr2bstd"]
    ],
)
def test_extract_judges(
    sample_dir: str, test_dir: str, true_judges: Dict[str, List[Judge]]
) -> None:
    tabges_path = os.path.join(test_dir, sample_dir, "tabges.htm")
    if not os.path.exists(tabges_path):
        pytest.skip(f"Missing {tabges_path}")
    html = get_html(tabges_path)
    soup = get_soup(html)
    judges = extract_judges_from_tabges(soup)
    gt_judges = true_judges[sample_dir]
    assert isinstance(judges, list)
    assert all(isinstance(j, Judge) for j in judges)
    assert judges, f"No judges extracted from {tabges_path}"
    keys = set()
    for j in judges:
        key = (j.code, j.name)
        assert key not in keys, f"Duplicate judge {key} in {tabges_path}"
        keys.add(key)
    # Check all ground truth judges are present (partial match)
    for gt_judge in gt_judges:
        assert any(j.matches_partial(gt_judge) for j in judges), (
            f"Missing judge: {gt_judge}"
        )


@pytest.mark.parametrize(
    "sample_dir",
    [
        pytest.param(d)
        for d in ["51-1105_ot_hgr2dstd", "52-1105_ot_hgr2cstd", "53-1105_ot_hgr2bstd"]
    ],
)
def test_extract_judges_from_deck(
    sample_dir: str, test_dir: str, true_judges: Dict[str, List[Judge]]
) -> None:
    deck_path = os.path.join(test_dir, sample_dir, "deck.htm")
    if not os.path.exists(deck_path):
        pytest.skip(f"Missing {deck_path}")
    html = get_html(deck_path)
    soup = get_soup(html)
    judges = extract_judges_from_deck(soup)
    gt_judges = true_judges[sample_dir]
    assert isinstance(judges, list)
    assert all(isinstance(j, Judge) for j in judges)
    assert judges, f"No judges extracted from {deck_path}"
    keys = set()
    for j in judges:
        key = (j.code, j.name)
        assert key not in keys, f"Duplicate judge {key} in {deck_path}"
        keys.add(key)
    # Check all ground truth judges are present (full match)
    for gt_judge in gt_judges:
        assert any(j.matches_full(gt_judge) for j in judges), (
            f"Missing judge: {gt_judge}"
        )


@pytest.mark.parametrize(
    "sample_dir",
    [
        pytest.param(d)
        for d in ["51-1105_ot_hgr2dstd", "52-1105_ot_hgr2cstd", "53-1105_ot_hgr2bstd"]
    ],
)
def test_extract_judges_from_ergwert(
    sample_dir: str, test_dir: str, true_judges: Dict[str, List[Judge]]
) -> None:
    ergwert_path = os.path.join(test_dir, sample_dir, "ergwert.htm")
    if not os.path.exists(ergwert_path):
        pytest.skip(f"Missing {ergwert_path}")
    html = get_html(ergwert_path)
    soup = get_soup(html)
    judges = extract_judges_from_ergwert(soup)
    gt_judges = true_judges[sample_dir]
    assert isinstance(judges, list)
    assert all(isinstance(j, Judge) for j in judges)
    assert judges, f"No judges extracted from {ergwert_path}"
    keys = set()
    for j in judges:
        key = (j.code, j.name)
        assert key not in keys, f"Duplicate judge {key} in {ergwert_path}"
        keys.add(key)
    # Check all ground truth judges are present (partial match)
    for gt_judge in gt_judges:
        assert any(j.matches_partial(gt_judge) for j in judges), (
            f"Missing judge: {gt_judge}"
        )


@pytest.mark.parametrize(
    "sample_dir",
    [
        pytest.param(d)
        for d in ["51-1105_ot_hgr2dstd", "52-1105_ot_hgr2cstd", "53-1105_ot_hgr2bstd"]
    ],
)
def test_extract_judges_from_wert_er(
    sample_dir: str, test_dir: str, true_judges: Dict[str, List[Judge]]
) -> None:
    wert_er_path = os.path.join(test_dir, sample_dir, "wert_er.htm")
    if not os.path.exists(wert_er_path):
        pytest.skip(f"Missing {wert_er_path}")
    html = get_html(wert_er_path)
    soup = get_soup(html)
    judges = extract_judges_from_wert_er(soup)
    gt_judges = true_judges[sample_dir]
    assert isinstance(judges, list)
    assert all(isinstance(j, Judge) for j in judges)
    assert judges, f"No judges extracted from {wert_er_path}"
    keys = set()
    for j in judges:
        key = (j.code, j.name)
        assert key not in keys, f"Duplicate judge {key} in {wert_er_path}"
        keys.add(key)
    # Check all ground truth judges are present (partial match)
    for gt_judge in gt_judges:
        assert any(j.matches_partial(gt_judge) for j in judges), (
            f"Missing judge: {gt_judge}"
        )