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


def _parse_all(dirpath: pathlib.Path):
    parser = TopTurnierParser()
    deck = _load(dirpath, "deck.htm")
    tabges = _load(dirpath, "tabges.htm")
    erg = _load(dirpath, "erg.htm")
    ergwert = _load(dirpath, "ergwert.htm")

    participants, event_name = parser.extract_participants(erg, filename="erg.htm")
    judges_from_deck = parser.extract_judges(deck, filename="deck.htm")
    judges_from_tabges = parser.extract_judges(tabges, filename="tabges.htm")
    judges_from_ergwert = parser.extract_judges(ergwert, filename="ergwert.htm")
    scores = parser.extract_scores(ergwert, filename="ergwert.htm")

    # Prefer judges with most info, merge simple union
    judge_keys = set()
    judges = []
    for lst in (judges_from_deck, judges_from_tabges, judges_from_ergwert):
        for j in lst:
            key = (j.code, j.name)
            if key not in judge_keys:
                judge_keys.add(key)
                judges.append(j)
    return participants, judges, scores, event_name


def _canon_compare(dirpath: pathlib.Path):
    deck = _load(dirpath, "deck.htm")
    tabges = _load(dirpath, "tabges.htm")
    erg = _load(dirpath, "erg.htm")
    ergwert = _load(dirpath, "ergwert.htm")

    c_deck = canonical_deck_html(deck)
    c_tabges = canonical_tabges_html(tabges)
    c_erg = canonical_erg_html(erg)
    c_ergwert = canonical_ergwert_html(ergwert)

    # Compare canonical(original) == canonical(original) roundtrip (idempotence)
    assert c_deck == canonical_deck_html(c_deck)
    assert c_tabges == canonical_tabges_html(c_tabges)
    assert c_erg == canonical_erg_html(c_erg)
    assert c_ergwert == canonical_ergwert_html(c_ergwert)

    # Also ensure canonical(original) is deterministic across runs
    assert c_deck == canonical_deck_html(deck)
    assert c_tabges == canonical_tabges_html(tabges)
    assert c_erg == canonical_erg_html(erg)
    assert c_ergwert == canonical_ergwert_html(ergwert)

    # Also confirm canonical equals generator output from parsed entities
    parser = TopTurnierParser()
    p_deck = parser.extract_judges(deck, filename="deck.htm")
    p_tabges_part, _ = parser.extract_participants(tabges, filename="tabges.htm")
    p_tabges_judges = parser.extract_judges(tabges, filename="tabges.htm")
    p_erg_part, _ = parser.extract_participants(erg, filename="erg.htm")
    p_ergwert_part, _ = parser.extract_participants(ergwert, filename="ergwert.htm")
    p_ergwert_judges = parser.extract_judges(ergwert, filename="ergwert.htm")
    p_scores = parser.extract_scores(ergwert, filename="ergwert.htm")

    from dancing_datacollection.data_defs.score import FinalRoundScore

    p_final_scores = [s for s in p_scores if isinstance(s, FinalRoundScore)]

    assert c_deck == generate_deck_html(p_deck)
    assert c_tabges == generate_tabges_html(p_tabges_part, p_tabges_judges)
    assert c_erg == generate_erg_html(p_erg_part)
    dances_en, codes_by_dance = parse_ergwert_header(ergwert)
    assert c_ergwert == generate_ergwert_html(
        p_ergwert_part, p_ergwert_judges, p_final_scores,
        dance_names_english=dances_en, judge_codes_per_dance=codes_by_dance,
    )


def test_roundtrip_for_51():
    base = pathlib.Path(__file__).parent / "51-1105_ot_hgr2dstd"
    _canon_compare(base)


def test_roundtrip_for_52():
    base = pathlib.Path(__file__).parent / "52-1105_ot_hgr2cstd"
    _canon_compare(base)


def test_roundtrip_for_53():
    base = pathlib.Path(__file__).parent / "53-1105_ot_hgr2bstd"
    _canon_compare(base)


