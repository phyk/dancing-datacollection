from .parsing_base import CompetitionParser
from bs4 import BeautifulSoup
import re
from .parsing_utils import (
    get_soup,
    extract_club_and_number,
)
from .parsing.erg import extract_participants_from_erg, extract_judges_from_erg
from .parsing.ergwert import (
    extract_participants_from_ergwert,
    extract_judges_from_ergwert,
)
from .parsing.tabges import (
    extract_participants_from_tabges,
    extract_judges_from_tabges,
    extract_scores_from_tabges,
)
from .parsing.wert_er import (
    extract_participants_from_wert_er,
    extract_judges_from_wert_er,
)
import logging
from dancing_datacollection.data_defs.judge import Judge
from typing import List, Optional
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
        """
        Extract judges from the given HTML, using the filename to select the correct extraction method.
        The filename must be one of: 'deck.htm', 'tabges.htm', 'ergwert.htm', 'wert_er.htm', 'erg.htm'.
        """
        if filename is None:
            raise ValueError("filename argument is required for extract_judges")
        soup = get_soup(html)
        if filename.endswith("deck.htm"):
            return extract_judges_from_deck(soup)
        elif filename.endswith("tabges.htm"):
            return extract_judges_from_tabges(soup)
        elif filename.endswith("ergwert.htm"):
            return extract_judges_from_ergwert(soup)
        elif filename.endswith("wert_er.htm"):
            return extract_judges_from_wert_er(soup)
        elif filename.endswith("erg.htm"):
            return extract_judges_from_erg(soup)
        else:
            raise ValueError(f"Unknown filename for judge extraction: {filename}")

    def extract_committee(self, html):
        soup = get_soup(html)
        return extract_committee_from_deck(soup)

    def extract_scores(self, html):
        soup = get_soup(html)
        return extract_scores_from_tabges(soup)

    def extract_final_scoring(self, html):
        parsing_logger.debug("extract_final_scoring: START")
        soup = get_soup(html)
        table = soup.find("table", class_="tab1")
        parsing_logger.debug(f"Found table: {bool(table)}")
        if not table:
            return []
        rows = table.find_all("tr")
        final_scores = []
        for row_idx, row in enumerate(rows):
            cells = row.find_all("td")
            parsing_logger.debug(
                f'Row {row_idx}: {[c.get_text(" ", strip=True) for c in cells]}'
            )
            if not cells or not cells[0].get("class", []):
                continue
            if "td3cv" in cells[0].get("class", []):
                placement = cells[0].get_text(strip=True)
                couple_cell = cells[1]
                names = couple_cell.get_text(" ", strip=True)
                club, _ = extract_club_and_number(couple_cell)
                number = cells[2].get_text(strip=True)
                lw_score = cells[9].get_text(strip=True) if len(cells) > 9 else ""
                tg_score = cells[15].get_text(strip=True) if len(cells) > 15 else ""
                qs_score = cells[21].get_text(strip=True) if len(cells) > 21 else ""
                total = (
                    cells[-1].get_text(strip=True)
                    if cells[-1].get("class", [""])[0].startswith("tddarkc")
                    else ""
                )
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
        Parse all available information from tabges.htm, logging any unrecognized or ambiguous content.
        Returns a dictionary with all found data, including unknown/extra fields.
        """
        parsing_logger.debug("parse_tabges_all: START")
        soup = get_soup(html)
        tables = soup.find_all("table")
        all_data = []
        for table_idx, table in enumerate(tables):
            table_data = {"table_idx": table_idx, "rows": []}
            rows = table.find_all("tr")
            for row_idx, row in enumerate(rows):
                cells = row.find_all(["td", "th"])
                cell_data = []
                for cell_idx, cell in enumerate(cells):
                    text = cell.get_text(" ", strip=True)
                    cell_class = cell.get("class", [])
                    cell_html = str(cell)
                    cell_info = {
                        "cell_idx": cell_idx,
                        "text": text,
                        "class": cell_class,
                        "html": cell_html,
                    }
                    # Heuristic: if cell class or text is not recognized, log it for user review
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
                            f"Unrecognized cell class in tabges.htm: Table {table_idx}, Row {row_idx}, Cell {cell_idx}, Class: {cell_class}, Text: {text}"
                        )
                    if text == "" or text == "\xa0":
                        parsing_logger.info(
                            f"Empty or ambiguous cell in tabges.htm: Table {table_idx}, Row {row_idx}, Cell {cell_idx}, HTML: {cell_html}"
                        )
                    cell_data.append(cell_info)
                table_data["rows"].append({"row_idx": row_idx, "cells": cell_data})
            all_data.append(table_data)
        parsing_logger.debug("parse_tabges_all: END")
        return all_data

    def parse_erg_all(self, html):
        """
        Parse all available information from erg.htm, logging any unrecognized or ambiguous content.
        Returns a dictionary with all found data, including unknown/extra fields.
        """
        parsing_logger.debug("parse_erg_all: START")
        soup = get_soup(html)
        tables = soup.find_all("table")
        all_data = []
        for table_idx, table in enumerate(tables):
            table_data = {"table_idx": table_idx, "rows": []}
            rows = table.find_all("tr")
            for row_idx, row in enumerate(rows):
                cells = row.find_all(["td", "th"])
                cell_data = []
                for cell_idx, cell in enumerate(cells):
                    text = cell.get_text(" ", strip=True)
                    cell_class = cell.get("class", [])
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
                            f"Unrecognized cell class in erg.htm: Table {table_idx}, Row {row_idx}, Cell {cell_idx}, Class: {cell_class}, Text: {text}"
                        )
                    if text == "" or text == "\xa0":
                        parsing_logger.info(
                            f"Empty or ambiguous cell in erg.htm: Table {table_idx}, Row {row_idx}, Cell {cell_idx}, HTML: {cell_html}"
                        )
                    cell_data.append(cell_info)
                table_data["rows"].append({"row_idx": row_idx, "cells": cell_data})
            all_data.append(table_data)
        parsing_logger.debug("parse_erg_all: END")
        return all_data

    def parse_deck_all(self, html):
        """
        Parse all available information from deck.htm, logging any unrecognized or ambiguous content.
        Returns a dictionary with all found data, including unknown/extra fields.
        """
        parsing_logger.debug("parse_deck_all: START")
        soup = get_soup(html)
        tables = soup.find_all("table")
        all_data = []
        for table_idx, table in enumerate(tables):
            table_data = {"table_idx": table_idx, "rows": []}
            rows = table.find_all("tr")
            for row_idx, row in enumerate(rows):
                cells = row.find_all(["td", "th"])
                cell_data = []
                for cell_idx, cell in enumerate(cells):
                    text = cell.get_text(" ", strip=True)
                    cell_class = cell.get("class", [])
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
        """
        Parse erg.htm: detect header row for dances, then for each couple row extract ranking, name_one, name_two, number, club, and for each dance cell, the judge rankings and overall score if present. Handles missing data for non-finalists. Returns a list of dicts, one per couple, with all extracted info.
        """
        parsing_logger.debug("extract_finalists_from_erg: START")
        soup = BeautifulSoup(html, "html.parser")
        tables = soup.find_all("table")
        couples = []
        for table in tables:
            rows = table.find_all("tr")
            if not rows or len(rows) < 2:
                continue
            # Find header row with dances (look for cells with short text, e.g., 'LW', 'TG', etc.)
            header_idx = None
            dances = []
            for idx, row in enumerate(rows):
                cells = row.find_all(["td", "th"])
                cell_texts = [c.get_text(strip=True) for c in cells]
                if len(cells) >= 3 and all(2 <= len(t) <= 4 for t in cell_texts[2:]):
                    header_idx = idx
                    dances = cell_texts[2:]
                    break
            # If no dance header found, treat as non-finalist table (3 columns: ranking, names, club)
            if header_idx is None or not dances:
                # Try to find the first data row (skip headers like '2. Zwischenrunde', etc.)
                for row in rows:
                    cells = row.find_all(["td", "th"])
                    if len(cells) != 3:
                        continue
                    ranking = cells[0].get_text(strip=True)
                    name_text = cells[1].get_text(" ", strip=True)
                    club = cells[2].get_text(strip=True)
                    # Extract names and number
                    name_number_match = re.match(r"(.+?)\((\d+)\)", name_text)
                    if name_number_match:
                        names = name_number_match.group(1).strip()
                        number = name_number_match.group(2).strip()
                    else:
                        names = name_text.strip()
                        number = None
                    # Split names into name_one and name_two
                    if " / " in names:
                        name_one, name_two = [n.strip() for n in names.split(" / ", 1)]
                    else:
                        name_one, name_two = names, ""
                    couples.append(
                        {
                            "ranking": ranking,
                            "name_one": name_one,
                            "name_two": name_two,
                            "number": number,
                            "club": club,
                            "dances": {},
                        }
                    )
                continue
            # Parse couple rows after header (finalists)
            for row in rows[header_idx + 1 :]:
                cells = row.find_all(["td", "th"])
                if len(cells) < 3:
                    continue
                # First cell: overall ranking
                ranking = cells[0].get_text(strip=True)
                # Second cell: names and club
                name_cell = cells[1]
                name_text = name_cell.get_text(" ", strip=True)
                # Extract club from <i> tag
                club_tag = name_cell.find("i")
                club = club_tag.get_text(strip=True) if club_tag else ""
                # Remove club from name_text if present
                if club:
                    name_text = name_text.replace(club, "").strip()
                # Extract names and number
                name_number_match = re.match(r"(.+?)\((\d+)\)", name_text)
                if name_number_match:
                    names = name_number_match.group(1).strip()
                    number = name_number_match.group(2).strip()
                else:
                    names = name_text.strip()
                    number = None
                # Split names into name_one and name_two
                if " / " in names:
                    name_one, name_two = [n.strip() for n in names.split(" / ", 1)]
                else:
                    name_one, name_two = names, ""
                # Dance cells
                dance_results = {}
                for dance, cell in zip(dances, cells[2:]):
                    cell_html = str(cell)
                    cell_soup = BeautifulSoup(cell_html, "html.parser")
                    td_tag = cell_soup.find(["td", "th"])
                    judge_ranks_text = ""
                    if td_tag:
                        for content in td_tag.contents:
                            if getattr(content, "name", None) in ["br", "div"]:
                                break
                            if isinstance(content, str):
                                judge_ranks_text += content.strip()
                    judge_ranks = (
                        [int(x) for x in judge_ranks_text if x.isdigit()]
                        if judge_ranks_text
                        else []
                    )
                    score_tag = cell_soup.find("div", class_="pz")
                    if score_tag:
                        try:
                            overall_score = float(score_tag.get_text(strip=True))
                        except Exception:
                            overall_score = None
                    else:
                        score = None
                        if td_tag and td_tag.br and td_tag.br.next_sibling:
                            try:
                                score = float(td_tag.br.next_sibling.strip())
                            except Exception:
                                score = None
                        overall_score = score
                    dance_results[dance] = {
                        "judge_ranks": judge_ranks,
                        "overall_score": overall_score,
                    }
                couples.append(
                    {
                        "ranking": ranking,
                        "name_one": name_one,
                        "name_two": name_two,
                        "number": number,
                        "club": club,
                        "dances": dance_results,
                    }
                )
        parsing_logger.debug(
            f"extract_finalists_from_erg: END, total couples={len(couples)}"
        )
        return couples

    def deduplicate_judges(self, judges: List[Judge]) -> List[Judge]:
        """
        Deduplicate a list of Judge objects by (code, name).
        Returns a list of unique Judge objects.
        """
        unique = {}
        for j in judges:
            key = (j.code, j.name)
            if key not in unique:
                unique[key] = j
        return list(unique.values())

    def make_judge(
        self, code: str, name: str, club: Optional[str] = "", logger=None
    ) -> Optional[Judge]:
        """
        Safely create a Judge object, logging and returning None if creation fails.
        Args:
            code: Judge code (str)
            name: Judge name (str)
            club: Judge club (str or None)
            logger: Optional logger for warnings
        Returns:
            Judge instance or None if invalid
        """
        try:
            return Judge(code=code, name=name, club=club)
        except Exception as e:
            if logger:
                logger.warning(
                    f"Invalid judge skipped: code={code}, name={name}, club={club}, error={e}"
                )
            return None


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
    for src in ["deck", "tabges", "ergwert", "wert_er"]:
        if src in html_files:
            if src == "deck":
                judges = parser.extract_judges_from_deck(html_files[src])
            elif src == "tabges":
                judges = parser.extract_judges(html_files[src])
            elif src == "ergwert":
                judges = parser.extract_judges(html_files[src])
            elif src == "wert_er":
                judges = parser.extract_judges(html_files[src])
            else:
                judges = []
            judge_sets[src] = set(judges)
            judge_lists[src] = judges

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
    for code_name in judge_keys_by_src[src]:
        best = None
        for src in judge_keys_by_src:
            for j in judge_sets[src]:
                if judge_key(j) == code_name:
                    if not best or (j.club and not best.club):
                        best = j
        best_judges[code_name] = best

    # Log inconsistencies
    for code_name in judge_keys_by_src[src]:
        clubs = set()
        for src in judge_keys_by_src:
            for j in judge_sets[src]:
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
