import logging
import re
from typing import List

from bs4 import BeautifulSoup
from bs4.element import Tag
from pydantic import ValidationError

from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.parsing.parsing_utils import deduplicate_judges, get_soup

parsing_logger = logging.getLogger("parsing_debug")


def extract_participants_from_tabges(soup: BeautifulSoup) -> List[Participant]:
    participants: List[Participant] = []
    seen = set()
    # Collect all unique participants from all td2gc cells in the file
    for cell in soup.find_all("td", attrs={"class": "td2gc"}):
        if not isinstance(cell, Tag):
            continue
        number_str = cell.get_text(strip=True)
        match = re.search(r"\d+", number_str)
        if not match:
            continue
        number_int = int(match.group(0))
        tooltip = cell.find("span", {"class": "tooltip2gc"})
        if tooltip and isinstance(tooltip, Tag):
            names = tooltip.get_text(strip=True)
            if "/" in names:
                name_one, name_two = (n.strip() for n in names.split("/", 1))
            else:
                name_one, name_two = names.strip(), None
            key = (number_int, name_one, name_two)
            if key not in seen:
                try:
                    if name_one:
                        participant = Participant(
                            name_one=name_one,
                            name_two=name_two,
                            number=number_int,
                            ranks=None,
                            club=None,
                        )
                        participants.append(participant)
                        seen.add(key)
                except ValidationError as e:
                    parsing_logger.warning(
                        "Skipping participant in tabges due to validation error: %s", e
                    )
    return participants


def extract_judges_from_tabges(soup: BeautifulSoup) -> List[Judge]:
    """
    Extract judges from tabges.htm. Looks for the Wertungsrichter row and parses judge codes and names.
    """
    judges: List[Judge] = []
    table = soup.find("table", attrs={"class": "tab1"})
    if not isinstance(table, Tag):
        return judges
    found_judges = False
    for _row_idx, row in enumerate(table.find_all("tr")):
        if not isinstance(row, Tag):
            continue
        cells = row.find_all("td")
        if not cells:
            continue
        if not found_judges:
            if cells[0].get_text(strip=True).replace(":", "") == "Wertungsrichter":
                found_judges = True
            continue
        cell0 = cells[0]
        if not isinstance(cell0, Tag):
            continue
        cell_class = cell0.get("class")
        if found_judges and cell_class and "td3" in cell_class:
            cell_soup = BeautifulSoup(str(cells[0]), "html.parser")
            judge_lines = [t for t in cell_soup.stripped_strings if t]
            for line in judge_lines:
                m = re.match(r"([A-Z]{2})\)\s*(.+)", line)
                if m:
                    code = m.group(1)
                    name = m.group(2)
                    club = ""
                    judge = Judge(code=code, name=name, club=club)
                    judges.append(judge)
    # Deduplicate by (code, name)
    return deduplicate_judges(judges)


def parse_tabges_all(html: str) -> List[List[List[str]]]:
    """
    Parse TopTurnier scoring tables in tabges.htm using BeautifulSoup to preserve structure.
    Returns a list of tables, where each table is a list of rows, and each row is a list of cell HTML content.
    """
    parsing_logger.debug("parse_tabges_all: START")
    soup = get_soup(html)
    all_tables_data = []
    for table in soup.find_all("table", attrs={"class": "tab1"}):
        if not isinstance(table, Tag):
            continue
        table_data = []
        for row in table.find_all("tr"):
            if not isinstance(row, Tag):
                continue
            row_data = [
                str(cell.decode_contents())
                for cell in row.find_all("td")
                if isinstance(cell, Tag)
            ]
            table_data.append(row_data)
        all_tables_data.append(table_data)
    parsing_logger.debug("parse_tabges_all: END")
    return all_tables_data