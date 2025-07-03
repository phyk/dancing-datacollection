import os
from dancing_datacollection.parsing_utils import setup_logging
from dancing_datacollection.parsing_topturnier import TopTurnierParser
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


def main():
    parser = TopTurnierParser()
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
                participants, _ = parser.extract_participants(html)
                if participants:
                    print(f"  Participants found in {fname}: {len(participants)}")
                    all_participants.extend(participants)
        # Deduplicate by (number, names, club)
        seen = set()
        unique_participants = []
        for p in all_participants:
            key = (p.get("number"), p.get("names"), p.get("club"))
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
            judges = parser.extract_judges(deck_html)
            print(f"  Judges found: {len(judges)}")
            committee = parser.extract_committee(deck_html)
            print(f"  Committee entries found: {len(committee)}")
        # Test scores extraction from tabges.htm
        tabges_path = os.path.join(dir_path, "tabges.htm")
        tabges_couples = set()
        if os.path.exists(tabges_path):
            with open(tabges_path, "r", encoding="utf-8") as f:
                tabges_html = f.read()
            scores = parser.extract_scores(tabges_html)
            print(f"  Score entries found: {len(scores)}")
            # Extract unique couple numbers from scores
            tabges_couples = set(
                s["number"] for s in scores if "number" in s and s["number"] is not None
            )
            print(f"  Unique couple numbers in tabges.htm: {len(tabges_couples)}")
            # Print a sample score entry
            if scores:
                print(f"    Sample score entry: {scores[0]}")
        # Test final scoring extraction from ergwert.htm
        ergwert_path = os.path.join(dir_path, "ergwert.htm")
        ergwert_couples = set()
        if os.path.exists(ergwert_path):
            with open(ergwert_path, "r", encoding="utf-8") as f:
                ergwert_html = f.read()
            final_scores = parser.extract_final_scoring(ergwert_html)
            ergwert_couples = set(
                f["number"]
                for f in final_scores
                if "number" in f and f["number"] is not None
            )
            print(f"  Unique couple numbers in ergwert.htm: {len(ergwert_couples)}")
        # Compare numbers
        participant_numbers = set(
            p["number"]
            for p in unique_participants
            if "number" in p and p["number"] is not None
        )
        print(f"  Unique couple numbers in participants: {len(participant_numbers)}")
        if tabges_couples and participant_numbers != tabges_couples:
            print(
                f"WARNING: Mismatch between participants and tabges.htm couples! Participants: {len(participant_numbers)}, Tabges: {len(tabges_couples)}"
            )
        if ergwert_couples and participant_numbers != ergwert_couples:
            print(
                f"WARNING: Mismatch between participants and ergwert.htm couples! Participants: {len(participant_numbers)}, Ergwert: {len(ergwert_couples)}"
            )


def test_extract_final_scoring():
    from dancing_datacollection.parsing_topturnier import TopTurnierParser

    parser = TopTurnierParser()
    with open("tests/51-1105_ot_hgr2dstd/ergwert.htm", encoding="utf-8") as f:
        html = f.read()
    final_scores = parser.extract_final_scoring(html)
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
