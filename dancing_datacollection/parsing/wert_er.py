import logging
import re
from typing import List, Set

from bs4 import BeautifulSoup
from bs4.element import Tag
from pydantic import ValidationError

from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.parsing.parsing_utils import deduplicate_judges

parsing_logger = logging.getLogger("parsing_debug")


def extract_participants_from_wert_er(soup: BeautifulSoup) -> List[Participant]:
    participants: List[Participant] = []
    table = soup.find("table", attrs={"class": "tab1"})
    seen_numbers: Set[int] = set()
    if isinstance(table, Tag):
        for cell in table.find_all("td", attrs={"class": "td3r"}):
            if not isinstance(cell, Tag):
                continue
            number_str = cell.get_text(strip=True)
            number_int = None
            match = re.search(r"\d+", number_str)
            if match:
                number_int = int(match.group(0))
            if not number_int or number_int in seen_numbers:
                continue
            seen_numbers.add(number_int)
            name_one = None
            name_two = None
            tooltip = cell.find("span", {"class": "tooltip3r"})
            if tooltip and isinstance(tooltip, Tag):
                names = tooltip.get_text(strip=True)
                if "/" in names:
                    name_one, name_two = (p.strip() for p in names.split("/", 1))
                else:
                    name_one = names.strip()
            if not name_one:
                continue
            try:
                participant = Participant(
                    name_one=name_one,
                    name_two=name_two,
                    number=number_int,
                    ranks=None,
                    club=None,
                )
                participants.append(participant)
            except ValidationError as e:
                parsing_logger.warning(
                    "Skipping participant in wert_er due to validation error: %s", e
                )
    return participants


def extract_judges_from_wert_er(soup: BeautifulSoup) -> List[Judge]:
    """
    Extract judges from wert_er.htm. Looks for the second row and parses judge codes and names from spans.
    """
    judges: List[Judge] = []
    table = soup.find("table", attrs={"class": "tab1"})
    if not isinstance(table, Tag):
        return judges
    rows = table.find_all("tr")
    if len(rows) < 2:
        return judges
    second_row = rows[1]
    if not isinstance(second_row, Tag):
        return judges
    judge_cells = second_row.find_all(["td", "th"])
    for cell in judge_cells:
        if not isinstance(cell, Tag):
            continue
        span = cell.find("span")
        if span and isinstance(span, Tag):
            code = (
                cell.contents[0].strip()
                if cell.contents and isinstance(cell.contents[0], str)
                else ""
            )
            name = span.get_text(strip=True)
            if len(code) == 2 and code.isupper():
                judge = Judge(code=code, name=name, club="")
                judges.append(judge)
    return deduplicate_judges(judges)