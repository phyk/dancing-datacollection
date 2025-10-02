import os
from typing import Callable, List

import pytest

from dancing_datacollection.data_defs.committee import Committee, CommitteeMember
from dancing_datacollection.html_canonicalize import canonicalize_html
from dancing_datacollection.html_generate import generate_committee_html
from dancing_datacollection.parsing.deck import (
    extract_committee_from_deck,
    extract_committee_html_from_deck,
)
from dancing_datacollection.parsing.parsing_utils import get_soup


def get_html(path: str) -> str:
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def true_committee_51() -> Committee:
    return Committee(
        members=[
            CommitteeMember(
                role="organizer",
                first_name="Hessischer Tanzsportverband",
                last_name=None,
                club=None,
            ),
            CommitteeMember(
                role="host",
                first_name="Hessischer Tanzsportverband",
                last_name=None,
                club=None,
            ),
            CommitteeMember(
                role="chairperson",
                first_name="Kai",
                last_name="Jungbluth",
                club="Tanz-Sport-Club Fischbach",
            ),
            CommitteeMember(
                role="committee_member",
                first_name="Mechthild",
                last_name="Bittighofer",
                club="Tanz-Freunde Fulda",
            ),
            CommitteeMember(
                role="protocol",
                first_name="EDV-Team Hessen",
                last_name="tanzt",
                club=None,
            ),
        ]
    )


def true_committee_52() -> Committee:
    return Committee(
        members=[
            CommitteeMember(
                role="organizer",
                first_name="Hessischer Tanzsportverband",
                last_name=None,
                club=None,
            ),
            CommitteeMember(
                role="host",
                first_name="Hessischer Tanzsportverband",
                last_name=None,
                club=None,
            ),
            CommitteeMember(
                role="chairperson",
                first_name="Kai",
                last_name="Jungbluth",
                club="Tanz-Sport-Club Fischbach",
            ),
            CommitteeMember(
                role="committee_member",
                first_name="Jens",
                last_name="Knigge",
                club="TSC Groß-Gerau d. TV 1846",
            ),
            CommitteeMember(
                role="protocol",
                first_name="EDV-Team Hessen",
                last_name="tanzt",
                club=None,
            ),
        ]
    )


def true_committee_53() -> Committee:
    return Committee(
        members=[
            CommitteeMember(
                role="organizer",
                first_name="Hessischer Tanzsportverband",
                last_name=None,
                club=None,
            ),
            CommitteeMember(
                role="host",
                first_name="Hessischer Tanzsportverband",
                last_name=None,
                club=None,
            ),
            CommitteeMember(
                role="chairperson",
                first_name="Kai",
                last_name="Jungbluth",
                club="Tanz-Sport-Club Fischbach",
            ),
            CommitteeMember(
                role="committee_member",
                first_name="Markus",
                last_name="Rahaus",
                club="Schwarz-Rot-Club Wetzlar",
            ),
            CommitteeMember(
                role="protocol",
                first_name="EDV-Team Hessen",
                last_name="tanzt",
                club=None,
            ),
        ]
    )


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
    true_committee_func: Callable[[], Committee],
    test_dir: str,
):
    deck_path = os.path.join(test_dir, sample_dir, "deck.htm")
    if not os.path.exists(deck_path):
        pytest.skip(f"Missing {deck_path}")
    html = get_html(deck_path)
    soup = get_soup(html)
    committee = extract_committee_from_deck(soup)
    true_committee = true_committee_func()
    assert committee == true_committee.members

    # Test that the regenerated HTML matches the original HTML snippet
    regenerated_html = generate_committee_html(committee)
    original_committee_html = extract_committee_html_from_deck(soup)
    assert canonicalize_html(regenerated_html) == canonicalize_html(
        original_committee_html
    )