from bs4 import BeautifulSoup
import re

def extract_participants(html):
    soup = BeautifulSoup(html, "html.parser")
    # Extract event name from <title>
    title_tag = soup.find('title')
    event_name = title_tag.get_text(strip=True) if title_tag else 'unknown_event'
    event_name = re.sub(r'[^\w\d-]+', '_', event_name)[:64]
    participants = []
    for table in soup.find_all('table'):
        for row in table.find_all('tr'):
            cells = row.find_all('td')
            if len(cells) < 2:
                continue
            couple_info = cells[1].get_text(" ", strip=True)
            club_tag = cells[1].find('i')
            club = club_tag.get_text(strip=True) if club_tag else None
            number_match = re.search(r'\((\d+)\)', couple_info)
            number = number_match.group(1) if number_match else None
            # Remove (number) from names
            names = re.sub(r'\s*\(\d+\)', '', couple_info).strip()
            # Split names on common delimiters
            name_one, name_two = None, None
            for delim in [' / ', ' & ', ' und ', ' and ']:
                if delim in names:
                    parts = names.split(delim)
                    if len(parts) == 2:
                        name_one, name_two = parts[0].strip(), parts[1].strip()
                        break
            if name_one is None or name_two is None:
                # Fallback: try splitting on whitespace
                parts = names.split()
                if len(parts) >= 2:
                    name_one = parts[0].strip()
                    name_two = ' '.join(parts[1:]).strip()
            if name_one and name_two and club and number:
                participants.append({
                    'name_one': name_one,
                    'name_two': name_two,
                    'number': number,
                    'club': club
                })
    return participants, event_name

def extract_judges(html):
    soup = BeautifulSoup(html, "html.parser")
    judges = []
    table = soup.find('table', class_='tab1')
    if not table:
        return judges
    found_judges = False
    for row in table.find_all('tr'):
        cells = row.find_all('td')
        if len(cells) < 2:
            continue
        if not found_judges:
            if cells[0].get_text(strip=True) == 'Wertungsrichter:':
                found_judges = True
            continue
        # After Wertungsrichter, collect all td2r rows
        if 'td2r' in cells[0].get('class', []):
            code = cells[0].get_text(strip=True).rstrip(':')
            name = ''
            club = ''
            spans = cells[1].find_all('span')
            if len(spans) >= 2:
                name = spans[0].get_text(strip=True)
                club = spans[1].get_text(strip=True)
            else:
                name = cells[1].get_text(strip=True)
            judges.append({'code': code, 'name': name, 'club': club})
        else:
            # Stop if we reach a row that is not a judge
            if found_judges:
                break
    return judges

def extract_committee(html):
    soup = BeautifulSoup(html, "html.parser")
    table = soup.find('table', class_='tab1')
    if not table:
        return []
    roles = [
        ('Veranstalter:', 'organizer'),
        ('Ausrichter:', 'host'),
        ('Turnierleiter:', 'chairperson'),
        ('Beisitzer:', 'committee_member'),
        ('Protokoll:', 'protocol')
    ]
    committee = []
    for row in table.find_all('tr'):
        cells = row.find_all('td')
        if len(cells) < 2:
            continue
        label = cells[0].get_text(strip=True)
        for role_label, role_key in roles:
            if label == role_label:
                value_cell = cells[1]
                spans = value_cell.find_all('span')
                name = ''
                club = ''
                if len(spans) >= 2:
                    name = spans[0].get_text(strip=True)
                    club = spans[1].get_text(strip=True)
                elif len(spans) == 1:
                    name = spans[0].get_text(strip=True)
                else:
                    name = value_cell.get_text(strip=True)
                committee.append({
                    'role': role_key,
                    'name': name,
                    'club': club,
                    'raw_value': value_cell.get_text(" ", strip=True)
                })
    return committee

