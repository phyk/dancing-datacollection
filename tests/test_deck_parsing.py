import os

from dancing_datacollection.data_defs.committee import CommitteeMember
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.html_canonicalize import canonicalize_html
from dancing_datacollection.html_generate import generate_deck_html
from dancing_datacollection.parsing.deck import (
    extract_committee_from_deck,
    extract_judges_from_deck,
)
from dancing_datacollection.parsing.parsing_utils import get_soup


def test_deck_parsing_and_regeneration():
    """
    Tests parsing of a deck.htm file, compares with true values,
    regenerates the HTML, and compares with the canonicalized original.
    """
    # Define the path to the test file
    test_file = os.path.join(
        os.path.dirname(__file__), "51-1105_ot_hgr2dstd", "deck.htm"
    )

    # Define the true values for comparison
    true_committee = [
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

    true_judges = [
        Judge(
            code="AT",
            first_name="Marcus",
            last_name="Bärschneider",
            club="TSC Blau-Gelb Hagen",
        ),
        Judge(
            code="AX",
            first_name="Robert",
            last_name="Block",
            club="Schwarz-Rot-Club Wetzlar",
        ),
        Judge(
            code="BW",
            first_name="Susanne",
            last_name="Kirchwehm",
            club="TSC Ostseebad Schönberg 1984",
        ),
        Judge(
            code="CJ",
            first_name="Erich",
            last_name="Mäser",
            club="TSC Rot-Gold Büdingen",
        ),
        Judge(
            code="EK",
            first_name="Peter",
            last_name="Landauer",
            club="Tanzsportgemeinschaft Bavaria, Augsburg",
        ),
    ]

    # 1. Parse the file and extract all information
    with open(test_file, "r", encoding="utf-8") as f:
        html_content = f.read()

    soup = get_soup(html_content)
    title = str((soup.title and soup.title.string) or "deck")
    extracted_committee = extract_committee_from_deck(soup)
    extracted_judges = extract_judges_from_deck(soup)

    # 2. Compare the information against the true values
    assert extracted_committee == true_committee
    assert extracted_judges == true_judges

    # 3. Regenerate the html from the information
    regenerated_html = generate_deck_html(extracted_judges, extracted_committee, title)

    # 4. Compare the regenerated html against the original html
    canonicalized_original_html = canonicalize_html(html_content)
    assert canonicalize_html(regenerated_html) == canonicalized_original_html