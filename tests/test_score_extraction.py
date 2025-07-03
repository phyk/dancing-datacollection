import os
import pytest
from dancing_datacollection.parsing_topturnier import TopTurnierParser
from dancing_datacollection.data_defs.score import FinalRoundScore, Score

TEST_DIR = os.path.dirname(__file__)

def ground_truth_ergwert_scores_51():
    # Auto-generated ground truth from current ergwert.htm extraction, with corrected judge codes and dance names
    return [
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='AT', dance_name='SlowWaltz'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='AX', dance_name='SlowWaltz'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='BW', dance_name='SlowWaltz'),
        FinalRoundScore(number=610, score=3, round_name='Endrunde', round_number=4, judge_code='CJ', dance_name='SlowWaltz'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='EK', dance_name='SlowWaltz'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='AT', dance_name='Tango'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='AX', dance_name='Tango'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='BW', dance_name='Tango'),
        FinalRoundScore(number=610, score=4, round_name='Endrunde', round_number=4, judge_code='CJ', dance_name='Tango'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='EK', dance_name='Tango'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='AT', dance_name='Quickstep'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='AX', dance_name='Quickstep'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='BW', dance_name='Quickstep'),
        FinalRoundScore(number=610, score=3, round_name='Endrunde', round_number=4, judge_code='CJ', dance_name='Quickstep'),
        FinalRoundScore(number=610, score=1, round_name='Endrunde', round_number=4, judge_code='EK', dance_name='Quickstep'),

        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='AT', dance_name='SlowWaltz'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='AX', dance_name='SlowWaltz'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='BW', dance_name='SlowWaltz'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='CJ', dance_name='SlowWaltz'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='EK', dance_name='SlowWaltz'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='AT', dance_name='Tango'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='AX', dance_name='Tango'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='BW', dance_name='Tango'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='CJ', dance_name='Tango'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='EK', dance_name='Tango'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='AT', dance_name='Quickstep'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='AX', dance_name='Quickstep'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='BW', dance_name='Quickstep'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='CJ', dance_name='Quickstep'),
        Score(number=610, score=True, round_name='2. Zwischenrunde', round_number=3, judge_code='EK', dance_name='Quickstep'),
        
    ]

@pytest.mark.parametrize('sample_dir,ground_truth_func', [
    ('51-1105_ot_hgr2dstd', ground_truth_ergwert_scores_51),
])
def test_extract_scores_from_ergwert(sample_dir, ground_truth_func):
    parser = TopTurnierParser()
    ergwert_path = os.path.join(TEST_DIR, sample_dir, 'ergwert.htm')
    if not os.path.exists(ergwert_path):
        pytest.skip(f"Missing {ergwert_path}")
    with open(ergwert_path, 'r', encoding='utf-8') as f:
        html = f.read()
    scores = parser.extract_scores(html, filename='ergwert.htm')
    assert isinstance(scores, list)
    assert scores, "No scores extracted"
    for gt in ground_truth_func():
        assert any(gt.partial_match(s) for s in scores), f"Missing score entry (partial match): {gt}"
    for entry in scores:
        assert isinstance(entry, (FinalRoundScore, Score)) 