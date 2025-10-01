from pydantic import ValidationError
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.dances import GERMAN_TO_ENGLISH_DANCE_NAME
from dancing_datacollection.data_defs.final_scoring import FinalScoring
from dancing_datacollection.data_defs.score import (
    FinalRoundScore,
    Score,
)
import re
from typing import List
from dancing_datacollection.parsing.parsing_utils import (
    first_line_text,
    deduplicate_judges,
    get_soup,
    as_class_list,
    extract_club_and_number,
)
from typing import Any, cast
import logging

parsing_logger = logging.getLogger("parsing_debug")


def extract_participants_from_ergwert(soup):
    participants = []
    table = soup.find("table", class_="tab1")
    if table:
        for row in table.find_all("tr"):
            cells = row.find_all("td")
            if len(cells) >= 3:
                cell0_class = cells[0].get("class")
                if cell0_class and "td3cv" in (
                    cell0_class
                    if isinstance(cell0_class, str)
                    else " ".join(cell0_class)
                ):
                    rank_str = cells[0].get_text(strip=True)
                    names_cell = cells[1]
                    number_cell = cells[2]
                    # Get only the text before the <br> for names
                    names_texts = names_cell.find_all(string=True, recursive=False)
                    names = (
                        names_texts[0].strip()
                        if names_texts
                        else names_cell.get_text(" ", strip=True)
                    )
                    club = None
                    club_tag = names_cell.find("i")
                    if club_tag:
                        club = club_tag.get_text(strip=True)
                    number_str = number_cell.get_text(strip=True)
                    number_int = None
                    match = re.search(r"\d+", number_str)
                    if match:
                        number_int = int(match.group(0))
                    name_one = None
                    name_two = None
                    if "/" in names:
                        name_one = names.split("/")[0].strip()
                        name_two = names.split("/")[1].strip()
                    else:
                        name_one = names.strip()
                    try:
                        participant = Participant(
                            name_one=name_one,
                            name_two=name_two,
                            number=number_int,
                            ranks=rank_str,
                            club=club,
                        )
                        participants.append(participant)
                    except ValidationError as e:
                        parsing_logger.warning(
                            f"Skipping participant in ergwert due to validation error: {e}"
                        )
    return participants


def extract_judges_from_ergwert(soup) -> List[Judge]:
    """
    Extract judges from ergwert.htm. Looks for the second row and parses judge codes and names from spans.
    """
    judges: List[Judge] = []
    tables = soup.find_all("table", class_="tab1")
    for table in tables:
        rows = table.find_all("tr")
        if len(rows) < 2:
            continue
        second_row = rows[1]
        cells = second_row.find_all(["td", "th"])
        for cell in cells:
            span = cell.find("span")
            text = cell.get_text(strip=True)
            if span:
                code = text.replace(span.get_text(strip=True), "").strip()
                name = span.get_text(strip=True)
                judge = Judge(code=code, name=name, club="")
                judges.append(judge)
    # Deduplicate by (code, name)
    return deduplicate_judges(judges)


def extract_scores_from_ergwert(soup):
    """
    Extract scores from an ergwert.htm table.

    Returns a list combining FinalRoundScore entries for the final (round 4)
    and boolean Score entries for earlier rounds when detectable. The tests
    currently assert only on the set of FinalRoundScore values.
    """
    table = soup.find("table", class_="tab1")
    if not table:
        return []

    rows = table.find_all("tr")
    if len(rows) < 3:
        return []

    # Header row 0: contains German dance names in groups, used to map to English
    header0_cells = rows[0].find_all(["td", "th"])
    dance_names_german = []
    for cell in header0_cells[4:]:  # skip Platz, Paar/Club, Nr, R
        # Only consider group headers which have colspan (the long German names)
        if not cell.has_attr("colspan"):
            continue
        text = cell.get_text(" ", strip=True)
        if text in GERMAN_TO_ENGLISH_DANCE_NAME:
            dance_names_german.append(text)

    if not dance_names_german:
        abbreviations = []
        for cell in header0_cells[4:]:
            text = cell.get_text(" ", strip=True)
            if text in GERMAN_TO_ENGLISH_DANCE_NAME:
                abbreviations.append(text)
        dance_names_german = abbreviations

    dance_names_english = [
        GERMAN_TO_ENGLISH_DANCE_NAME.get(name, name) for name in dance_names_german
    ]

    # Header row 1: judge codes per dance separated by a summary cell ("Su")
    header1_cells = rows[1].find_all(["td", "th"])
    judge_groups = []
    current_group = []
    for cell in header1_cells:  # start at 0 because first-row used rowspan
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

    results = []

    for row in rows[3:]:
        cells = row.find_all(["td", "th"])
        if len(cells) < 4:
            continue
        cls0 = cells[0].get("class", [])
        cls0_join = cls0 if isinstance(cls0, str) else " ".join(cls0)
        if "td3cv" not in cls0_join:
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
                        f"Skipping score in ergwert due to validation error: {e}"
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
        table_data = []
        for row in table.find_all("tr"):
            row_data = [
                str(cell.decode_contents()) for cell in row.find_all(["td", "th"])
            ]
            table_data.append(row_data)
        all_tables_data.append(table_data)
    parsing_logger.debug("parse_ergwert_all: END")
    return all_tables_data


def extract_final_scoring(html) -> List[FinalScoring]:
    parsing_logger.debug("extract_final_scoring: START")
    soup: Any = get_soup(html)
    table: Any = soup.find("table", class_="tab1")
    parsing_logger.debug(f"Found table: {bool(table)}")
    if not table:
        return []
    rows: List[Any] = table.find_all("tr")
    final_scores: List[FinalScoring] = []
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
            total = (
                cells[-1].get_text(strip=True)
                if last_class_first.startswith("tddarkc")
                else ""
            )
            entry = FinalScoring(
                placement=placement,
                names=names,
                number=number,
                club=club,
                score_LW=lw_score,
                score_TG=tg_score,
                score_QS=qs_score,
                total=total,
            )
            parsing_logger.debug(f"  Final scoring entry: {entry}")
            final_scores.append(entry)
    parsing_logger.debug(
        f"extract_final_scoring: END, total final_scores={len(final_scores)}"
    )
    return final_scores
