import pathlib
from dancing_datacollection.parsing_topturnier import TopTurnierParser
from dancing_datacollection.html_canonicalize import (
    canonical_deck_html,
    canonical_tabges_html,
    canonical_erg_html,
    canonical_ergwert_html,
    parse_ergwert_header,
)
from dancing_datacollection.html_generate import (
    generate_deck_html,
    generate_tabges_html,
    generate_erg_html,
    generate_ergwert_html,
)


def _load(dirpath: pathlib.Path, name: str) -> str:
    p = dirpath / name
    assert p.exists(), f"Missing {name} in {dirpath}"
    return p.read_text(encoding="utf-8")


def _test_recreation(dirpath: pathlib.Path):
    parser = TopTurnierParser()

    # Load original HTML files
    deck_orig = _load(dirpath, "deck.htm")
    tabges_orig = _load(dirpath, "tabges.htm")
    erg_orig = _load(dirpath, "erg.htm")
    ergwert_orig = _load(dirpath, "ergwert.htm")

    # Parse data from original HTML
    p_deck = parser.extract_judges(deck_orig, filename="deck.htm")
    p_tabges_part, _ = parser.extract_participants(tabges_orig, filename="tabges.htm")
    p_tabges_judges = parser.extract_judges(tabges_orig, filename="tabges.htm")
    p_erg_part, _ = parser.extract_participants(erg_orig, filename="erg.htm")
    p_ergwert_part, _ = parser.extract_participants(ergwert_orig, filename="ergwert.htm")
    p_ergwert_judges = parser.extract_judges(ergwert_orig, filename="ergwert.htm")
    p_scores = parser.extract_scores(ergwert_orig, filename="ergwert.htm")

    from dancing_datacollection.data_defs.score import FinalRoundScore
    p_final_scores = [s for s in p_scores if isinstance(s, FinalRoundScore)]

    # Generate new HTML from parsed data
    deck_new = generate_deck_html(p_deck)
    tabges_new = generate_tabges_html(p_tabges_part, p_tabges_judges)
    erg_new = generate_erg_html(p_erg_part)
    dances_en, codes_by_dance = parse_ergwert_header(ergwert_orig)
    ergwert_new = generate_ergwert_html(
        p_ergwert_part, p_ergwert_judges, p_final_scores,
        dance_names_english=dances_en, judge_codes_per_dance=codes_by_dance,
    )

    # Compare canonicalized original with canonicalized new
    assert canonical_deck_html(deck_orig) == canonical_deck_html(deck_new)
    assert canonical_tabges_html(tabges_orig) == canonical_tabges_html(tabges_new)
    assert canonical_erg_html(erg_orig) == canonical_erg_html(erg_new)
    assert canonical_ergwert_html(ergwert_orig) == canonical_ergwert_html(ergwert_new)


def test_recreation_for_51():
    base = pathlib.Path(__file__).parent / "51-1105_ot_hgr2dstd"
    _test_recreation(base)


def test_recreation_for_52():
    base = pathlib.Path(__file__).parent / "52-1105_ot_hgr2cstd"
    _test_recreation(base)


def test_recreation_for_53():
    base = pathlib.Path(__file__).parent / "53-1105_ot_hgr2bstd"
    _test_recreation(base)