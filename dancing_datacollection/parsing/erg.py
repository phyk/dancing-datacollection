from dancing_datacollection.data_defs.participant import Participant
import re

def extract_participants_from_erg(soup):
    participants = []
    # Default: try to extract from erg.htm (detailed)
    table = soup.find('table', class_='tab1')
    if table:
        rows = table.find_all('tr')
        for row_idx, row in enumerate(rows):
            cells = row.find_all('td')
            if len(cells) < 3:
                continue
            rank_str = cells[0].get_text(strip=True)
            couple_cell = cells[1]
            names = couple_cell.get_text(" ", strip=True)
            club = None
            club_tag = couple_cell.find('i')
            if club_tag:
                club = club_tag.get_text(strip=True)
            # Extract number from names cell using regex
            number = None
            match = re.search(r'\((\d+)\)', names)
            if match:
                number = int(match.group(1))
            name_one = None
            name_two = None
            if '/' in names:
                name_one = names.split('/')[0].strip()
                name_two = names.split('/')[1].strip()
            else:
                name_one = names.strip()
            if name_one and number:
                try:
                    p = Participant(name_one=name_one, name_two=name_two, number=number, ranks=rank_str, club=club)
                    participants.append(p)
                except Exception as e:
                    print(f'Invalid participant skipped (erg): {e}', flush=True)
    return participants 

def extract_judges_from_erg(soup):
    """
    erg.htm does not contain judge information in TopTurnier format. Always returns an empty list.
    """
    return [] 