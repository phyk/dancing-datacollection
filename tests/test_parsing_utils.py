import pytest
from bs4 import BeautifulSoup

from dancing_datacollection.parsing.parsing_utils import (
    as_class_list,
    element_has_class,
    first_line_text,
    deduplicate_judges,
    split_names,
    extract_name_and_club_from_spans,
    deduplicate_participants,
)
from dancing_datacollection.data_defs.judge import Judge


def test_as_class_list():
    assert as_class_list(["a", "b"]) == ["a", "b"]
    assert as_class_list("a b") == ["a b"]
    assert as_class_list(None) == []


def test_element_has_class():
    soup = BeautifulSoup('<div class="x y">hi</div>', "html.parser")
    div = soup.find("div")
    assert element_has_class(div, "x") is True
    assert element_has_class(div, "z") is False


def test_first_line_text():
    soup = BeautifulSoup("<td>12\n34</td>", "html.parser")
    td = soup.find("td")
    assert first_line_text(td) == "12"


def test_deduplicate_judges_prefers_with_club():
    judges = [
        Judge(code="AB", name="Alice Smith", club=""),
        Judge(code="AB", name="Alice Smith", club="ClubX"),
        Judge(code="CD", name="Bob Doe", club=None),
        Judge(code="CD", name="Bob Doe", club=""),
    ]
    deduped = deduplicate_judges(judges)
    key_to_club = {(j.code, j.name): j.club for j in deduped}
    assert key_to_club[("AB", "Alice Smith")] == "ClubX"
    assert ("CD", "Bob Doe") in key_to_club


def test_split_names_common_delimiters():
    assert split_names("Alice / Bob") == ("Alice", "Bob")
    assert split_names("Alice & Bob") == ("Alice", "Bob")
    assert split_names("Alice und Bob") == ("Alice", "Bob")
    assert split_names("Alice and Bob") == ("Alice", "Bob")


def test_split_names_whitespace_fallback():
    assert split_names("Alice Bob") == ("Alice", "Bob")
    assert split_names("Single") == (None, None)


def test_extract_name_and_club_from_spans_prefers_spans():
    from bs4 import BeautifulSoup

    soup = BeautifulSoup(
        "<td><span>Alice & Bob</span><span>Club X</span></td>", "html.parser"
    )
    td = soup.find("td")
    name, club = extract_name_and_club_from_spans(td)
    assert name == "Alice & Bob"
    assert club == "Club X"


def test_extract_name_and_club_from_spans_single_span():
    from bs4 import BeautifulSoup

    soup = BeautifulSoup("<td><span>Alice & Bob</span></td>", "html.parser")
    td = soup.find("td")
    name, club = extract_name_and_club_from_spans(td)
    assert name == "Alice & Bob"
    assert club == ""


def test_deduplicate_participants_by_number_name_club():
    items = [
        {"number": 1, "names": "A / B", "club": "X"},
        {"number": 1, "names": "A / B", "club": "X"},
        {"number": 2, "names": "C / D", "club": "Y"},
        {"number": 2, "names": "C / D", "club": None},
    ]
    out = deduplicate_participants(items)
    keys = {(p["number"], p["names"], p["club"]) for p in out}
    assert (1, "A / B", "X") in keys
    assert (2, "C / D", "Y") in keys
