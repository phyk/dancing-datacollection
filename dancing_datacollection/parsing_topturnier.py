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
                try:
                    number = int(number) if number is not None else None
                except Exception:
                    number = None
                parsing_logger.debug(f'  club: {club}, number: {number}')
                names = re.sub(r'\s*\(\d+\)', '', couple_info).strip()
                parsing_logger.debug(f'  names (after removing number): {names}')
                name_one, name_two = split_names(names)
                # Remove club from name_two if present
                if name_two and club and club in name_two:
                    name_two = name_two.replace(club, '').strip()
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
                        'club': club
                    })
        parsing_logger.debug(f'extract_committee: END, total committee={len(committee)}')
        return committee

    def extract_scores(self, html):
        parsing_logger.debug('extract_scores: START')
        soup = get_soup(html)
        tables = soup.find_all('table', class_='tab1')
        parsing_logger.debug(f'Found {len(tables)} tables with class tab1')
        scores = []
        for table_idx, table in enumerate(tables):
            rows = table.find_all('tr')
            if len(rows) < 3:
                parsing_logger.error(f'Not enough rows in scores table {table_idx}')
                continue
            idx = 0
            while idx < len(rows):
                row = rows[idx]
                cells = row.find_all('td')
                # Detect start of a round
                if len(cells) > 0 and 'Ergebnis der' in cells[0].get_text(strip=True):
                    round_name = cells[0].get_text(strip=True)
                    # Find the couple number row: look ahead for a row where most cells are numeric
                    couple_row = None
                    lookahead = 1
                    while idx + lookahead < len(rows):
                        candidate_cells = rows[idx + lookahead].find_all('td')[1:]
                        if not candidate_cells:
                            lookahead += 1
                            continue
                        numeric_count = 0
                        for cell in candidate_cells:
                            val = cell.get_text(strip=True)
                            if val.isdigit():
                                numeric_count += 1
                        if numeric_count >= max(1, len(candidate_cells) // 2):
                            couple_row = rows[idx + lookahead]
                            break
                        lookahead += 1
                    if couple_row is None:
                        parsing_logger.warning(f'No valid couple number row found for round {round_name} in table {table_idx}')
                        idx += lookahead
                        continue
                    couple_cells = couple_row.find_all('td')[1:]
                    couples = []
                    for cidx, cell in enumerate(couple_cells):
                        num = cell.get_text(strip=True)
                        try:
                            num = int(num)
                        except Exception:
                            continue
                        couples.append(num)
                    # Print all rows and their cell counts after the couple number row
                    score_row_idx = idx + lookahead + 1
                    debug_row_counter = 0
                    while score_row_idx < len(rows):
                        scells = rows[score_row_idx].find_all('td')
                        parsing_logger.debug(f'AFTER COUPLE ROW: Table {table_idx}, Round {round_name}, Row {debug_row_counter}, Cell count: {len(scells)}, Content: {[cell.get_text(" ", strip=True) for cell in scells]}')
                        debug_row_counter += 1
                        score_row_idx += 1
                    # Score rows: after couple_row, look for rows where first cell contains multiple judge codes/names (space-separated or multiple words)
                    score_row_idx = idx + lookahead + 1
                    dance_rows = []
                    while score_row_idx < len(rows):
                        scells = rows[score_row_idx].find_all('td')
                        if len(scells) == len(couples) + 1:
                            first_cell_text = scells[0].get_text(" ", strip=True)
                            # Heuristic: if first cell contains more than 2 words, treat as judge codes/names
                            if len(first_cell_text.split()) >= 2:
                                dance_rows.append((score_row_idx, rows[score_row_idx]))
                                score_row_idx += 1
                                continue
                        break
                    num_dances = len(dance_rows)
                    parsing_logger.debug(f'Number of score rows (dances) found for round {round_name} in table {table_idx}: {num_dances}')
                    if num_dances < 3 or num_dances > 5:
                        parsing_logger.warning(f'Unexpected number of dances (score rows): {num_dances} in round {round_name} in table {table_idx}')
                    # Infer dances
                    ballroom = ['Slow Waltz', 'Tango', 'Viennese Waltz', 'Slow Foxtrott', 'Quick Step']
                    latin = ['Samba', 'Cha cha cha', 'Rumba', 'Paso Doble', 'Jive']
                    title = soup.title.get_text() if soup.title else ''
                    if 'Standard' in title or 'Ballroom' in title:
                        dances = ballroom[:num_dances]
                    else:
                        dances = latin[:num_dances]
                    parsing_logger.debug(f'Dances: {dances}')
                    # Extract judge codes from the first cell of each dance row (split by space)
                    for dance_idx, (drow_idx, drow) in enumerate(dance_rows):
                        dcells = drow.find_all('td')
                        judge_codes = [code.strip() for code in dcells[0].get_text(" ", strip=True).split() if code.strip()]
                        for couple_idx, cell in enumerate(dcells[1:]):
                            if couple_idx >= len(couples):
                                continue
                            # Split the cell by space to get scores for each judge
                            votes = [v.strip() for v in cell.get_text(" ", strip=True).split() if v.strip()]
                            if len(votes) != len(judge_codes):
                                continue
                            for judge_idx, vote in enumerate(votes):
                                code = judge_codes[judge_idx]
                                voted = bool(vote and vote.strip() and vote.strip() != '0')
                                entry = {
                                    'round': round_name,
                                    'number': couples[couple_idx],
                                    'judge_code': code,
                                    'dance': dances[dance_idx],
                                    'voted': voted
                                }
                                scores.append(entry)
                    idx = score_row_idx
                else:
                    idx += 1
        parsing_logger.debug(f'extract_scores: END, total scores={len(scores)}')
        return scores

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