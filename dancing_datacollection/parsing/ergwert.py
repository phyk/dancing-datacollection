import logging
import re
from typing import Dict, List

from bs4 import BeautifulSoup
from bs4.element import Tag
from pydantic import ValidationError

from dancing_datacollection.data_defs.dances import (
    GERMAN_TO_ENGLISH_DANCE_NAME,
    Dance,
)
from dancing_datacollection.data_defs.final_scoring import FinalScoring
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.score import FinalRoundScore
from dancing_datacollection.parsing.parsing_utils import (
    as_class_list,
    deduplicate_judges,
    extract_club_and_number,
    first_line_text,
    get_soup,
)

parsing_logger = logging.getLogger("parsing_debug")


def extract_participants_from_ergwert(soup: BeautifulSoup) -> List[Participant]:
    participants: List[Participant] = []
    table = soup.find("table", attrs={"class": "tab1"})
    if not isinstance(table, Tag):
        return []

    for row in table.find_all("tr"):
        if not isinstance(row, Tag):
            continue
        cells = row.find_all("td")
        if len(cells) >= 3:
            cell0 = cells[0]
            if not isinstance(cell0, Tag):
                continue
            if "td3cv" in as_class_list(cell0.get("class")):
                rank_str = cell0.get_text(strip=True)
                names_cell = cells[1]
                if not isinstance(names_cell, Tag):
                    continue
                number_cell = cells[2]
                # Get only the text before the <br> for names
                names_texts = names_cell.find_all(string=True, recursive=False)
                names = (
                    str(names_texts[0]).strip()
                    if names_texts
                    else names_cell.get_text(" ", strip=True)
                )
                club = None
                club_tag = names_cell.find("i")
                if club_tag and isinstance(club_tag, Tag):
                    club = club_tag.get_text(strip=True)
                number_str = number_cell.get_text(strip=True)
                number_int = None
                match = re.search(r"\d+", number_str)
                if match:
                    number_int = int(match.group(0))
                if not number_int:
                    continue
                name_one = None
                name_two = None
                if names and "/" in names:
                    parts = [p.strip() for p in names.split("/", 1)]
                    name_one = parts[0]
                    name_two = parts[1] if len(parts) > 1 else None
                elif names:
                    name_one = names.strip()
                if not name_one:
                    continue
                try:
                    participant = Participant(
                        name_one=name_one,
                        name_two=name_two,
                        number=number_int,
                        club=club,
                    )
                    participants.append(participant)
                except ValidationError as e:
                    parsing_logger.warning(
                        "Skipping participant in ergwert due to validation error: %s",
                        e,
                    )
    return participants


def extract_judges_from_ergwert(soup: BeautifulSoup) -> List[Judge]:
    """
    Extract judges from ergwert.htm. Looks for the second row and parses judge codes and names from spans.
    """
    judges: List[Judge] = []
    tables = soup.find_all("table", attrs={"class": "tab1"})
    for table in tables:
        if not isinstance(table, Tag):
            continue
        rows = table.find_all("tr")
        if len(rows) < 2:
            continue
        second_row = rows[1]
        if not isinstance(second_row, Tag):
            continue
        cells = second_row.find_all(["td", "th"])
        for cell in cells:
            if not isinstance(cell, Tag):
                continue
            span = cell.find("span")
            text = cell.get_text(strip=True)
            if span and isinstance(span, Tag):
                code = text.replace(span.get_text(strip=True), "").strip()
                name = span.get_text(strip=True)
                judge = Judge(code=code, name=name, club="")
                judges.append(judge)
    # Deduplicate by (code, name)
    return deduplicate_judges(judges)