def extract_scores(html):
    soup = BeautifulSoup(html, "html.parser")
    tables = soup.find_all('table')
    scores = []
    for table in tables:
        headers = []
        round_name = None
        for row in table.find_all('tr'):
            cells = row.find_all(['td', 'th'])
            if not cells:
                continue
            # Detect round name
            if len(cells) == 1 and cells[0].has_attr('colspan'):
                round_name = cells[0].get_text(strip=True)
                continue
            # Detect header row
            if any('Platz' in c.get_text() for c in cells):
                headers = [c.get_text(strip=True) for c in cells]
                continue
            # Data rows
            if headers and len(cells) == len(headers):
                data = {headers[i]: cells[i].get_text(" ", strip=True) for i in range(len(headers))}
                data['round'] = round_name
                scores.append(data)
            # Special case: detailed judge/dance scores (complex tables)
            # (For now, just collect all rows with a placement and couple info)
            if any('Platz' in c.get_text() for c in headers) and len(cells) > 2:
                placement = cells[0].get_text(strip=True)
                couple = cells[1].get_text(" ", strip=True)
                number = cells[2].get_text(strip=True) if len(cells) > 2 else None
                # Club may be in <i> tag
                club = ''
                if cells[1].find('i'):
                    club = cells[1].find('i').get_text(strip=True)
                scores.append({
                    'round': round_name,
                    'placement': placement,
                    'names': couple,
                    'number': number,
                    'club': club,
                    # Add more fields as needed
                })
    return scores

def extract_scores_from_tabges(html):
    soup = BeautifulSoup(html, "html.parser")
    table = soup.find('table', class_='tab1')
    if not table:
        return []
    # Get couple numbers and names from the header row
    header = table.find_all('tr')[1]
    couple_cells = header.find_all('td')[1:]
    couples = []
    for cell in couple_cells:
        num = cell.get_text(strip=True)
        tooltip = cell.find('span', class_='tooltip2gc')
        names = tooltip.get_text(strip=True) if tooltip else ''
        couples.append({'number': num, 'names': names})
    # Get judge codes and names from the next row
    judge_row = table.find_all('tr')[2]
    judge_cells = judge_row.find_all('td')[1:]
    judge_codes = []
    for cell in judge_cells:
        code = cell.get_text(strip=True).split(')')[0]
        judge_codes.append(code)
    # Parse all score rows
    scores = []
    round_name = None
    for row in table.find_all('tr')[3:]:
        cells = row.find_all('td')
        if not cells:
            continue
        # Detect round name
        if 'Ergebnis der' in cells[0].get_text():
            round_name = cells[0].get_text(strip=True)
            for i, cell in enumerate(cells[1:]):
                placement = cell.get_text(strip=True)
                if placement:
                    entry = {
                        'round': round_name,
                        'number': couples[i]['number'] if i < len(couples) else '',
                        'names': couples[i]['names'] if i < len(couples) else '',
                        'placement': placement
                    }
                    scores.append(entry)
            continue
        # Detect qualification row
        if 'Qualifiziert' in cells[0].get_text():
            qual_round = cells[0].get_text(strip=True)
            for i, cell in enumerate(cells[1:]):
                qualified = cell.get_text(strip=True)
                if qualified:
                    entry = {
                        'round': qual_round,
                        'number': couples[i]['number'] if i < len(couples) else '',
                        'names': couples[i]['names'] if i < len(couples) else '',
                        'qualified': qualified
                    }
                    scores.append(entry)
            continue
        # Per-judge scores row
        if len(cells) > 1 and cells[0].get('class', [''])[0].startswith('td3'):
            for i, cell in enumerate(cells[1:]):
                judge_scores = cell.get_text("|", strip=True).split('|')
                entry = {
                    'round': round_name,
                    'number': couples[i]['number'] if i < len(couples) else '',
                    'names': couples[i]['names'] if i < len(couples) else '',
                    'judge_scores': judge_scores
                }
                scores.append(entry)
    return scores 