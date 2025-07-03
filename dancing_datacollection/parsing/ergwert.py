from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.judge import Judge
import re

def extract_participants_from_ergwert(soup):
    participants = []
    table = soup.find('table', class_='tab1')
    if table:
        for row in table.find_all('tr'):
            cells = row.find_all('td')
            if len(cells) >= 3:
                cell0_class = cells[0].get('class')
                if cell0_class and 'td3cv' in (cell0_class if isinstance(cell0_class, str) else ' '.join(cell0_class)):
                    rank_str = cells[0].get_text(strip=True)
                    names_cell = cells[1]
                    number_cell = cells[2]
                    names = names_cell.get_text(" ", strip=True)
                    club = None
                    club_tag = names_cell.find('i')
                    if club_tag:
                        club = club_tag.get_text(strip=True)
                    number_str = number_cell.get_text(strip=True)
                    number_int = None
                    match = re.search(r'\d+', number_str)
                    if match:
                        number_int = int(match.group(0))
                    name_one = None
                    name_two = None
                    if '/' in names:
                        name_one = names.split('/')[0].strip()
                        name_two = names.split('/')[1].strip()
                    else:
                        name_one = names.strip()
                    participant = Participant(
                        name_one=name_one,
                        name_two=name_two,
                        number=number_int,
                        ranks=[int(r) for r in re.findall(r'\d+', rank_str)] if rank_str else None,
                        club=club
                    )
                    if participant.name_one and participant.number is not None:
                        participants.append(participant)
                    else:
                        print(f'Invalid participant skipped (ergwert): name_one={participant.name_one}, number={participant.number}', flush=True)
    return participants 

def extract_judges_from_ergwert(soup):
    """
    Extract judges from ergwert.htm. Looks for the second row and parses judge codes and names from spans.
    """
    judges = []
    tables = soup.find_all('table', class_='tab1')
    for table in tables:
        rows = table.find_all('tr')
        if len(rows) < 2:
            continue
        second_row = rows[1]
        cells = second_row.find_all(['td', 'th'])
        for cell in cells:
            span = cell.find('span')
            text = cell.get_text(strip=True)
            if span:
                code = text.replace(span.get_text(strip=True), '').strip()
                name = span.get_text(strip=True)
                judge = Judge(code=code, name=name, club='')
                judges.append(judge)
    # Deduplicate by (code, name)
    unique = {}
    for j in judges:
        key = (j.code, j.name)
        if key not in unique:
            unique[key] = j
    return list(unique.values()) 