import os
from dancing_datacollection.parsing_topturnier import TopTurnierParser
from dancing_datacollection.data_defs.score import FinalRoundScore

TEST_DIR = os.path.dirname(__file__)
ergwert_path = os.path.join(TEST_DIR, "51-1105_ot_hgr2dstd", "ergwert.htm")

parser = TopTurnierParser()
with open(ergwert_path, "r", encoding="utf-8") as f:
    html = f.read()
scores = parser.extract_scores(html, filename="ergwert.htm")
final_scores = [s for s in scores if type(s).__name__ == "FinalRoundScore"]

for s in sorted(final_scores, key=lambda x: (x.number, x.dance_name, x.judge_code)):
    print(
        f"FinalRoundScore(number={s.number}, score={s.score}, round_number={s.round_number}, judge_code='{s.judge_code}', dance_name='{s.dance_name}')"
    )
