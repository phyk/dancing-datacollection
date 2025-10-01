import logging
import re
from typing import Any, Dict, List, Union, cast

from bs4 import BeautifulSoup
from bs4.element import Tag
from pydantic import ValidationError

from dancing_datacollection.data_defs.dances import (
    GERMAN_TO_ENGLISH_DANCE_NAME,
    Dance,
)
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.results import (
    DanceScore,
    FinalRoundPlacing,
    PreliminaryRoundPlacing,
    ResultRound,
)
from dancing_datacollection.parsing.parsing_utils import get_soup

parsing_logger = logging.getLogger("parsing_debug")


def extract_results_from_erg(html: str) -> List[ResultRound]:
    soup = get_soup(html)
    results: List[ResultRound] = []

    all_tables = soup.find_all("table")
    if len(all_tables) < 2:
        return []

    result_tables = all_tables[1:]

    # First table is always the final round
    final_round_table = result_tables[0]
    if not isinstance(final_round_table, Tag):
        return []
    final_rows = final_round_table.find_all("tr")
    if final_rows:
        if not isinstance(final_rows[0], Tag) or not isinstance(final_rows[1], Tag):
            return []
        round_name = final_rows[0].get_text(strip=True)
        header_row = final_rows[1]
        if not isinstance(header_row, Tag):
            return []
        header_cells = header_row.find_all("td")
        dance_names = [h.get_text(strip=True) for h in header_cells[2:-1]]
        placings: list[Union[FinalRoundPlacing, PreliminaryRoundPlacing]] = []
        for row in final_rows[2:]:
            if not isinstance(row, Tag):
                continue
            cells = row.find_all("td")
            if len(cells) < 2:
                continue

            rank = cells[0].get_text(strip=True)
            couple_cell = cells[1]
            if not isinstance(couple_cell, Tag):
                continue
            name_text = couple_cell.get_text(separator="|", strip=True)
            name_parts = name_text.split("|")
            full_name = name_parts[0]

            number_match = re.search(r"\((\d+)\)", full_name)
            if not number_match:
                parsing_logger.warning("Could not parse number from %s", full_name)
                continue
            number = int(number_match.group(1))

            clean_name = re.sub(r"\s*\(\d+\)", "", full_name).strip()
            name_one, name_two = (
                (clean_name.split(" / ", 1) + [None])[:2]
                if " / " in clean_name
                else (clean_name, None)
            )
            if not name_one:
                parsing_logger.warning("Could not parse name from %s", full_name)
                continue

            club_tag = couple_cell.find("i")
            club = (
                club_tag.get_text(strip=True)
                if club_tag and isinstance(club_tag, Tag)
                else None
            )

            try:
                participant = Participant(
                    name_one=name_one,
                    name_two=name_two,
                    number=number,
                    club=club,
                    ranks=rank,  # type: ignore
                )
            except ValidationError as e:
                parsing_logger.warning("Skipping participant due to validation error: %s", e)
                continue

            dance_scores: Dict[Dance, DanceScore] = {}
            for i, dn in enumerate(dance_names):
                dance_enum = GERMAN_TO_ENGLISH_DANCE_NAME.get(dn)
                if not dance_enum:
                    parsing_logger.warning("Unknown dance abbreviation: %s", dn)
                    continue
                score_cell = cells[i + 2]
                if isinstance(score_cell, Tag):
                    score_cell_html = str(score_cell.decode_contents())
                    parts = score_cell_html.split("<br/>")
                    marks_str = parts[0].strip()
                    marks = [int(char) for char in marks_str if char.isdigit()]
                    place_str_match = (
                        re.search(r"[\d\.]+", parts[1]) if len(parts) > 1 else None
                    )
                    place = float(place_str_match.group(0)) if place_str_match else 0.0
                    dance_scores[dance_enum] = DanceScore(marks=marks, place=place)

            total_score_str = cells[-1].get_text(strip=True)
            total_score = float(total_score_str) if total_score_str else 0.0

            placing = FinalRoundPlacing(
                rank=rank,
                participant=participant,
                dance_scores=dance_scores,
                total_score=total_score,
            )
            placings.append(placing)

        if placings:
            results.append(ResultRound(name=round_name, placings=placings))

    # Second table contains all preliminary rounds
    if len(result_tables) > 1:
        prelim_table = result_tables[1]
        if not isinstance(prelim_table, Tag):
            return results
        current_round_name = ""
        current_placings: list[Union[FinalRoundPlacing, PreliminaryRoundPlacing]] = []

        for row in prelim_table.find_all("tr"):
            if not isinstance(row, Tag):
                continue
            cells = row.find_all("td")
            if len(cells) == 1:
                if current_round_name and current_placings:
                    results.append(
                        ResultRound(name=current_round_name, placings=current_placings)
                    )
                current_round_name = cells[0].get_text(strip=True)
                current_placings = []
            elif len(cells) >= 2 and current_round_name:
                rank = cells[0].get_text(strip=True)
                name_text = cells[1].get_text(strip=True)
                club = cells[2].get_text(strip=True) if len(cells) > 2 else None

                number_match = re.search(r"\((\d+)\)", name_text)
                if not number_match:
                    parsing_logger.warning("Could not parse number from %s", name_text)
                    continue
                number = int(number_match.group(1))
                clean_name = re.sub(r"\s*\(\d+\)", "", name_text).strip()
                name_one, name_two = (
                    (clean_name.split(" / ", 1) + [None])[:2]
                    if " / " in clean_name
                    else (clean_name, None)
                )
                if not name_one:
                    parsing_logger.warning("Could not parse name from %s", name_text)
                    continue

                try:
                    participant = Participant(
                        name_one=name_one,
                        name_two=name_two,
                        number=number,
                        club=club,
                        ranks=rank,  # type: ignore
                    )
                except ValidationError as e:
                    parsing_logger.warning(
                        "Skipping participant due to validation error: %s", e
                    )
                    continue

                placing = PreliminaryRoundPlacing(rank=rank, participant=participant)
                current_placings.append(placing)

        if current_round_name and current_placings:
            results.append(ResultRound(name=current_round_name, placings=current_placings))

    return results


