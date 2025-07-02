from .parsing_base import CompetitionParser
from bs4 import BeautifulSoup
import re
from .parsing_utils import get_soup, extract_club_and_number, split_names, extract_name_and_club_from_spans
import logging

parsing_logger = logging.getLogger('parsing_debug')

class TopTurnierParser(CompetitionParser):
    def extract_participants(self, html):
        parsing_logger.debug('extract_participants: START')
        soup = get_soup(html)
        title_tag = soup.find('title')
        event_name = title_tag.get_text(strip=True) if title_tag else 'unknown_event'
        event_name = re.sub(r'[^\w\d-]+', '_', event_name)[:64]
        parsing_logger.debug(f'Event name: {event_name}')
        participants = []
        for table_idx, table in enumerate(soup.find_all('table')):
            for row_idx, row in enumerate(table.find_all('tr')):
                cells = row.find_all('td')
                parsing_logger.debug(f'Row {row_idx} in table {table_idx}: {[c.get_text(" ", strip=True) for c in cells]}')
                if len(cells) < 2:
                    continue
                couple_info = cells[1].get_text(" ", strip=True)
                parsing_logger.debug(f'  couple_info: {couple_info}')
                club, number = extract_club_and_number(cells[1])
                parsing_logger.debug(f'  club: {club}, number: {number}')
                names = re.sub(r'\s*\(\d+\)', '', couple_info).strip()
                parsing_logger.debug(f'  names (after removing number): {names}')
                name_one, name_two = split_names(names)
                parsing_logger.debug(f'  name_one: {name_one}, name_two: {name_two}')
                if name_one and name_two and club and number:
                    participants.append({
                        'name_one': name_one,
                        'name_two': name_two,
                        'number': number,
                        'club': club
                    })
        parsing_logger.debug(f'extract_participants: END, total participants={len(participants)}')
        return participants, event_name

    def extract_judges(self, html):
        parsing_logger.debug('extract_judges: START')
        soup = get_soup(html)
        judges = []
        table = soup.find('table', class_='tab1')
        parsing_logger.debug(f'Found table: {bool(table)}')
        if not table:
            return judges
        found_judges = False
        for row_idx, row in enumerate(table.find_all('tr')):
            cells = row.find_all('td')
            parsing_logger.debug(f'Row {row_idx}: {[c.get_text(" ", strip=True) for c in cells]}')
            if len(cells) < 2:
                continue
            if not found_judges:
                if cells[0].get_text(strip=True) == 'Wertungsrichter:':
                    found_judges = True
                continue
            if 'td2r' in cells[0].get('class', []):
                code = cells[0].get_text(strip=True).rstrip(':')
                name, club = extract_name_and_club_from_spans(cells[1])
                parsing_logger.debug(f'  Judge: code={code}, name={name}, club={club}')
                judges.append({'code': code, 'name': name, 'club': club})
            else:
                if found_judges:
                    break
        parsing_logger.debug(f'extract_judges: END, total judges={len(judges)}')
        return judges

    def extract_committee(self, html):
        parsing_logger.debug('extract_committee: START')
        soup = get_soup(html)
        table = soup.find('table', class_='tab1')
        parsing_logger.debug(f'Found table: {bool(table)}')
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
        for row_idx, row in enumerate(table.find_all('tr')):
            cells = row.find_all('td')
            parsing_logger.debug(f'Row {row_idx}: {[c.get_text(" ", strip=True) for c in cells]}')
            if len(cells) < 2:
                continue
            label = cells[0].get_text(strip=True)
            for role_label, role_key in roles:
                if label == role_label:
                    value_cell = cells[1]
                    name, club = extract_name_and_club_from_spans(value_cell)
                    parsing_logger.debug(f'  Committee: role={role_key}, name={name}, club={club}, raw_value={value_cell.get_text(" ", strip=True)}')
                    committee.append({
                        'role': role_key,
                        'name': name,
                        'club': club,
                        'raw_value': value_cell.get_text(" ", strip=True)
                    })
        parsing_logger.debug(f'extract_committee: END, total committee={len(committee)}')
        return committee

    def extract_scores(self, html):
        parsing_logger.debug('extract_scores: START')
        soup = get_soup(html)
        table = soup.find('table', class_='tab1')
        parsing_logger.debug(f'Found table: {bool(table)}')
        if not table:
            return []
        header = table.find_all('tr')[1]
        couple_cells = header.find_all('td')[1:]
        couples = []
        for idx, cell in enumerate(couple_cells):
            num = cell.get_text(strip=True)
            tooltip = cell.find('span', class_='tooltip2gc')
            names = tooltip.get_text(strip=True) if tooltip else ''
            parsing_logger.debug(f'Couple {idx}: number={num}, names={names}')
            couples.append({'number': num, 'names': names})
        judge_row = table.find_all('tr')[2]
        judge_cells = judge_row.find_all('td')[1:]
        judge_codes = []
        for idx, cell in enumerate(judge_cells):
            code = cell.get_text(strip=True).split(')')[0]
            parsing_logger.debug(f'Judge {idx}: code={code}')
            judge_codes.append(code)
        scores = []
        round_name = None
        for row_idx, row in enumerate(table.find_all('tr')[3:]):
            cells = row.find_all('td')
            parsing_logger.debug(f'Row {row_idx+3}: {[c.get_text(" ", strip=True) for c in cells]}')
            if not cells:
                continue
            if 'Ergebnis der' in cells[0].get_text():
                round_name = cells[0].get_text(strip=True)
                parsing_logger.debug(f'  round_name: {round_name}')
                for i, cell in enumerate(cells[1:]):
                    placement = cell.get_text(strip=True)
                    if placement:
                        entry = {
                            'round': round_name,
                            'number': couples[i]['number'] if i < len(couples) else '',
                            'names': couples[i]['names'] if i < len(couples) else '',
                            'placement': placement
                        }
                        parsing_logger.debug(f'  Ergebnis entry: {entry}')
                        scores.append(entry)
                continue
            if 'Qualifiziert' in cells[0].get_text():
                qual_round = cells[0].get_text(strip=True)
                parsing_logger.debug(f'  qual_round: {qual_round}')
                for i, cell in enumerate(cells[1:]):
                    qualified = cell.get_text(strip=True)
                    if qualified:
                        entry = {
                            'round': qual_round,
                            'number': couples[i]['number'] if i < len(couples) else '',
                            'names': couples[i]['names'] if i < len(couples) else '',
                            'qualified': qualified
                        }
                        parsing_logger.debug(f'  Qualifiziert entry: {entry}')
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
                    parsing_logger.debug(f'  Judge scores entry: {entry}')
                    scores.append(entry)
        norm = []
        for s in scores:
            norm.append({k: s.get(k, None) for k in ['round', 'number', 'names']})
        parsing_logger.debug(f'extract_scores: END, total scores={len(scores)}')
        return norm

    def extract_final_scoring(self, html):
        parsing_logger.debug('extract_final_scoring: START')
        soup = get_soup(html)
        table = soup.find('table', class_='tab1')
        parsing_logger.debug(f'Found table: {bool(table)}')
        if not table:
            return []
        rows = table.find_all('tr')
        final_scores = []
        for row_idx, row in enumerate(rows):
            cells = row.find_all('td')
            parsing_logger.debug(f'Row {row_idx}: {[c.get_text(" ", strip=True) for c in cells]}')
            if not cells or not cells[0].get('class', []):
                continue
            if 'td3cv' in cells[0].get('class', []):
                placement = cells[0].get_text(strip=True)
                couple_cell = cells[1]
                names = couple_cell.get_text(" ", strip=True)
                club, _ = extract_club_and_number(couple_cell)
                number = cells[2].get_text(strip=True)
                lw_score = cells[9].get_text(strip=True) if len(cells) > 9 else ''
                tg_score = cells[15].get_text(strip=True) if len(cells) > 15 else ''
                qs_score = cells[21].get_text(strip=True) if len(cells) > 21 else ''
                total = cells[-1].get_text(strip=True) if cells[-1].get('class', [''])[0].startswith('tddarkc') else ''
                entry = {
                    'placement': placement,
                    'names': names,
                    'number': number,
                    'club': club,
                    'score_LW': lw_score,
                    'score_TG': tg_score,
                    'score_QS': qs_score,
                    'total': total
                }
                parsing_logger.debug(f'  Final scoring entry: {entry}')
                final_scores.append(entry)
        parsing_logger.debug(f'extract_final_scoring: END, total final_scores={len(final_scores)}')
        return final_scores 