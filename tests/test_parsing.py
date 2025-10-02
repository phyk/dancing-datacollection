import logging
import os
from typing import List

from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.parsing.ergwert import (
    extract_final_scoring,
    extract_participants_from_ergwert,
    extract_scores_from_ergwert,
)
from dancing_datacollection.parsing.parsing_utils import get_soup, setup_logging
from dancing_datacollection.parsing.tabges import extract_participants_from_tabges
from dancing_datacollection.parsing.wert_er import extract_participants_from_wert_er

# Set up logging before anything else
setup_logging()
logging.basicConfig(level=logging.INFO, format="%(message)s")


# Clean logs directory before running tests
LOG_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "logs")
if os.path.exists(LOG_DIR):
    for fname in os.listdir(LOG_DIR):
        fpath = os.path.join(LOG_DIR, fname)
        if os.path.isfile(fpath):
            os.remove(fpath)

TEST_DIR = os.path.dirname(__file__)
SAMPLE_DIRS = [
    "51-1105_ot_hgr2dstd",
    "52-1105_ot_hgr2cstd",
    "53-1105_ot_hgr2bstd",
]


def extract_participants(html: str, filename: str) -> List[Participant]:
    """Helper to dispatch participant extraction based on filename."""
    soup = get_soup(html)
    if filename.endswith("ergwert.htm"):
        return extract_participants_from_ergwert(soup)
    if filename.endswith("tabges.htm"):
        return extract_participants_from_tabges(soup)
    if filename.endswith("wert_er.htm"):
        return extract_participants_from_wert_er(soup)
    return []


def main() -> None:
    for sample_dir in SAMPLE_DIRS:
        dir_path = os.path.join(TEST_DIR, sample_dir)
        if not os.path.isdir(dir_path):
            logging.error("Directory not found: %s", dir_path)
            continue
        logging.info("\nExploring files in %s:", sample_dir)
        # Test participants extraction
        all_participants: List[Participant] = []
        for fname in os.listdir(dir_path):
            if fname.endswith(".htm"):
                fpath = os.path.join(dir_path, fname)
                with open(fpath, "r", encoding="utf-8") as f:
                    html = f.read()
                participants = extract_participants(html, fname)
                if participants:
                    logging.info("  Participants found in %s: %d", fname, len(participants))
                    all_participants.extend(participants)
        # Deduplicate by (number, name_one, name_two, club)
        seen = set()
        unique_participants: List[Participant] = []
        for p in all_participants:
            key = (p.number, p.name_one, p.name_two, p.club)
            if key not in seen:
                seen.add(key)
                unique_participants.append(p)
        logging.info(
            "Summary for %s: %d unique participants found.",
            sample_dir,
            len(unique_participants),
        )
        # Test scores and final scoring extraction from ergwert.htm
        ergwert_path = os.path.join(dir_path, "ergwert.htm")
        ergwert_couples = set()
        if os.path.exists(ergwert_path):
            with open(ergwert_path, "r", encoding="utf-8") as f:
                ergwert_html = f.read()
            soup = get_soup(ergwert_html)
            scores = extract_scores_from_ergwert(soup)
            logging.info("  Score entries found: %d", len(scores))
            final_scores = extract_final_scoring(ergwert_html)
            logging.info("  Final scoring entries found: %d", len(final_scores))
            ergwert_couples = {f.number for f in final_scores if f.number is not None}
            logging.info("  Unique couple numbers in ergwert.htm: %d", len(ergwert_couples))

        # Compare numbers
        participant_numbers = {p.number for p in unique_participants if p.number is not None}
        logging.info("  Unique couple numbers in participants: %d", len(participant_numbers))
        if ergwert_couples and participant_numbers != ergwert_couples:
            logging.warning(
                "WARNING: Mismatch between participants and ergwert.htm couples! Participants: %d, Ergwert: %d",
                len(participant_numbers),
                len(ergwert_couples),
            )


def test_extract_final_scoring() -> None:
    with open("tests/51-1105_ot_hgr2dstd/ergwert.htm", encoding="utf-8") as f:
        html = f.read()
    final_scores = extract_final_scoring(html)
    assert isinstance(final_scores, list)
    assert final_scores, "No final scores extracted"
    for entry in final_scores:
        assert entry.placement is not None
        assert entry.names is not None
        assert entry.total is not None


if __name__ == "__main__":
    main()
