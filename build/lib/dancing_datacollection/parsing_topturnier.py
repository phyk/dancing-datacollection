from .parsing_base import CompetitionParser
from bs4 import BeautifulSoup
import re
from .parsing_utils import (
    get_soup,
    extract_club_and_number,
    as_class_list,
)
from .parsing.erg import extract_participants_from_erg, extract_judges_from_erg
from .parsing.ergwert import (
    extract_participants_from_ergwert,
    extract_judges_from_ergwert,
    extract_scores_from_ergwert,
)
from .parsing.tabges import extract_participants_from_tabges, extract_judges_from_tabges
from .parsing.wert_er import (
    extract_participants_from_wert_er,
    extract_judges_from_wert_er,
)
import logging
from dancing_datacollection.data_defs.judge import Judge
from typing import Any, List, Optional, cast
from .parsing.deck import extract_judges_from_deck
from .parsing.committee import extract_committee_from_deck

parsing_logger = logging.getLogger("parsing_debug")


class TopTurnierParser(CompetitionParser):
    def extract_participants(self, html, filename=None):
        """
        Extract participants from the given HTML, using the filename to select the correct extraction method.
        The filename must be one of: 'erg.htm', 'ergwert.htm', 'tabges.htm', 'wert_er.htm'.
        """
        if filename is None:
            raise ValueError("filename argument is required for extract_participants")
        soup = get_soup(html)
        # Use filename to select extraction method
        if filename.endswith("erg.htm"):
            participants = extract_participants_from_erg(soup)
        elif filename.endswith("ergwert.htm"):
            participants = extract_participants_from_ergwert(soup)
        elif filename.endswith("tabges.htm"):
            participants = extract_participants_from_tabges(soup)
        elif filename.endswith("wert_er.htm"):
            participants = extract_participants_from_wert_er(soup)
        else:
            raise ValueError(f"Unknown filename for participant extraction: {filename}")
        # Event name extraction (for compatibility)
        title_tag = soup.find("title")
        event_name = title_tag.get_text(strip=True) if title_tag else "unknown_event"
        event_name = re.sub(r"[^\w\d-]+", "_", event_name)[:64]
        return participants, event_name

    def extract_judges(self, html, filename=None) -> List[Judge]:
        """Dispatch judge extraction based on filename suffix."""
        if filename is None:
            raise ValueError("filename argument is required for extract_judges")
        soup = get_soup(html)
        handlers = {
            "deck.htm": extract_judges_from_deck,
            "tabges.htm": extract_judges_from_tabges,
            "ergwert.htm": extract_judges_from_ergwert,
            "wert_er.htm": extract_judges_from_wert_er,
            "erg.htm": extract_judges_from_erg,
        }
        for suffix, func in handlers.items():
            if filename.endswith(suffix):
                return func(soup)
        raise ValueError(f"Unknown filename for judge extraction: {filename}")

    def extract_committee(self, html):
        soup = get_soup(html)
        return extract_committee_from_deck(soup)

    def extract_scores(self, html, filename=None):
        if filename is None:
            raise ValueError("filename argument is required for extract_scores")
        soup = get_soup(html)
        if filename.endswith("ergwert.htm"):
            return extract_scores_from_ergwert(soup)
        else:
            return []

    def extract_final_scoring(self, html):
        parsing_logger.debug("extract_final_scoring: START")
        soup: Any = get_soup(html)
        table: Any = soup.find("table", class_="tab1")
        parsing_logger.debug(f"Found table: {bool(table)}")
        if not table:
            return []
        rows: List[Any] = table.find_all("tr")
        final_scores = []
        for row_idx, row in enumerate(rows):
            cells: List[Any] = cast(Any, row).find_all("td")
            parsing_logger.debug(
                f"Row {row_idx}: {[c.get_text(' ', strip=True) for c in cells]}"
            )
            if not cells:
                continue
            classes0_list: List[str] = as_class_list(cells[0].get("class"))
            if "td3cv" in classes0_list:
                placement = cells[0].get_text(strip=True)
                couple_cell = cells[1]
                names = couple_cell.get_text(" ", strip=True)
                club, _ = extract_club_and_number(couple_cell)
                number = cells[2].get_text(strip=True)
                lw_score = cells[9].get_text(strip=True) if len(cells) > 9 else ""
                tg_score = cells[15].get_text(strip=True) if len(cells) > 15 else ""
                qs_score = cells[21].get_text(strip=True) if len(cells) > 21 else ""
                last_classes = as_class_list(cells[-1].get("class"))
                last_class_first = last_classes[0] if last_classes else ""
                total = cells[-1].get_text(strip=True) if last_class_first.startswith("tddarkc") else ""
                entry = {
                    "placement": placement,
                    "names": names,
                    "number": number,
                    "club": club,
                    "score_LW": lw_score,
                    "score_TG": tg_score,
                    "score_QS": qs_score,
                    "total": total,
                }
                parsing_logger.debug(f"  Final scoring entry: {entry}")
                final_scores.append(entry)
        parsing_logger.debug(
            f"extract_final_scoring: END, total final_scores={len(final_scores)}"
        )
        return final_scores

    def parse_tabges_all(self, html):
        """
        Parse TopTurnier scoring tables in tabges.htm via pandas.read_html.
        """
        parsing_logger.debug("parse_tabges_all: START")
        try:
            import pandas as pd
            from io import StringIO
            return pd.read_html(StringIO(html), attrs={"class": "tab1"}, header=None)
        except Exception as e:
            parsing_logger.error(f"parse_tabges_all failed via pandas.read_html: {e}")
            return []

    def parse_erg_all(self, html):
        """Lightweight dump of erg.htm tables for inspection (dev aid)."""
        parsing_logger.debug("parse_erg_all: START")
        soup: Any = get_soup(html)
        result = []
        for table_idx, table in enumerate(cast(Any, soup).find_all("table")):
            rows_dump = []
            for row_idx, row in enumerate(cast(Any, table).find_all("tr")):
                cells: List[Any] = cast(Any, row).find_all(["td", "th"])
                rows_dump.append({
                    "row_idx": row_idx,
                    "cells": [c.get_text(" ", strip=True) for c in cells],
                })
            result.append({"table_idx": table_idx, "rows": rows_dump})
        parsing_logger.debug("parse_erg_all: END")
        return result

    def parse_deck_all(self, html):
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

    def extract_finalists_from_erg(self, html):
        """Developer helper to explore finalist rows in erg.htm."""
        parsing_logger.debug("extract_finalists_from_erg: START")
        soup: Any = BeautifulSoup(html, "html.parser")
        couples = []
        for table in cast(Any, soup).find_all("table"):
            rows: List[Any] = cast(Any, table).find_all("tr")
            for row in rows:
                cells: List[Any] = cast(Any, row).find_all(["td", "th"])
                if len(cells) < 3:
                    continue
                classes = cells[0].get("class")
                if "td3cv" in (classes if isinstance(classes, str) else " ".join(classes or [])):
                    name_text = cells[1].get_text(" ", strip=True)
                    m = re.search(r"\((\d+)\)", name_text)
                    number = m.group(1) if m else None
                    couples.append({"number": number, "name": name_text})
        parsing_logger.debug("extract_finalists_from_erg: END")
        return couples