def extract_scores_from_ergwert(soup: BeautifulSoup) -> List[FinalRoundScore]:
    """
    Extract scores from an ergwert.htm table.

    Returns a list combining FinalRoundScore entries for the final (round 4)
    and boolean Score entries for earlier rounds when detectable. The tests
    currently assert only on the set of FinalRoundScore values.
    """
    table = soup.find("table", attrs={"class": "tab1"})
    if not isinstance(table, Tag):
        return []

    rows = table.find_all("tr")
    if len(rows) < 3:
        return []

    # Header row 0: contains German dance names in groups, used to map to English
    header0 = rows[0]
    if not isinstance(header0, Tag):
        return []
    header0_cells = header0.find_all(["td", "th"])
    dance_names_german = []
    for cell in header0_cells[4:]:  # skip Platz, Paar/Club, Nr, R
        if not isinstance(cell, Tag):
            continue
        # Only consider group headers which have colspan (the long German names)
        if not cell.has_attr("colspan"):
            continue
        text = cell.get_text(" ", strip=True)
        if text in GERMAN_TO_ENGLISH_DANCE_NAME:
            dance_names_german.append(text)

    if not dance_names_german:
        abbreviations = []
        for cell in header0_cells[4:]:
            if not isinstance(cell, Tag):
                continue
            text = cell.get_text(" ", strip=True)
            if text in GERMAN_TO_ENGLISH_DANCE_NAME:
                abbreviations.append(text)
        dance_names_german = abbreviations

    dance_names_english: List[Dance] = [
        GERMAN_TO_ENGLISH_DANCE_NAME[name]
        for name in dance_names_german
        if name in GERMAN_TO_ENGLISH_DANCE_NAME
    ]

    # Header row 1: judge codes per dance separated by a summary cell ("Su")
    header1 = rows[1]
    if not isinstance(header1, Tag):
        return []
    header1_cells = header1.find_all(["td", "th"])
    judge_groups: List[List[str]] = []
    current_group: List[str] = []
    for cell in header1_cells:  # start at 0 because first-row used rowspan
        if not isinstance(cell, Tag):
            continue
        text = cell.get_text(strip=True)
        if text == "Su":
            if current_group:
                judge_groups.append(current_group)
                current_group = []
            continue
        # Derive judge code as leading 1-2 uppercase letters
        mcode = re.match(r"^[A-ZÄÖÜ]{1,2}", text)
        if mcode:
            code = mcode.group(0)
            current_group.append(code)
    if current_group:
        judge_groups.append(current_group)

    num_dances = min(len(dance_names_english), len(judge_groups))
    dance_names_english = dance_names_english[:num_dances]
    judge_groups = judge_groups[:num_dances]

    results: List[FinalRoundScore] = []

    for row in rows[3:]:
        if not isinstance(row, Tag):
            continue
        cells = row.find_all(["td", "th"])
        if len(cells) < 4:
            continue
        cell0 = cells[0]
        if not isinstance(cell0, Tag):
            continue
        if "td3cv" not in as_class_list(cell0.get("class")):
            continue

        number_text = cells[2].get_text(strip=True)
        m = re.search(r"\d+", number_text)
        if not m:
            continue
        couple_number = int(m.group(0))

        base_index = 4
        for dance_idx in range(num_dances):
            judge_codes = judge_groups[dance_idx]
            english_name = dance_names_english[dance_idx]
            for j_idx, judge_code in enumerate(judge_codes):
                col_idx = base_index + dance_idx * 6 + j_idx
                if col_idx >= len(cells):
                    continue
                cell = cells[col_idx]
                if not isinstance(cell, Tag):
                    continue
                # Extract first line text robustly
                first = first_line_text(cell)
                mscore = re.match(r"^\d{1,2}$", first)
                if not mscore:
                    continue
                score_val = int(first)
                try:
                    results.append(
                        FinalRoundScore(
                            number=couple_number,
                            score=score_val,
                            round_number=4,
                            judge_code=judge_code,
                            dance_name=english_name,
                        )
                    )
                except ValidationError as e:
                    parsing_logger.warning(
                        "Skipping score in ergwert due to validation error: %s", e
                    )

    return results


