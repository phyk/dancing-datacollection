from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.judge import Judge
import re


def extract_participants_from_tabges(soup):
    participants = []
    seen = set()
    # Collect all unique participants from all td2gc cells in the file
    for cell in soup.find_all("td", class_="td2gc"):
        number_str = cell.get_text(strip=True)
        match = re.search(r"\d+", number_str)
        if not match:
            continue
        number_int = int(match.group(0))
        tooltip = cell.find("span", class_="tooltip2gc")
        if tooltip:
            names = tooltip.get_text(strip=True)
            if "/" in names:
                name_one, name_two = [n.strip() for n in names.split("/", 1)]
            else:
                name_one, name_two = names.strip(), None
            key = (number_int, name_one, name_two)
            if key not in seen:
                seen.add(key)
                participants.append(
                    Participant(
                        name_one=name_one,
                        name_two=name_two,
                        number=number_int,
                        ranks=None,
                        club=None,
                    )
                )
    return participants


def extract_judges_from_tabges(soup):
    """
    Extract judges from tabges.htm. Looks for the Wertungsrichter row and parses judge codes and names.
    """
    judges = []
    table = soup.find("table", class_="tab1")
    if not table:
        return judges
    found_judges = False
    for row_idx, row in enumerate(table.find_all("tr")):
        cells = row.find_all("td")
        if len(cells) < 1:
            continue
        if not found_judges:
            if cells[0].get_text(strip=True).replace(":", "") == "Wertungsrichter":
                found_judges = True
            continue
        if found_judges and "td3" in cells[0].get("class", []):
            from bs4 import BeautifulSoup
            import re

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
    unique = {}
    for j in judges:
        key = (j.code, j.name)
        if key not in unique:
            unique[key] = j
    return list(unique.values())


def extract_scores_from_tabges(soup):
    """
    Extract scores from tabges.htm. Returns a list of dicts with at least 'number' and 'score' keys.
    """
    import re
    print('extract_scores_from_tabges: START')
    tables = soup.find_all('table', class_='tab1')
    print(f'Found {len(tables)} tables with class tab1')
    scores = []
    for table_idx, table in enumerate(tables):
        rows = table.find_all('tr')
        couple_numbers = []
        found_couples = False
        for row_idx, row in enumerate(rows):
            cells = row.find_all('td')
            print(f'Table {table_idx} Row {row_idx}: {[c.get("class") for c in cells]}')
            # Find the couple number header row
            if any('td2gc' in (c.get('class') or []) for c in cells):
                couple_numbers = []
                for c in cells[1:]:
                    text = c.get_text(strip=True)
                    match = re.match(r'(\d+)', text)
                    if match:
                        num = int(match.group(1))
                        couple_numbers.append(num)
                    else:
                        couple_numbers.append(None)
                found_couples = True
                print(f'Table {table_idx} Row {row_idx}: Found couple numbers: {couple_numbers}')
                continue
            # Parse score rows (class 'td5c')
            if any('td5c' in (c.get('class') or []) for c in cells) and found_couples:
                print(f'Table {table_idx} Row {row_idx}: Parsing score row')
                for couple_idx, c in enumerate(cells[1:]):
                    if couple_idx >= len(couple_numbers):
                        continue
                    couple_number = couple_numbers[couple_idx]
                    if couple_number is None:
                        continue
                    values = c.get_text(" ", strip=True).replace('\xa0', '').split("\n")
                    for value in values:
                        value = value.strip()
                        if value == '' or value == '&nbsp;':
                            continue
                        try:
                            score = int(value)
                        except Exception:
                            continue
                        entry = {
                            'number': couple_number,
                            'score': score,
                        }
                        scores.append(entry)
    print(f'Extracted {len(scores)} scores in total')
    return scores
