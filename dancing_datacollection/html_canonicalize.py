from typing import List

from dancing_datacollection.parsing_topturnier import TopTurnierParser
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.score import FinalRoundScore
from dancing_datacollection.html_generate import (
    generate_deck_html,
    generate_tabges_html,
    generate_erg_html,
    generate_ergwert_html,
)
from bs4 import BeautifulSoup
def parse_ergwert_header(html: str) -> tuple[list[str], dict[str, list[str]]]:
    """Extract (dance_names_english, judge_codes_per_dance) from an ergwert.htm HTML string."""
    soup = BeautifulSoup(html, "html.parser")
    table = soup.find("table", class_="tab1")
    dance_names_english: list[str] = []
    judge_codes_per_dance: dict[str, list[str]] = {}
    if not table:
        return dance_names_english, judge_codes_per_dance
    rows = table.find_all("tr")
    if len(rows) < 2:
        return dance_names_english, judge_codes_per_dance
    header0_cells = rows[0].find_all(["td", "th"])
    names = []
    for cell in header0_cells[4:]:
        text = cell.get_text(" ", strip=True)
        if text:
            names.append(text)
    from dancing_datacollection.data_defs.score import GERMAN_TO_ENGLISH_DANCE_NAME
    for n in names:
        eng = GERMAN_TO_ENGLISH_DANCE_NAME.get(n, n)
        if eng not in dance_names_english:
            dance_names_english.append(eng)
        if len(dance_names_english) >= 3:
            break
    header1_cells = rows[1].find_all(["td", "th"])
    codes_groups: list[list[str]] = []
    current: list[str] = []
    for cell in header1_cells:
        text = cell.get_text(strip=True)
        if text == "Su":
            codes_groups.append(current)
            current = []
        elif text:
            current.append(text)
    if current:
        codes_groups.append(current)
    for idx, name in enumerate(dance_names_english):
        group = codes_groups[idx] if idx < len(codes_groups) else []
        judge_codes_per_dance[name] = group
    return dance_names_english, judge_codes_per_dance



def canonical_deck_html(html: str) -> str:
    parser = TopTurnierParser()
    judges: List[Judge] = parser.extract_judges(html, filename="deck.htm")
    return generate_deck_html(judges)


def canonical_tabges_html(html: str) -> str:
    parser = TopTurnierParser()
    # participants can be gathered from tabges via TopTurnierParser
    participants, _ = parser.extract_participants(html, filename="tabges.htm")
    judges: List[Judge] = parser.extract_judges(html, filename="tabges.htm")
    return generate_tabges_html(participants, judges)


def canonical_erg_html(html: str) -> str:
    parser = TopTurnierParser()
    participants, _ = parser.extract_participants(html, filename="erg.htm")
    return generate_erg_html(participants)


def canonical_ergwert_html(html: str) -> str:
    parser = TopTurnierParser()
    participants, _ = parser.extract_participants(html, filename="ergwert.htm")
    judges: List[Judge] = parser.extract_judges(html, filename="ergwert.htm")
    scores = parser.extract_scores(html, filename="ergwert.htm")
    final_scores: List[FinalRoundScore] = [
        s for s in scores if isinstance(s, FinalRoundScore)
    ]
    dance_names_english, judge_codes_per_dance = parse_ergwert_header(html)
    return generate_ergwert_html(
        participants,
        judges,
        final_scores,
        dance_names_english=dance_names_english,
        judge_codes_per_dance=judge_codes_per_dance,
    )


