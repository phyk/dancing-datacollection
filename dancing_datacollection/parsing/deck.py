import logging
from typing import Any, Dict, List, cast

from bs4 import BeautifulSoup
from bs4.element import Tag
from pydantic import ValidationError

from dancing_datacollection.data_defs.committee import CommitteeMember
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.parsing.parsing_utils import (
    deduplicate_judges,
    extract_name_and_club_from_spans,
    get_soup,
)

parsing_logger = logging.getLogger("parsing_debug")


def merge_judges_prefer_club(*lists: List[Judge]) -> List[Judge]:
    """Merge judge lists and deduplicate, preferring entries that have a non-empty club."""
    merged: List[Judge] = []
    for lst in lists:
        merged.extend(lst)
    return deduplicate_judges(merged)


def extract_judges_from_deck(soup: BeautifulSoup) -> List[Judge]:
    """
    Extract judges from deck.htm using the annotated structure:
    Table 1, rows 6-10: cell 0 is judge code (remove colon), cell 1 is 'Last, First Club'.
    Parse name as last name, first name, and the rest as club.
    Return a list of Judge dataclasses with code, name, club.
    """
    logger = logging.getLogger("parsing_debug")
    tables = soup.find_all("table")
    judges = []
    if len(tables) < 2:
        logger.warning("Expected at least 2 tables in deck.htm")
        return judges
    table = tables[1]
    if not isinstance(table, Tag):
        return judges
    rows = table.find_all("tr")
    for row in rows:
        if not isinstance(row, Tag):
            continue
        cells = row.find_all(["td", "th"])
        if len(cells) < 2:
            continue
        # Only process rows where the first cell has class 'td2r' (judge rows)
        cell0 = cells[0]
        if isinstance(cell0, Tag) and "td2r" in (cell0.get("class") or []):
            code = cell0.get_text(strip=True).replace(":", "")
            # Use spans to extract name and club
            cell1 = cells[1]
            if not isinstance(cell1, Tag):
                continue
            spans = cell1.find_all("span")
            if len(spans) >= 2:
                name_raw = (
                    spans[0]
                    .get_text(strip=True)
                    .replace("\xa0", "")
                    .replace("\u00a0", "")
                    .strip()
                )
                club = (
                    spans[1]
                    .get_text(strip=True)
                    .replace("\xa0", "")
                    .replace("\u00a0", "")
                    .strip()
                )
                if ", " in name_raw:
                    last, first = [x.strip() for x in name_raw.split(",", 1)]
                    name = f"{first} {last}"
                else:
                    name = name_raw
            else:
                name = cell1.get_text(strip=True)
                club = ""
            logger.debug("  Judge: code=%s, name=%s, club=%s", code, name, club)
            try:
                judge = Judge(code=code, name=name, club=club)
                judges.append(judge)
            except ValidationError as e:
                logger.warning(
                    "Invalid judge skipped: code=%s, name=%s, club=%s, error=%s",
                    code,
                    name,
                    club,
                    e,
                )
    return deduplicate_judges(judges)


def parse_deck_all(html: str) -> List[Dict[str, Any]]:
    """
    Parse all available information from deck.htm, logging any unrecognized or ambiguous content.
    Returns a dictionary with all found data, including unknown/extra fields.
    """
    parsing_logger.debug("parse_deck_all: START")
    soup: Any = get_soup(html)
    tables: List[Any] = cast(Any, soup).find_all("table")
    all_data = []
    for table_idx, table in enumerate(tables):
        table_data: Dict[str, Any] = {"table_idx": table_idx, "rows": []}
        rows: List[Any] = cast(Any, table).find_all("tr")
        for row_idx, row in enumerate(rows):
            cells: List[Any] = cast(Any, row).find_all(["td", "th"])
            cell_data = []
            for cell_idx, cell in enumerate(cells):
                text = cell.get_text(" ", strip=True)
                raw_classes = cell.get("class")
                if isinstance(raw_classes, list):
                    cell_class: List[str] = raw_classes
                elif isinstance(raw_classes, str):
                    cell_class = [raw_classes]
                else:
                    cell_class = []
                cell_html = str(cell)
                cell_info = {
                    "cell_idx": cell_idx,
                    "text": text,
                    "class": cell_class,
                    "html": cell_html,
                }
                known_classes = {
                    "td2",
                    "td2c",
                    "td2gc",
                    "td1",
                    "td3",
                    "td3c",
                    "td3cv",
                    "td5c",
                    "td5cv",
                }
                if not set(cell_class).intersection(known_classes):
                    parsing_logger.warning(
                        "Unrecognized cell class in deck.htm: Table %d, Row %d, Cell %d, Class: %s, Text: %s",
                        table_idx,
                        row_idx,
                        cell_idx,
                        cell_class,
                        text,
                    )
                if text == "" or text == "\xa0":
                    parsing_logger.info(
                        "Empty or ambiguous cell in deck.htm: Table %d, Row %d, Cell %d, HTML: %s",
                        table_idx,
                        row_idx,
                        cell_idx,
                        cell_html,
                    )
                cell_data.append(cell_info)
            table_data["rows"].append({"row_idx": row_idx, "cells": cell_data})
        all_data.append(table_data)
    parsing_logger.debug("parse_deck_all: END")
    return all_data


def extract_committee_from_deck(soup: BeautifulSoup) -> List[CommitteeMember]:
    logger = logging.getLogger("parsing_debug")
    table = soup.find("table", attrs={"class": "tab1"})
    logger.debug("Found table: %s", bool(table))
    if not isinstance(table, Tag):
        return []
    roles = [
        ("Veranstalter:", "organizer"),
        ("Ausrichter:", "host"),
        ("Turnierleiter:", "chairperson"),
        ("Beisitzer:", "committee_member"),
        ("Protokoll:", "protocol"),
    ]
    committee = []
    for row_idx, row in enumerate(table.find_all("tr")):
        if not isinstance(row, Tag):
            continue
        cells = row.find_all("td")
        logger.debug(
            "Row %d: %s",
            row_idx,
            [c.get_text(" ", strip=True) for c in cells],
        )
        if len(cells) < 2:
            continue
        label = cells[0].get_text(strip=True)
        for role_label, role_key in roles:
            if label == role_label:
                value_cell = cells[1]
                if not isinstance(value_cell, Tag):
                    continue
                name, club = extract_name_and_club_from_spans(value_cell)
                logger.debug(
                    "  Committee: role=%s, name=%s, club=%s, raw_value=%s",
                    role_key,
                    name,
                    club,
                    value_cell.get_text(" ", strip=True),
                )
                committee.append(CommitteeMember(role=role_key, name=name, club=club))
    logger.debug("extract_committee_from_deck: END, total committee=%d", len(committee))
    return committee
