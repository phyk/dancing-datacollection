import os
import pytest
from bs4 import BeautifulSoup
from dancing_datacollection.parsing_topturnier import TopTurnierParser
from dancing_datacollection.html_generate import (
    generate_deck_html,
    generate_erg_html,
    generate_tabges_html,
    generate_ergwert_html,
)
from dancing_datacollection.html_canonicalize import canonicalize_html
from dancing_datacollection.data_defs.committee import CommitteeMember
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.score import FinalRoundScore
from dancing_datacollection.data_defs.results import ResultRound

TEST_DIR = os.path.dirname(__file__)
SAMPLE_DIRS = [
    "51-1105_ot_hgr2dstd",
    "52-1105_ot_hgr2cstd",
    "53-1105_ot_hgr2bstd",
]

def read_file_content(path):
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


@pytest.mark.parametrize("sample_dir", SAMPLE_DIRS)
def test_generate_deck_html(sample_dir):
    dir_path = os.path.join(TEST_DIR, sample_dir)
    parser = TopTurnierParser()

    # Parse from deck.htm
    deck_path = os.path.join(dir_path, "deck.htm")
    deck_html_in = read_file_content(deck_path)
    soup = BeautifulSoup(deck_html_in, "html.parser")
    title = soup.title.string if soup.title else "deck"
    judges = parser.extract_judges(deck_html_in, filename="deck.htm")
    committee = parser.extract_committee(deck_html_in)

    # Generate HTML
    generated_html = generate_deck_html(judges, committee, title=title)

    # Compare with golden file
    golden_path = os.path.join(dir_path, "deck.golden.htm")
    golden_html = read_file_content(golden_path)

    assert canonicalize_html(generated_html) == canonicalize_html(golden_html)


@pytest.mark.parametrize("sample_dir", SAMPLE_DIRS)
def test_generate_ergwert_html(sample_dir):
    dir_path = os.path.join(TEST_DIR, sample_dir)
    parser = TopTurnierParser()

    # Parse from ergwert.htm
    ergwert_path = os.path.join(dir_path, "ergwert.htm")
    if not os.path.exists(ergwert_path):
        pytest.skip(f"ergwert.htm not found in {sample_dir}")
    ergwert_html_in = read_file_content(ergwert_path)
    soup = BeautifulSoup(ergwert_html_in, "html.parser")
    title = soup.title.string if soup.title else "ergwert"
    tables_data = parser.parse_ergwert_all(ergwert_html_in)

    # Generate HTML
    generated_html = generate_ergwert_html(tables_data, title=title)

    # Compare with golden file
    golden_path = os.path.join(dir_path, "ergwert.golden.htm")
    golden_html = read_file_content(golden_path)

    assert canonicalize_html(generated_html) == canonicalize_html(golden_html)


from dancing_datacollection.parsing.erg import extract_results_from_erg


@pytest.mark.parametrize("sample_dir", SAMPLE_DIRS)
def test_generate_tabges_html(sample_dir):
    dir_path = os.path.join(TEST_DIR, sample_dir)
    parser = TopTurnierParser()

    # Parse from tabges.htm
    tabges_path = os.path.join(dir_path, "tabges.htm")
    tabges_html_in = read_file_content(tabges_path)
    soup = BeautifulSoup(tabges_html_in, "html.parser")
    title = soup.title.string if soup.title else "tabges"
    tables_data = parser.parse_tabges_all(tabges_html_in)

    # Generate HTML
    generated_html = generate_tabges_html(tables_data, title=title)

    # Compare with golden file
    golden_path = os.path.join(dir_path, "tabges.golden.htm")
    golden_html = read_file_content(golden_path)

    assert canonicalize_html(generated_html) == canonicalize_html(golden_html)


@pytest.mark.parametrize("sample_dir", SAMPLE_DIRS)
def test_generate_erg_html(sample_dir):
    dir_path = os.path.join(TEST_DIR, sample_dir)

    # Parse from erg.htm
    erg_path = os.path.join(dir_path, "erg.htm")
    erg_html_in = read_file_content(erg_path)
    soup = BeautifulSoup(erg_html_in, "html.parser")
    title = soup.title.string if soup.title else "erg"

    results = extract_results_from_erg(erg_html_in)

    # Generate HTML
    generated_html = generate_erg_html(results, title=title)

    # Compare with golden file
    golden_path = os.path.join(dir_path, "erg.golden.htm")
    golden_html = read_file_content(golden_path)

    assert canonicalize_html(generated_html) == canonicalize_html(golden_html)