from dancing_datacollection.data_defs.committee import CommitteeMember
from dancing_datacollection.parsing_utils import extract_name_and_club_from_spans
import logging


def extract_committee_from_deck(soup):
    logger = logging.getLogger("parsing_debug")
    table = soup.find("table", class_="tab1")
    logger.debug(f"Found table: {bool(table)}")
    if not table:
        return []
    roles = [
        ("Veranstalter:", "organizer"),
        ("Ausrichter:", "host"),
        ("Turnierleiter:", "chairperson"),
        ("Beisitzer:", "committee_member"),
        ("Protokoll:", "protocol"),
    ]
    committee = []
    for row_idx, row in enumerate(table.find_all("tr")):
        cells = row.find_all("td")
        logger.debug(f'Row {row_idx}: {[c.get_text(" ", strip=True) for c in cells]}')
        if len(cells) < 2:
            continue
        label = cells[0].get_text(strip=True)
        for role_label, role_key in roles:
            if label == role_label:
                value_cell = cells[1]
                name, club = extract_name_and_club_from_spans(value_cell)
                logger.debug(
                    f'  Committee: role={role_key}, name={name}, club={club}, raw_value={value_cell.get_text(" ", strip=True)}'
                )
                committee.append(CommitteeMember(role=role_key, name=name, club=club))
    logger.debug(f"extract_committee_from_deck: END, total committee={len(committee)}")
    return committee
