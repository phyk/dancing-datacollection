import os
from typing import Callable, List

import pytest

from dancing_datacollection.data_defs.committee import CommitteeMember
from dancing_datacollection.parsing.deck import extract_committee_from_deck
from dancing_datacollection.parsing.parsing_utils import get_soup


def get_html(path: str) -> str:
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def true_committee_51() -> List[CommitteeMember]:
    return [
        CommitteeMember(role="organizer", name="Hessischer Tanzsportverband", club=""),
        CommitteeMember(role="host", name="Hessischer Tanzsportverband", club=""),
        CommitteeMember(
            role="chairperson", name="Jungbluth, Kai", club="Tanz-Sport-Club Fischbach"
        ),
        CommitteeMember(
            role="committee_member",
            name="Bittighofer, Mechthild",
            club="Tanz-Freunde Fulda",
        ),
        CommitteeMember(role="protocol", name="tanzt, EDV-Team Hessen", club=""),
    ]


def true_committee_52() -> List[CommitteeMember]:
    return [
        CommitteeMember(role="organizer", name="Hessischer Tanzsportverband", club=""),
        CommitteeMember(role="host", name="Hessischer Tanzsportverband", club=""),
        CommitteeMember(
            role="chairperson", name="Jungbluth, Kai", club="Tanz-Sport-Club Fischbach"
        ),
        CommitteeMember(
            role="committee_member",
            name="Knigge, Jens",
            club="TSC Groß-Gerau d. TV 1846",
        ),
        CommitteeMember(role="protocol", name="tanzt, EDV-Team Hessen", club=""),
    ]


def true_committee_53() -> List[CommitteeMember]:
    return [
        CommitteeMember(role="organizer", name="Hessischer Tanzsportverband", club=""),
        CommitteeMember(role="host", name="Hessischer Tanzsportverband", club=""),
        CommitteeMember(
            role="chairperson", name="Jungbluth, Kai", club="Tanz-Sport-Club Fischbach"
        ),
        CommitteeMember(
            role="committee_member",
            name="Rahaus, Markus",
            club="Schwarz-Rot-Club Wetzlar",
        ),
        CommitteeMember(role="protocol", name="tanzt, EDV-Team Hessen", club=""),
    ]


@pytest.mark.parametrize(
    ("sample_dir", "true_committee_func"),
    [
        ("51-1105_ot_hgr2dstd", true_committee_51),
        ("52-1105_ot_hgr2cstd", true_committee_52),
        ("53-1105_ot_hgr2bstd", true_committee_53),
    ],
)
def test_extract_committee(
    sample_dir: str,
    true_committee_func: Callable[[], List[CommitteeMember]],
    test_dir: str,
) -> None:
    deck_path = os.path.join(test_dir, sample_dir, "deck.htm")
    if not os.path.exists(deck_path):
        pytest.skip(f"Missing {deck_path}")
    html = get_html(deck_path)
    soup = get_soup(html)
    committee = extract_committee_from_deck(soup)
    expected = true_committee_func()
    assert committee == expected, (
        f"Extracted committee does not match ground truth for {sample_dir}.\n"
        f"Extracted: {committee}\nExpected: {expected}"
    )