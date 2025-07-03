from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.judge import Judge
import re


def extract_participants_from_wert_er(soup):
    participants = []
    table = soup.find("table", class_="tab1")
    seen_numbers = set()
    if table:
        for cell in table.find_all("td", class_="td3r"):
            number_str = cell.get_text(strip=True)
            number_int = None
            match = re.search(r"\d+", number_str)
            if match:
                number_int = int(match.group(0))
            if number_int in seen_numbers:
                continue
            seen_numbers.add(number_int)
            name_one = None
            name_two = None
            tooltip = cell.find("span", class_="tooltip3r")
            if tooltip:
                names = tooltip.get_text(strip=True)
                if "/" in names:
                    name_one = names.split("/")[0].strip()
                    name_two = names.split("/")[1].strip()
                else:
                    name_one = names.strip()
            participant = Participant(
                name_one=name_one,
                name_two=name_two,
                number=number_int,
                ranks=None,
                club=None,
            )
            if participant.name_one and participant.number is not None:
                participants.append(participant)
            else:
                print(
                    f"Invalid participant skipped (wert_er): name_one={participant.name_one}, number={participant.number}",
                    flush=True,
                )
    return participants


def extract_judges_from_wert_er(soup):
    """
    Extract judges from wert_er.htm. Looks for the second row and parses judge codes and names from spans.
    """
    judges = []
    table = soup.find("table", class_="tab1")
    if not table:
        return judges
    rows = table.find_all("tr")
    if len(rows) < 2:
        return judges
    judge_cells = rows[1].find_all(["td", "th"])
    for cell in judge_cells:
        text = cell.get_text(strip=True)
        span = cell.find("span")
        if span:
            code = (
                cell.contents[0].strip()
                if cell.contents and isinstance(cell.contents[0], str)
                else ""
            )
            name = span.get_text(strip=True)
            if len(code) == 2 and code.isupper():
                judge = Judge(code=code, name=name, club="")
                judges.append(judge)
    # Deduplicate by (code, name)
    unique = {}
    for j in judges:
        key = (j.code, j.name)
        if key not in unique:
            unique[key] = j
    return list(unique.values())
