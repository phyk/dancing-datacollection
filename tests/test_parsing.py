import os
from dancing_datacollection.parsing_utils import setup_logging, get_soup
from dancing_datacollection.parsing.deck import extract_judges_from_deck
from dancing_datacollection.parsing.committee import extract_committee_from_deck
from dancing_datacollection.parsing.ergwert import (
    extract_scores_from_ergwert,
    extract_final_scoring,
    extract_participants_from_ergwert,
)
from dancing_datacollection.parsing.erg import extract_participants_from_erg
from dancing_datacollection.parsing.tabges import extract_participants_from_tabges
from dancing_datacollection.parsing.wert_er import extract_participants_from_wert_er
from dancing_datacollection.data_defs.participant import Participant
import pytest

# Set up logging before anything else
setup_logging()

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


def extract_participants(html, filename):
    """Helper to dispatch participant extraction based on filename."""
    soup = get_soup(html)
    if filename.endswith("erg.htm"):
        return extract_participants_from_erg(soup)
    elif filename.endswith("ergwert.htm"):
        return extract_participants_from_ergwert(soup)
    elif filename.endswith("tabges.htm"):
        return extract_participants_from_tabges(soup)
    elif filename.endswith("wert_er.htm"):
        return extract_participants_from_wert_er(soup)
    return []


def main():
    for sample_dir in SAMPLE_DIRS:
        dir_path = os.path.join(TEST_DIR, sample_dir)
        if not os.path.isdir(dir_path):
            print(f"Directory not found: {dir_path}")
            continue
        print(f"\nExploring files in {sample_dir}:")
        # Test participants extraction
        all_participants = []
        for fname in os.listdir(dir_path):
            if fname.endswith(".htm"):
                fpath = os.path.join(dir_path, fname)
                with open(fpath, "r", encoding="utf-8") as f:
                    html = f.read()
                participants = extract_participants(html, fname)
                if participants:
                    print(f"  Participants found in {fname}: {len(participants)}")
                    all_participants.extend(participants)
        # Deduplicate by (number, name_one, name_two, club)
        seen = set()
        unique_participants = []
        for p in all_participants:
            key = (p.number, p.name_one, p.name_two, p.club)
            if key not in seen:
                seen.add(key)
                unique_participants.append(p)
        print(
            f"Summary for {sample_dir}: {len(unique_participants)} unique participants found."
        )
        # Test judges and committee extraction from deck.htm
        deck_path = os.path.join(dir_path, "deck.htm")
        if os.path.exists(deck_path):
            with open(deck_path, "r", encoding="utf-8") as f:
                deck_html = f.read()
            soup = get_soup(deck_html)
            judges = extract_judges_from_deck(soup)
            print(f"  Judges found: {len(judges)}")
            committee = extract_committee_from_deck(soup)
            print(f"  Committee entries found: {len(committee)}")
        # Test scores and final scoring extraction from ergwert.htm
        ergwert_path = os.path.join(dir_path, "ergwert.htm")
        ergwert_couples = set()
        if os.path.exists(ergwert_path):
            with open(ergwert_path, "r", encoding="utf-8") as f:
                ergwert_html = f.read()
            soup = get_soup(ergwert_html)
            scores = extract_scores_from_ergwert(soup)
            print(f"  Score entries found: {len(scores)}")
            final_scores = extract_final_scoring(ergwert_html)
            print(f"  Final scoring entries found: {len(final_scores)}")
            ergwert_couples = set(
                f["number"]
                for f in final_scores
                if "number" in f and f["number"] is not None
            )
            print(f"  Unique couple numbers in ergwert.htm: {len(ergwert_couples)}")

        # Compare numbers
        participant_numbers = set(
            p.number for p in unique_participants if p.number is not None
        )
        print(f"  Unique couple numbers in participants: {len(participant_numbers)}")
        if ergwert_couples and participant_numbers != ergwert_couples:
            print(
                f"WARNING: Mismatch between participants and ergwert.htm couples! Participants: {len(participant_numbers)}, Ergwert: {len(ergwert_couples)}"
            )


def test_extract_final_scoring():
    with open("tests/51-1105_ot_hgr2dstd/ergwert.htm", encoding="utf-8") as f:
        html = f.read()
    final_scores = extract_final_scoring(html)
    assert isinstance(final_scores, list)
    assert final_scores, "No final scores extracted"
    for entry in final_scores:
        assert "placement" in entry
        assert "names" in entry
        assert "total" in entry
        assert entry["placement"]
        assert entry["names"]
        assert entry["total"]


if __name__ == "__main__":
    main()