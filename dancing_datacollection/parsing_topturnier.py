from .parsing_base import CompetitionParser
from bs4 import BeautifulSoup
import re

class TopTurnierParser(CompetitionParser):
    def extract_participants(self, html):
        soup = BeautifulSoup(html, "html.parser")
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

    def extract_judges(self, html):
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
                if found_judges:
                    break
        return judges

    def extract_committee(self, html):
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

    def extract_scores(self, html):
        soup = BeautifulSoup(html, "html.parser")
        table = soup.find('table', class_='tab1')
        if not table:
            # Always return at least an empty list
            return []
        header = table.find_all('tr')[1]
        couple_cells = header.find_all('td')[1:]
        couples = []
        for cell in couple_cells:
            num = cell.get_text(strip=True)
            tooltip = cell.find('span', class_='tooltip2gc')
            names = tooltip.get_text(strip=True) if tooltip else ''
            couples.append({'number': num, 'names': names})
        judge_row = table.find_all('tr')[2]
        judge_cells = judge_row.find_all('td')[1:]
        judge_codes = []
        for cell in judge_cells:
            code = cell.get_text(strip=True).split(')')[0]
            judge_codes.append(code)
        scores = []
        round_name = None
        for row in table.find_all('tr')[3:]:
            cells = row.find_all('td')
            if not cells:
                continue
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
        # Normalize: ensure all dicts have 'round', 'number', 'names'
        norm = []
        for s in scores:
            norm.append({k: s.get(k, None) for k in ['round', 'number', 'names']})
        return norm

    def extract_final_scoring(self, html):
        soup = BeautifulSoup(html, "html.parser")
        table = soup.find('table', class_='tab1')
        if not table:
            return []
        rows = table.find_all('tr')
        final_scores = []
        for row in rows:
            cells = row.find_all('td')
            if not cells or not cells[0].get('class', []):
                continue
            # Only process rows with class td3cv (placement in final)
            if 'td3cv' in cells[0].get('class', []):
                placement = cells[0].get_text(strip=True)
                couple_cell = cells[1]
                names = couple_cell.get_text(" ", strip=True)
                club_tag = couple_cell.find('i')
                club = club_tag.get_text(strip=True) if club_tag else ''
                number = cells[2].get_text(strip=True)
                # Per-dance scores are in cells with class td3www (summary per dance)
                lw_score = cells[9].get_text(strip=True) if len(cells) > 9 else ''
                tg_score = cells[15].get_text(strip=True) if len(cells) > 15 else ''
                qs_score = cells[21].get_text(strip=True) if len(cells) > 21 else ''
                # Total is in the last cell with class tddarkc
                total = cells[-1].get_text(strip=True) if cells[-1].get('class', [''])[0].startswith('tddarkc') else ''
                final_scores.append({
                    'placement': placement,
                    'names': names,
                    'number': number,
                    'club': club,
                    'score_LW': lw_score,
                    'score_TG': tg_score,
                    'score_QS': qs_score,
                    'total': total
                })
        return final_scores 