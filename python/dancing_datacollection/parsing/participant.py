import logging
from typing import List, Tuple

from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.competition import CompetitionInfo
from dancing_datacollection.parsing.erg import extract_participants_from_erg
from dancing_datacollection.parsing.ergwert import extract_participants_from_ergwert
from dancing_datacollection.parsing.parsing_utils import (
    extract_event_name_from_soup,
    get_soup,
    parse_competition_title,
)
from dancing_datacollection.parsing.tabges import extract_participants_from_tabges
from dancing_datacollection.parsing.wert_er import extract_participants_from_wert_er

logger = logging.getLogger(__name__)


def extract_competition_info_from_html(html: str) -> CompetitionInfo:
    """Extracts competition information from the HTML title."""
    soup = get_soup(html)
    title_tag = soup.find("title")
    title = title_tag.get_text(strip=True) if title_tag else ""
    return parse_competition_title(title)


def extract_participants_and_event_name(
    html: str, filename: str
) -> Tuple[List[Participant], str, CompetitionInfo]:
    """
    Extracts participants, the event name, and competition info from HTML content.

    This function acts as a dispatcher, calling the appropriate
    participant extraction function based on the provided filename.
    It also extracts the event name and structured competition info
    from the HTML's <title> tag.

    Args:
        html: The HTML content to parse.
        filename: The name of the file from which the HTML was sourced.

    Returns:
        A tuple containing:
        - A list of participants.
        - The sanitized event name.
        - A CompetitionInfo object.
    """
    soup = get_soup(html)
    info = extract_competition_info_from_html(html)
    event_name = extract_event_name_from_soup(soup)
    participants: List[Participant] = []

    if filename.endswith("erg.htm"):
        participants = extract_participants_from_erg(soup)
    elif filename.endswith("ergwert.htm"):
        participants = extract_participants_from_ergwert(soup)
    elif filename.endswith("tabges.htm"):
        participants = extract_participants_from_tabges(soup)
    elif filename.endswith("wert_er.htm"):
        participants = extract_participants_from_wert_er(soup)
    else:
        logger.warning("Unknown filename for participant extraction: %s", filename)

    return participants, event_name, info
