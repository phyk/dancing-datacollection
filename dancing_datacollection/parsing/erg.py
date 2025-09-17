from dancing_datacollection.data_defs.participant import Participant
import re
from typing import Any, List
from dancing_datacollection.parsing_utils import get_soup


def extract_participants_from_erg(soup_or_html):
    # Accept either soup or raw html for convenience
    if isinstance(soup_or_html, str):
        soup = get_soup(soup_or_html)
    else:
        soup = soup_or_html
    participants = []
    # Extract from erg.htm (detailed, final round)
    table = soup.find("table", class_="tab1")
    if table:
        rows = table.find_all("tr")
        for row_idx, row in enumerate(rows):
            cells = row.find_all("td")
            if len(cells) < 3:
                continue
            # 1. Extract rank as list of ints
            rank_str = cells[0].get_text(strip=True)
            ranks = Participant._parse_ranks(rank_str)
            # 2. Extract names and number
            couple_cell = cells[1]
            names_texts = couple_cell.find_all(string=True, recursive=False)
            names = (
                names_texts[0].strip()
                if names_texts
                else couple_cell.get_text(" ", strip=True)
            )
            # 3. Extract club
            club = None
            club_tag = couple_cell.find("i")
            if club_tag:
                club = club_tag.get_text(strip=True)
            elif len(cells) > 2:
                club = cells[2].get_text(strip=True)
            # 4. Extract number
            number = None
            match = re.search(r"\((\d+)\)", names)
            if match:
                number = int(match.group(1))
            # 5. Extract name_one and name_two
            name_one = None
            name_two = None
            if "/" in names:
                name_one, name_two = [n.strip() for n in names.split("/", 1)]
                name_two = re.sub(r"\(\d+\)", "", name_two).strip()
            else:
                name_one = names.strip()
            # 6. Only add if required fields are present
            if name_one and number:
                try:
                    p = Participant(
                        name_one=name_one,
                        name_two=name_two,
                        number=number,
                        ranks=ranks,
                        club=club,
                    )
                    participants.append(p)
                except Exception as e:
                    print(f"Invalid participant skipped (erg): {e}", flush=True)
    # Extract from tab2 and similar tables (earlier rounds)
    for table in soup.find_all("table"):
        if table.get("class") and "tab1" in table.get("class", []):
            continue  # already processed above
        rows = table.find_all("tr")
        for row in rows:
            cells = row.find_all("td")
            if len(cells) < 2:
                continue
            # 1. Extract rank as list of ints
            rank_str = cells[0].get_text(strip=True)
            ranks = Participant._parse_ranks(rank_str)
            # 2. Extract names and number
            names = (
                cells[1].get_text(" ", strip=True)
                if len(cells) > 1
                else cells[0].get_text(" ", strip=True)
            )
            # 3. Extract club
            club = cells[2].get_text(strip=True) if len(cells) > 2 else None
            # 4. Extract number
            number = None
            match = re.search(r"\((\d+)\)", names)
            if match:
                number = int(match.group(1))
            # 5. Extract name_one and name_two
            name_one = None
            name_two = None
            if "/" in names:
                name_one, name_two = [n.strip() for n in names.split("/", 1)]
                name_two = re.sub(r"\(\d+\)", "", name_two).strip()
            else:
                name_one = names.strip()
            # 6. Only add if required fields are present and deduplicate
            if name_one and number:
                if not any(
                    p.number == number
                    and p.name_one == name_one
                    and p.name_two == name_two
                    and p.club == club
                    for p in participants
                ):
                    try:
                        p = Participant(
                            name_one=name_one,
                            name_two=name_two,
                            number=number,
                            ranks=ranks,
                            club=club,
                        )
                        participants.append(p)
                    except Exception as e:
                        print(
                            f"Invalid participant skipped (erg, tab2): {e}", flush=True
                        )
    return participants


def extract_judges_from_erg(soup):
    """
    erg.htm does not contain judge information in TopTurnier format. Always returns an empty list.
    """
    return []
