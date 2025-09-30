from dancing_datacollection.data_defs.committee import CommitteeMember
from dancing_datacollection.data_defs.judge import Judge
from typing import List, Any, cast
import logging
from dancing_datacollection.parsing.parsing_utils import (
    deduplicate_judges,
    get_soup,
    extract_name_and_club_from_spans,
)


parsing_logger = logging.getLogger("parsing_debug")


def merge_judges_prefer_club(*lists: List[Judge]) -> List[Judge]:
    """Merge judge lists and deduplicate, preferring entries that have a non-empty club."""
    merged: List[Judge] = []
    for lst in lists:
        merged.extend(lst)
    return deduplicate_judges(merged)


def extract_judges_from_deck(soup) -> List[Judge]:
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
    rows = tables[1].find_all("tr")
    for row in rows:
        cells = row.find_all(["td", "th"])
        if len(cells) < 2:
            continue
        # Only process rows where the first cell has class 'td2r' (judge rows)
        if "td2r" in (cells[0].get("class") or []):
            code = cells[0].get_text(strip=True).replace(":", "")
            # Use spans to extract name and club
            spans = cells[1].find_all("span")
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
                if "," in name_raw:
                    last, first = [x.strip() for x in name_raw.split(",", 1)]
                    name = f"{first} {last}"
                else:
                    name = name_raw
            else:
                name = cells[1].get_text(strip=True)
                club = ""
            logger.debug(f"  Judge: code={code}, name={name}, club={club}")
            try:
                judge = Judge(code=code, name=name, club=club)
                judges.append(judge)
            except Exception as e:
                logger.warning(
                    f"Invalid judge skipped: code={code}, name={name}, club={club}, error={e}"
                )
    return deduplicate_judges(judges)


def parse_deck_all(html):
    """
    Parse all available information from deck.htm, logging any unrecognized or ambiguous content.
    Returns a dictionary with all found data, including unknown/extra fields.
    """
    parsing_logger.debug("parse_deck_all: START")
    soup: Any = get_soup(html)
    tables: List[Any] = cast(Any, soup).find_all("table")
    all_data = []
    for table_idx, table in enumerate(tables):
        table_data = {"table_idx": table_idx, "rows": []}
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
                        f"Unrecognized cell class in deck.htm: Table {table_idx}, Row {row_idx}, Cell {cell_idx}, Class: {cell_class}, Text: {text}"
                    )
                if text == "" or text == "\xa0":
                    parsing_logger.info(
                        f"Empty or ambiguous cell in deck.htm: Table {table_idx}, Row {row_idx}, Cell {cell_idx}, HTML: {cell_html}"
                    )
                cell_data.append(cell_info)
            table_data["rows"].append({"row_idx": row_idx, "cells": cell_data})
        all_data.append(table_data)
    parsing_logger.debug("parse_deck_all: END")
    return all_data


def extract_committee_from_deck(soup):
    logger = logging.getLogger("parsing_debug")
    table = soup.find("table", class_="tab1")
    logger.debug(f"Found table: {bool(table)}")
    if not table:
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
        cells = row.find_all("td")
        logger.debug(f"Row {row_idx}: {[c.get_text(' ', strip=True) for c in cells]}")
        if len(cells) < 2:
            continue
        label = cells[0].get_text(strip=True)
        for role_label, role_key in roles:
            if label == role_label:
                value_cell = cells[1]
                name, club = extract_name_and_club_from_spans(value_cell)
                logger.debug(
                    f"  Committee: role={role_key}, name={name}, club={club}, raw_value={value_cell.get_text(' ', strip=True)}"
                )
                committee.append(CommitteeMember(role=role_key, name=name, club=club))
    logger.debug(f"extract_committee_from_deck: END, total committee={len(committee)}")
    return committee
