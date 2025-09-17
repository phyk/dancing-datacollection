import os
import pytest
from dancing_datacollection.parsing_topturnier import TopTurnierParser
from dancing_datacollection.data_defs.score import FinalRoundScore, Score

TEST_DIR = os.path.dirname(__file__)


def ground_truth_ergwert_scores_51():
    # Returns (final_round_scores, all_other_scores)
    # All scores for all couples and all rounds, ordered by round (descending), number, dance, judge
    final_round_scores = [
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="AT", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="AX", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="BW", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=610, score=3, round_number=4, judge_code="CJ", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="EK", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="AT", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="AX", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="BW", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=610, score=3, round_number=4, judge_code="CJ", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="EK", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="AT", dance_name="Tango"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="AX", dance_name="Tango"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="BW", dance_name="Tango"
        ),
        FinalRoundScore(
            number=610, score=4, round_number=4, judge_code="CJ", dance_name="Tango"
        ),
        FinalRoundScore(
            number=610, score=1, round_number=4, judge_code="EK", dance_name="Tango"
        ),
        FinalRoundScore(
            number=616, score=2, round_number=4, judge_code="AT", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=616, score=2, round_number=4, judge_code="AX", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=616, score=2, round_number=4, judge_code="BW", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=616, score=1, round_number=4, judge_code="CJ", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=616, score=5, round_number=4, judge_code="EK", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=616, score=3, round_number=4, judge_code="AT", dance_name="Tango"
        ),
        FinalRoundScore(
            number=616, score=3, round_number=4, judge_code="AX", dance_name="Tango"
        ),
        FinalRoundScore(
            number=616, score=2, round_number=4, judge_code="BW", dance_name="Tango"
        ),
        FinalRoundScore(
            number=616, score=5, round_number=4, judge_code="CJ", dance_name="Tango"
        ),
        FinalRoundScore(
            number=616, score=3, round_number=4, judge_code="EK", dance_name="Tango"
        ),
        FinalRoundScore(
            number=616, score=3, round_number=4, judge_code="AT", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=616, score=4, round_number=4, judge_code="AX", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=616, score=2, round_number=4, judge_code="BW", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=616, score=4, round_number=4, judge_code="CJ", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=616, score=4, round_number=4, judge_code="EK", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=617, score=4, round_number=4, judge_code="AT", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=617, score=3, round_number=4, judge_code="AX", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=617, score=3, round_number=4, judge_code="BW", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=617, score=5, round_number=4, judge_code="CJ", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=617, score=2, round_number=4, judge_code="EK", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=617, score=5, round_number=4, judge_code="AT", dance_name="Tango"
        ),
        FinalRoundScore(
            number=617, score=6, round_number=4, judge_code="AX", dance_name="Tango"
        ),
        FinalRoundScore(
            number=617, score=3, round_number=4, judge_code="BW", dance_name="Tango"
        ),
        FinalRoundScore(
            number=617, score=1, round_number=4, judge_code="CJ", dance_name="Tango"
        ),
        FinalRoundScore(
            number=617, score=6, round_number=4, judge_code="EK", dance_name="Tango"
        ),
        FinalRoundScore(
            number=617, score=5, round_number=4, judge_code="AT", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=617, score=3, round_number=4, judge_code="AX", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=617, score=3, round_number=4, judge_code="BW", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=617, score=2, round_number=4, judge_code="CJ", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=617, score=5, round_number=4, judge_code="EK", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=611, score=6, round_number=4, judge_code="AT", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=611, score=4, round_number=4, judge_code="AX", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=611, score=6, round_number=4, judge_code="BW", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=611, score=4, round_number=4, judge_code="CJ", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=611, score=3, round_number=4, judge_code="EK", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=611, score=6, round_number=4, judge_code="AT", dance_name="Tango"
        ),
        FinalRoundScore(
            number=611, score=2, round_number=4, judge_code="AX", dance_name="Tango"
        ),
        FinalRoundScore(
            number=611, score=6, round_number=4, judge_code="BW", dance_name="Tango"
        ),
        FinalRoundScore(
            number=611, score=3, round_number=4, judge_code="CJ", dance_name="Tango"
        ),
        FinalRoundScore(
            number=611, score=2, round_number=4, judge_code="EK", dance_name="Tango"
        ),
        FinalRoundScore(
            number=611, score=6, round_number=4, judge_code="AT", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=611, score=2, round_number=4, judge_code="AX", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=611, score=4, round_number=4, judge_code="BW", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=611, score=5, round_number=4, judge_code="CJ", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=611, score=2, round_number=4, judge_code="EK", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=619, score=3, round_number=4, judge_code="AT", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=619, score=5, round_number=4, judge_code="AX", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=619, score=4, round_number=4, judge_code="BW", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=619, score=2, round_number=4, judge_code="CJ", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=619, score=6, round_number=4, judge_code="EK", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=619, score=2, round_number=4, judge_code="AT", dance_name="Tango"
        ),
        FinalRoundScore(
            number=619, score=4, round_number=4, judge_code="AX", dance_name="Tango"
        ),
        FinalRoundScore(
            number=619, score=4, round_number=4, judge_code="BW", dance_name="Tango"
        ),
        FinalRoundScore(
            number=619, score=2, round_number=4, judge_code="CJ", dance_name="Tango"
        ),
        FinalRoundScore(
            number=619, score=5, round_number=4, judge_code="EK", dance_name="Tango"
        ),
        FinalRoundScore(
            number=619, score=2, round_number=4, judge_code="AT", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=619, score=6, round_number=4, judge_code="AX", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=619, score=5, round_number=4, judge_code="BW", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=619, score=1, round_number=4, judge_code="CJ", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=619, score=6, round_number=4, judge_code="EK", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=615, score=5, round_number=4, judge_code="AT", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=615, score=6, round_number=4, judge_code="AX", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=615, score=5, round_number=4, judge_code="BW", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=615, score=6, round_number=4, judge_code="CJ", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=615, score=4, round_number=4, judge_code="EK", dance_name="SlowWaltz"
        ),
        FinalRoundScore(
            number=615, score=4, round_number=4, judge_code="AT", dance_name="Tango"
        ),
        FinalRoundScore(
            number=615, score=5, round_number=4, judge_code="AX", dance_name="Tango"
        ),
        FinalRoundScore(
            number=615, score=5, round_number=4, judge_code="BW", dance_name="Tango"
        ),
        FinalRoundScore(
            number=615, score=6, round_number=4, judge_code="CJ", dance_name="Tango"
        ),
        FinalRoundScore(
            number=615, score=4, round_number=4, judge_code="EK", dance_name="Tango"
        ),
        FinalRoundScore(
            number=615, score=4, round_number=4, judge_code="AT", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=615, score=5, round_number=4, judge_code="AX", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=615, score=6, round_number=4, judge_code="BW", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=615, score=6, round_number=4, judge_code="CJ", dance_name="Quickstep"
        ),
        FinalRoundScore(
            number=615, score=3, round_number=4, judge_code="EK", dance_name="Quickstep"
        ),
    ]
    all_other_scores = [
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=600,
            score=False,
            round_number=1,
            judge_code="AT",
            dance_name="Tango",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="AX",
            dance_name="Tango",
        ),
        Score(
            number=600,
            score=False,
            round_number=1,
            judge_code="BW",
            dance_name="Tango",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="CJ",
            dance_name="Tango",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="EK",
            dance_name="Tango",
        ),
        Score(
            number=600,
            score=False,
            round_number=1,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=600,
            score=True,
            round_number=1,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=602,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=602,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=602,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=602,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=602,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=602,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=602, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=602, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=602, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=602, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=603,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=603, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=603, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=603, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=603, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
        Score(
            number=604,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=604,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=604,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=604,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=604,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=604, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=604, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
        Score(
            number=604, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=606,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=606,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=606,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=606,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=606,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=606,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=606, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=606, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=607,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=607,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=607,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=607,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=607,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=607,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=607,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=607, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=607, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
        Score(
            number=609,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=609,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=609,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=609,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=609,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=609,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=609,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=609,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=609, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=609, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=609, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=609, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
        Score(
            number=609, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=610,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=610, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=610, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=610, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=610, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
        Score(
            number=610, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=611,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=611,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=611,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=611,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=611,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=611,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=611, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=611, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=612,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=612,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=612,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=612,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=612,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=612,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=612,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=612,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=612, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=612, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=613,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=613, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=613, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=613, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=613, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=615,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=615,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=615,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=615,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=615,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=615, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=615, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=616,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=616,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=616,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=616,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=616,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=616,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=616,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=616,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=616, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=616, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=616, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=616, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=617,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=617,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=617,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=617,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=617,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=617,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=617, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=617, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=617, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=617, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
        Score(
            number=617, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=619,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="Quickstep",
        ),
        Score(
            number=619,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=619,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=619,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=619,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=619,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=619,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=619, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=619, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=619, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
        Score(
            number=620,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=620,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=620,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=620,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=620,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=620,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=620, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=620, score=True, round_number=3, judge_code="BW", dance_name="Tango"
        ),
        Score(
            number=620, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
        Score(
            number=621,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=621,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="Quickstep",
        ),
        Score(
            number=621,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="Quickstep",
        ),
        Score(
            number=621,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=621,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=621,
            score=True,
            round_number=3,
            judge_code="EK",
            dance_name="SlowWaltz",
        ),
        Score(
            number=621, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=621, score=True, round_number=3, judge_code="AX", dance_name="Tango"
        ),
        Score(
            number=621, score=True, round_number=3, judge_code="EK", dance_name="Tango"
        ),
        Score(
            number=624,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="Quickstep",
        ),
        Score(
            number=624,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="Quickstep",
        ),
        Score(
            number=624,
            score=True,
            round_number=3,
            judge_code="AT",
            dance_name="SlowWaltz",
        ),
        Score(
            number=624,
            score=True,
            round_number=3,
            judge_code="AX",
            dance_name="SlowWaltz",
        ),
        Score(
            number=624,
            score=True,
            round_number=3,
            judge_code="BW",
            dance_name="SlowWaltz",
        ),
        Score(
            number=624,
            score=True,
            round_number=3,
            judge_code="CJ",
            dance_name="SlowWaltz",
        ),
        Score(
            number=624, score=True, round_number=3, judge_code="AT", dance_name="Tango"
        ),
        Score(
            number=624, score=True, round_number=3, judge_code="CJ", dance_name="Tango"
        ),
    ]
    return final_round_scores, all_other_scores


@pytest.mark.parametrize(
    "sample_dir,ground_truth_func",
    [
        ("51-1105_ot_hgr2dstd", ground_truth_ergwert_scores_51),
    ],
)
def test_extract_scores_from_ergwert(sample_dir, ground_truth_func):
    parser = TopTurnierParser()
    ergwert_path = os.path.join(TEST_DIR, sample_dir, "ergwert.htm")
    if not os.path.exists(ergwert_path):
        pytest.skip(f"Missing {ergwert_path}")
    with open(ergwert_path, "r", encoding="utf-8") as f:
        html = f.read()
    scores = parser.extract_scores(html, filename="ergwert.htm")
    # Only compare FinalRoundScore objects
    final_round_scores, all_other_scores = ground_truth_func()
    extracted_final = set(s for s in scores if type(s).__name__ == "FinalRoundScore")
    ground_truth_final = set(final_round_scores)
    assert ground_truth_final == extracted_final, (
        f"Missing: {ground_truth_final - extracted_final}\n"
        f"Extra: {extracted_final - ground_truth_final}\n"
    )


@pytest.mark.parametrize(
    "sample_dir",
    [
        "51-1105_ot_hgr2dstd",
    ],
)
def test_parse_tabges_all_with_pandas(sample_dir):
    pd = pytest.importorskip("pandas")
    parser = TopTurnierParser()
    tabges_path = os.path.join(TEST_DIR, sample_dir, "tabges.htm")
    if not os.path.exists(tabges_path):
        pytest.skip(f"Missing {tabges_path}")
    with open(tabges_path, "r", encoding="utf-8") as f:
        html = f.read()
    tables = parser.parse_tabges_all(html)
    assert isinstance(tables, list)
    assert tables, "No tables parsed from tabges.htm"
    assert all(isinstance(df, pd.DataFrame) for df in tables)
    # Smoke-check: the first table should include the judge codes row text and some numbers
    first_df = tables[0]
    # Ensure at least some numeric entries and the words 'Wertungsrichter'/'Startnummer' are present
    flat_values = first_df.astype(str).values.ravel().tolist()
    assert any("Wertungsrichter" in v for v in flat_values)
    assert any("Startnummer" in v for v in flat_values)
    assert any(v.isdigit() and len(v) in (1, 2, 3) for v in flat_values)
