import os
import pytest
from bs4 import BeautifulSoup
from dancing_datacollection.html_generate import (
    generate_deck_html,
    generate_erg_html,
    generate_tabges_html,
    generate_ergwert_html,
)
from dancing_datacollection.html_canonicalize import canonicalize_html
from dancing_datacollection.parsing.deck import extract_judges_from_deck
from dancing_datacollection.parsing.committee import extract_committee_from_deck
from dancing_datacollection.parsing.ergwert import parse_ergwert_all
from dancing_datacollection.parsing.tabges import parse_tabges_all
from dancing_datacollection.parsing.erg import extract_results_from_erg
from dancing_datacollection.parsing.parsing_utils import get_soup

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

    # Parse from deck.htm
    deck_path = os.path.join(dir_path, "deck.htm")
    deck_html_in = read_file_content(deck_path)
    soup = get_soup(deck_html_in)
    title = soup.title.string if soup.title else "deck"
    judges = extract_judges_from_deck(soup)
    committee = extract_committee_from_deck(soup)

    # Generate HTML
    generated_html = generate_deck_html(judges, committee, title=title)

    # Compare with golden file
    golden_path = os.path.join(dir_path, "deck.golden.htm")
    golden_html = read_file_content(golden_path)

    assert canonicalize_html(generated_html) == canonicalize_html(golden_html)


@pytest.mark.parametrize("sample_dir", SAMPLE_DIRS)
def test_generate_ergwert_html(sample_dir):
    dir_path = os.path.join(TEST_DIR, sample_dir)

    # Parse from ergwert.htm
    ergwert_path = os.path.join(dir_path, "ergwert.htm")
    if not os.path.exists(ergwert_path):
        pytest.skip(f"ergwert.htm not found in {sample_dir}")
    ergwert_html_in = read_file_content(ergwert_path)
    soup = get_soup(ergwert_html_in)
    title = soup.title.string if soup.title else "ergwert"
    tables_data = parse_ergwert_all(ergwert_html_in)

    # Generate HTML
    generated_html = generate_ergwert_html(tables_data, title=title)

    # Compare with golden file
    golden_path = os.path.join(dir_path, "ergwert.golden.htm")
    golden_html = read_file_content(golden_path)

    assert canonicalize_html(generated_html) == canonicalize_html(golden_html)


@pytest.mark.parametrize("sample_dir", SAMPLE_DIRS)
def test_generate_tabges_html(sample_dir):
    dir_path = os.path.join(TEST_DIR, sample_dir)

    # Parse from tabges.htm
    tabges_path = os.path.join(dir_path, "tabges.htm")
    tabges_html_in = read_file_content(tabges_path)
    soup = get_soup(tabges_html_in)
    title = soup.title.string if soup.title else "tabges"
    tables_data = parse_tabges_all(tabges_html_in)

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
    soup = get_soup(erg_html_in)
    title = soup.title.string if soup.title else "erg"

    results = extract_results_from_erg(erg_html_in)

    # Generate HTML
    generated_html = generate_erg_html(results, title=title)

    # Compare with golden file
    golden_path = os.path.join(dir_path, "erg.golden.htm")
    golden_html = read_file_content(golden_path)

    assert canonicalize_html(generated_html) == canonicalize_html(golden_html)