def extract_participants_from_erg(
    soup_or_html: Union[str, BeautifulSoup],
) -> List[Participant]:
    """
    Maintains compatibility with the old interface by extracting a flat list of participants.
    """
    html = str(soup_or_html) if isinstance(soup_or_html, BeautifulSoup) else soup_or_html

    results = extract_results_from_erg(html)
    participants: List[Participant] = []
    seen_numbers = set()
    for res_round in results:
        for placing in res_round.placings:
            if placing.participant.number not in seen_numbers:
                participants.append(placing.participant)
                seen_numbers.add(placing.participant.number)
    return participants


def extract_judges_from_erg(soup: BeautifulSoup) -> List[Judge]:
    """
    erg.htm does not contain judge information in TopTurnier format. Always returns an empty list.
    """
    return []


def parse_erg_all(html: str) -> List[Any]:
    """Lightweight dump of erg.htm tables for inspection (dev aid)."""
    parsing_logger.debug("parse_erg_all: START")
    soup: Any = get_soup(html)
    result = []
    for table_idx, table in enumerate(cast(Any, soup).find_all("table")):
        rows_dump = []
        for row_idx, row in enumerate(cast(Any, table).find_all("tr")):
            cells: List[Any] = cast(Any, row).find_all(["td", "th"])
            rows_dump.append(
                {
                    "row_idx": row_idx,
                    "cells": [c.get_text(" ", strip=True) for c in cells],
                }
            )
        result.append({"table_idx": table_idx, "rows": rows_dump})
    parsing_logger.debug("parse_erg_all: END")
    return result


def extract_finalists_from_erg(html: str) -> List[Any]:
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