def compare_all_html_data(directory):
    """
    For a given directory, parse all relevant HTML files using the corresponding parse_all methods (or extract methods if parse_all is not available),
    aggregate the data, and compare for inconsistencies. Print/log any differences found.
    Now also returns the set of Judge dataclasses present in all files, with the most information.
    """
    import pathlib
    import logging

    logger = logging.getLogger("parsing_debug")
    logger.info(f"Comparing all HTML data in {directory}")

    parser = TopTurnierParser()
    dir_path = pathlib.Path(directory)

    # Read all HTML files for direct judge extraction
    html_files = {}
    for fname in ["deck.htm", "tabges.htm", "ergwert.htm", "wert_er.htm", "erg.htm"]:
        fkey = fname.replace(".htm", "")
        fpath = dir_path / fname
        if fpath.exists():
            html_files[fkey] = fpath.read_text(encoding="utf-8")

    # Extract judges from each file
    judge_sets = {}
    judge_lists = {}
    for source_name in ["deck", "tabges", "ergwert", "wert_er", "erg"]:
        if source_name in html_files:
            filename = f"{source_name}.htm"
            judges = parser.extract_judges(html_files[source_name], filename=filename)
            judge_sets[source_name] = set(judges)
            judge_lists[source_name] = judges

    # Define judge_key before use
    def judge_key(j):
        return (j.code, j.name)

    # Find sets of (code, name) for each file
    judge_keys_by_src = {
        src: set(judge_key(j) for j in judge_sets[src]) for src in judge_sets
    }
    all_key_sets = list(judge_keys_by_src.values())
    if not all_key_sets:
        logger.info("No judge info found in any file.")
        return set()
    # Check if all sets are identical
    all_identical = all(s == all_key_sets[0] for s in all_key_sets[1:])
    if not all_identical:
        logger.error("Judge sets are not identical across all files!")
        for src, keys in judge_keys_by_src.items():
            logger.error(f"  {src}: {keys}")
        # Optionally, log missing judges per file
        all_keys_union = set.union(*all_key_sets)
        for src, keys in judge_keys_by_src.items():
            missing = all_keys_union - keys
            if missing:
                logger.error(f"  Judges missing in {src}: {missing}")
        return set()
    # If identical, proceed as before

    # For each common judge, pick the dataclass with the most info (prefer club if available)
    best_judges = {}
    any_src_key = next(iter(judge_keys_by_src))
    for code_name in judge_keys_by_src[any_src_key]:
        best = None
        for src_key in judge_keys_by_src:
            for j in judge_sets[src_key]:
                if judge_key(j) == code_name:
                    if not best or (j.club and not best.club):
                        best = j
        best_judges[code_name] = best

    # Log inconsistencies
    for code_name in judge_keys_by_src[any_src_key]:
        clubs = set()
        for src_key in judge_keys_by_src:
            for j in judge_sets[src_key]:
                if judge_key(j) == code_name:
                    clubs.add(j.club)
        # Only warn if all clubs are not None and there is more than one unique club
        non_null_clubs = set(c for c in clubs if c is not None)
        if len(non_null_clubs) > 1 and len(non_null_clubs) == len(clubs):
            logger.warning(f"Inconsistent club for judge {code_name}: {clubs}")

    logger.info("Final consistent judges:")
    for j in best_judges.values():
        logger.info(f"  {j}")
    return set(best_judges.values())
