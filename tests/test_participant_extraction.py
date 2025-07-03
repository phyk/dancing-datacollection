import os
import pytest
from dancing_datacollection.parsing_topturnier import TopTurnierParser
from dancing_datacollection.data_defs.participant import Participant

def get_html(path):
    with open(path, 'r', encoding='utf-8') as f:
        return f.read()

# Exhaustive ground truth participants for each test case
@pytest.fixture(scope="module")
def true_participants():
    return {
        '51-1105_ot_hgr2dstd': [
            {"name_one": "Jonathan Kummetz", "name_two": "Elisabeth Findeiß", "club": "1. TC Rot-Gold Bayreuth", "number": 610},
            {"name_one": "Konstantin Plöger", "name_two": "Laura Utz", "club": "TSZ Blau-Gold Casino, Darmstadt", "number": 616},
            {"name_one": "Maik Rau", "name_two": "Carina Rau", "club": "Flensburger TC", "number": 617},
            {"name_one": "Raphael Michel", "name_two": "Carolin Kimmig", "club": "TSC Grün-Gold Heidelberg", "number": 611},
            {"name_one": "Sullivan Sadzik", "name_two": "Laura Mayer", "club": "TC Rot-Weiß Kaiserslautern", "number": 619},
            {"name_one": "Emanuel Ostermaier", "name_two": "Katharina Dropmann", "club": "1. Tanzsport Zentrum Freising", "number": 615},
        ],
        '52-1105_ot_hgr2cstd': [
            {"number": 519, "club": "TSC Erlangen d. TB 1888"},
            {"number": 521, "club": "TSC Grün-Weiß Aquisgrana Aachen"},
            {"number": 516, "club": "TSA Schwarz-Gold d. ESV Ingolstadt"},
            {"number": 523, "club": "TSC Schwarz-Gold im ASC Göttingen v 1846 e.V"},
            {"number": 528, "club": "TTC Erlangen"},
            {"number": 505, "club": "TSC Astoria Karlsruhe"},
        ],
        '53-1105_ot_hgr2bstd': [
            {"number": 401, "club": "TSC Astoria Karlsruhe"},
            {"number": 421, "club": "Tanzsportteam im  ASC Göttingen v. 1846"},
            {"number": 423, "club": "Club Céronne im ETV Hamburg"},
            {"number": 408, "club": "TC Der Frankfurter Kreis"},
            {"number": 405, "club": "Rot-Weiss-Klub Kassel"},
            {"number": 415, "club": "TC Kristall Jena"},
        ],
    }

@pytest.mark.parametrize('sample_dir', [pytest.param(d) for d in ['51-1105_ot_hgr2dstd', '52-1105_ot_hgr2cstd', '53-1105_ot_hgr2bstd']])
def test_extract_participants_from_erg(sample_dir, test_dir, true_participants):
    parser = TopTurnierParser()
    erg_path = os.path.join(test_dir, sample_dir, 'erg.htm')
    if not os.path.exists(erg_path):
        pytest.skip(f"Missing {erg_path}")
    html = get_html(erg_path)
    participants, _ = parser.extract_participants(html, filename='erg.htm')
    assert isinstance(participants, list)
    assert all(isinstance(p, Participant) for p in participants)
    assert participants, f"No participants extracted from {erg_path}"
    keys = set()
    for p in participants:
        key = (p.number, p.name_one, p.club)
        assert key not in keys, f"Duplicate participant {key} in {erg_path}"
        keys.add(key)
        assert p.name_one is not None and p.number is not None, f"Missing required fields in {p}"
    # Check all ground truth participants are present
    expected = true_participants[sample_dir]
    for tp in expected:
        assert any(
            (getattr(p, 'number', None) == tp.get('number')) and
            (tp.get('name_one') is None or getattr(p, 'name_one', None) == tp.get('name_one')) and
            (tp.get('name_two') is None or getattr(p, 'name_two', None) == tp.get('name_two')) and
            (getattr(p, 'club', None) == tp.get('club'))
            for p in participants
        ), f"Missing participant: {tp}"