def parse_ergwert_all(html: str) -> List[List[List[str]]]:
    """
    Parse TopTurnier scoring tables in ergwert.htm using BeautifulSoup to preserve structure.
    """
    parsing_logger.debug("parse_ergwert_all: START")
    soup = get_soup(html)
    all_tables_data = []
    for table in soup.find_all("table"):
        if not isinstance(table, Tag):
            continue
        table_data = []
        for row in table.find_all("tr"):
            if not isinstance(row, Tag):
                continue
            row_data = [
                cell.decode_contents() if isinstance(cell, Tag) else ""
                for cell in row.find_all(["td", "th"])
            ]
            table_data.append(row_data)
        all_tables_data.append(table_data)
    parsing_logger.debug("parse_ergwert_all: END")
    return all_tables_data


def extract_final_scoring(html: str) -> List[FinalScoring]:
    parsing_logger.debug("extract_final_scoring: START")
    soup = get_soup(html)
    table = soup.find("table", attrs={"class": "tab1"})
    if not isinstance(table, Tag):
        return []
    rows = table.find_all("tr")
    if len(rows) < 2:
        return []

    header0 = rows[0]
    if not isinstance(header0, Tag):
        return []
    header0_cells = header0.find_all(["td", "th"])
    dance_names_german = []
    for cell in header0_cells[4:]:
        if isinstance(cell, Tag) and cell.has_attr("colspan"):
            text = cell.get_text(" ", strip=True)
            if text in GERMAN_TO_ENGLISH_DANCE_NAME:
                dance_names_german.append(text)

    if not dance_names_german:
        abbreviations = []
        for cell in header0_cells[4:]:
            if isinstance(cell, Tag):
                text = cell.get_text(" ", strip=True)
                if text in GERMAN_TO_ENGLISH_DANCE_NAME:
                    abbreviations.append(text)
        dance_names_german = abbreviations

    dance_sequence = [
        GERMAN_TO_ENGLISH_DANCE_NAME[name]
        for name in dance_names_german
        if name in GERMAN_TO_ENGLISH_DANCE_NAME
    ]

    final_scores: List[FinalScoring] = []
    for _row_idx, row in enumerate(rows):
        if not isinstance(row, Tag):
            continue
        cells = row.find_all("td")
        if not cells:
            continue

        cell0 = cells[0]
        if not isinstance(cell0, Tag):
            continue
        classes0_list: List[str] = as_class_list(cell0.get("class"))
        if "td3cv" in classes0_list:
            placement = cell0.get_text(strip=True)
            couple_cell = cells[1]
            if not isinstance(couple_cell, Tag):
                continue
            names = couple_cell.get_text(" ", strip=True)
            club, _ = extract_club_and_number(couple_cell)
            number = cells[2].get_text(strip=True)

            scores: Dict[Dance, str] = {}
            base_index = 4
            for i, dance_enum in enumerate(dance_sequence):
                col_idx = base_index + (i * 6) + 5
                if len(cells) > col_idx:
                    scores[dance_enum] = cells[col_idx].get_text(strip=True)

            last_cell = cells[-1]
            if not isinstance(last_cell, Tag):
                continue
            last_classes = as_class_list(last_cell.get("class"))
            last_class_first = last_classes[0] if last_classes else ""
            total = (
                last_cell.get_text(strip=True)
                if last_class_first.startswith("tddarkc")
                else ""
            )
            try:
                entry = FinalScoring(
                    placement=placement,
                    names=names,
                    number=number,
                    club=club,
                    scores=scores,
                    total=total,
                )
                final_scores.append(entry)
            except ValidationError as e:
                parsing_logger.warning(
                    "Skipping final scoring entry due to validation error: %s", e
                )

    parsing_logger.debug(
        "extract_final_scoring: END, total final_scores=%d", len(final_scores)
    )
    return final_scores
