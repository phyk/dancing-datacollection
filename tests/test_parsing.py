import os
from dancing_datacollection.parsing_topturnier import TopTurnierParser

TEST_DIR = os.path.dirname(__file__)
SAMPLE_DIRS = [
    '51-1105_ot_hgr2dstd',
    '52-1105_ot_hgr2cstd',
    '53-1105_ot_hgr2bstd',
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
            if fname.endswith('.htm'):
                fpath = os.path.join(dir_path, fname)
                with open(fpath, 'r', encoding='utf-8') as f:
                    html = f.read()
                participants, _ = parser.extract_participants(html)
                if participants:
                    print(f"  Participants found in {fname}: {len(participants)}")
                    all_participants.extend(participants)
        # Deduplicate by (number, names, club)
        seen = set()
        unique_participants = []
        for p in all_participants:
            key = (p.get('number'), p.get('names'), p.get('club'))
            if key not in seen:
                seen.add(key)
                unique_participants.append(p)
        print(f"Summary for {sample_dir}: {len(unique_participants)} unique participants found.")
        # Test judges and committee extraction from deck.htm
        deck_path = os.path.join(dir_path, 'deck.htm')
        if os.path.exists(deck_path):
            with open(deck_path, 'r', encoding='utf-8') as f:
                deck_html = f.read()
            judges = parser.extract_judges(deck_html)
            print(f"  Judges found: {len(judges)}")
            committee = parser.extract_committee(deck_html)
            print(f"  Committee entries found: {len(committee)}")
        # Test scores extraction from tabges.htm
        tabges_path = os.path.join(dir_path, 'tabges.htm')
        if os.path.exists(tabges_path):
            with open(tabges_path, 'r', encoding='utf-8') as f:
                tabges_html = f.read()
            scores = parser.extract_scores(tabges_html)
            print(f"  Score entries found: {len(scores)}")
            # Print a sample score entry
            if scores:
                print(f"    Sample score entry: {scores[0]}")

def test_extract_final_scoring():
    from dancing_datacollection.parsing_topturnier import TopTurnierParser
    parser = TopTurnierParser()
    with open('tests/51-1105_ot_hgr2dstd/ergwert.htm', encoding='utf-8') as f:
        html = f.read()
    final_scores = parser.extract_final_scoring(html)
    assert isinstance(final_scores, list)
    assert final_scores, "No final scores extracted"
    for entry in final_scores:
        assert 'placement' in entry
        assert 'names' in entry
        assert 'total' in entry
        assert entry['placement']
        assert entry['names']
        assert entry['total']

if __name__ == '__main__':
    main() 