@pytest.mark.parametrize('sample_dir', [pytest.param(d) for d in ['51-1105_ot_hgr2dstd', '52-1105_ot_hgr2cstd', '53-1105_ot_hgr2bstd']])
def test_extract_participants_from_ergwert(sample_dir, test_dir, true_participants):
    parser = TopTurnierParser()
    ergwert_path = os.path.join(test_dir, sample_dir, 'ergwert.htm')
    if not os.path.exists(ergwert_path):
        pytest.skip(f"Missing {ergwert_path}")
    html = get_html(ergwert_path)
    participants, _ = parser.extract_participants(html, filename='ergwert.htm')
    assert isinstance(participants, list)
    assert all(isinstance(p, Participant) for p in participants)
    assert participants, f"No participants extracted from {ergwert_path}"
    keys = set()
    for p in participants:
        key = (p.number, p.name_one, p.club)
        assert key not in keys, f"Duplicate participant {key} in {ergwert_path}"
        keys.add(key)
        assert p.name_one is not None and p.number is not None, f"Missing required fields in {p}"
    # Check all ground truth participants are present
    expected = true_participants[sample_dir]
    for tp in expected:
        assert any(
            (getattr(p, 'number', None) == tp.get('number')) and
            (tp.get('name_one') is None or getattr(p, 'name_one', None) == tp.get('name_one')) and
            (tp.get('name_two') is None or getattr(p, 'name_two', None) == tp.get('name_two')) and
            (getattr(p, 'club', None) == tp.get('club'))
            for p in participants
        ), f"Missing participant: {tp}"

@pytest.mark.parametrize('sample_dir', [pytest.param(d) for d in ['51-1105_ot_hgr2dstd', '52-1105_ot_hgr2cstd', '53-1105_ot_hgr2bstd']])
def test_extract_participants_from_tabges(sample_dir, test_dir, true_participants):
    parser = TopTurnierParser()
    tabges_path = os.path.join(test_dir, sample_dir, 'tabges.htm')
    if not os.path.exists(tabges_path):
        pytest.skip(f"Missing {tabges_path}")
    html = get_html(tabges_path)
    participants, _ = parser.extract_participants(html, filename='tabges.htm')
    assert isinstance(participants, list)
    assert all(isinstance(p, Participant) for p in participants)
    assert participants, f"No participants extracted from {tabges_path}"
    keys = set()
    for p in participants:
        key = (p.number, p.name_one, p.club)
        assert key not in keys, f"Duplicate participant {key} in {tabges_path}"
        keys.add(key)
        assert p.number is not None, f"Missing number in {p}"
    # Check all ground truth participants are present
    expected = true_participants[sample_dir]
    for tp in expected:
        assert any(
            (getattr(p, 'number', None) == tp.get('number')) and
            (tp.get('name_one') is None or getattr(p, 'name_one', None) == tp.get('name_one')) and
            (tp.get('name_two') is None or getattr(p, 'name_two', None) == tp.get('name_two')) and
            (getattr(p, 'club', None) == tp.get('club'))
            for p in participants
        ), f"Missing participant: {tp}"

@pytest.mark.parametrize('sample_dir', [pytest.param(d) for d in ['51-1105_ot_hgr2dstd', '52-1105_ot_hgr2cstd', '53-1105_ot_hgr2bstd']])
def test_extract_participants_from_wert_er(sample_dir, test_dir, true_participants):
    parser = TopTurnierParser()
    wert_er_path = os.path.join(test_dir, sample_dir, 'wert_er.htm')
    if not os.path.exists(wert_er_path):
        pytest.skip(f"Missing {wert_er_path}")
    html = get_html(wert_er_path)
    participants, _ = parser.extract_participants(html, filename='wert_er.htm')
    assert isinstance(participants, list)
    assert all(isinstance(p, Participant) for p in participants)
    assert participants, f"No participants extracted from {wert_er_path}"
    keys = set()
    for p in participants:
        key = (p.number, p.name_one, p.club)
        assert key not in keys, f"Duplicate participant {key} in {wert_er_path}"
        keys.add(key)
        assert p.name_one is not None and p.number is not None, f"Missing required fields in {p}"
    # Check all ground truth participants are present
    expected = true_participants[sample_dir]
    for tp in expected:
        assert any(
            (getattr(p, 'number', None) == tp.get('number')) and
            (tp.get('name_one') is None or getattr(p, 'name_one', None) == tp.get('name_one')) and
            (tp.get('name_two') is None or getattr(p, 'name_two', None) == tp.get('name_two')) and
            (getattr(p, 'club', None) == tp.get('club'))
            for p in participants
        ), f"Missing participant: {tp}" 