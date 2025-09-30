import logging
from dancing_datacollection.parsing_utils import get_soup, extract_event_name_from_soup
from dancing_datacollection.parsing.erg import extract_participants_from_erg
from dancing_datacollection.parsing.ergwert import extract_participants_from_ergwert
from dancing_datacollection.parsing.tabges import extract_participants_from_tabges
from dancing_datacollection.parsing.wert_er import extract_participants_from_wert_er

logger = logging.getLogger(__name__)


def extract_participants_and_event_name(html: str, filename: str):
    """
    Extracts participants and the event name from HTML content.

    This function acts as a dispatcher, calling the appropriate
    participant extraction function based on the provided filename.
    It also extracts the event name from the HTML's <title> tag.

    Args:
        html: The HTML content to parse.
        filename: The name of the file from which the HTML was sourced.

    Returns:
        A tuple containing a list of participants and the event name.
    """
    soup = get_soup(html)
    event_name = extract_event_name_from_soup(soup)
    participants = []

    if filename.endswith("erg.htm"):
        participants = extract_participants_from_erg(soup)
    elif filename.endswith("ergwert.htm"):
        participants = extract_participants_from_ergwert(soup)
    elif filename.endswith("tabges.htm"):
        participants = extract_participants_from_tabges(soup)
    elif filename.endswith("wert_er.htm"):
        participants = extract_participants_from_wert_er(soup)
    else:
        logger.warning(f"Unknown filename for participant extraction: {filename}")

    return participants, event_name