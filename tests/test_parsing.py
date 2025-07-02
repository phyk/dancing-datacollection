import os
from dancing_datacollection.__main__ import CompetitionParser

TEST_DIR = os.path.dirname(__file__)
SAMPLES = [
    '51-1105_ot_hgr2dstd/index.htm',
    '52-1105_ot_hgr2cstd/index.htm',
    '53-1105_ot_hgr2bstd/index.htm',
]

def main():
    for sample in SAMPLES:
        path = os.path.join(TEST_DIR, sample)
        if not os.path.exists(path):
            print(f"File not found: {path}")
            continue
        with open(path, 'r', encoding='utf-8') as f:
            html = f.read()
        parser = CompetitionParser(html)
        data = parser.extract()
        print(f"\nParsed participants from {sample}:")
        for participant in data['participants']:
            print(participant)
        if not data['participants']:
            print("  No participants found.")

if __name__ == '__main__':
    main() 