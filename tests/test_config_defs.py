import datetime
from dancing_datacollection.data_defs.age_group import AgeGroup
from dancing_datacollection.data_defs.discipline import Discipline
from dancing_datacollection.data_defs.level import Level
from dancing_datacollection.parsing.parsing_utils import parse_competition_title

def test_parse_competition_title():
    title = "11.05.2024 Hgr.II B Standard"
    info = parse_competition_title(title)

    assert info.comp_date == datetime.date(2024, 5, 11)
    assert info.age_group == AgeGroup.ADULT_2
    assert info.level == Level.B
    assert info.discipline == Discipline.BALLROOM
    assert info.min_dances == 5

def test_parse_competition_title_multiword():
    title = "11.05.2024 Kinder I D Latein"
    info = parse_competition_title(title)

    assert info.comp_date == datetime.date(2024, 5, 11)
    assert info.age_group == AgeGroup.JUV_1
    assert info.level == Level.D
    assert info.discipline == Discipline.LATIN
    assert info.min_dances == 3

def test_level_min_dances_2026():
    # Level D legacy: 3, 2026: 4
    level_d = Level.D
    assert level_d.get_min_dances(datetime.date(2024, 5, 11)) == 3
    assert level_d.get_min_dances(datetime.date(2026, 1, 1)) == 4

def test_level_min_dances_C():
    # Level C legacy: 4, 2026: 5
    level_c = Level.C
    assert level_c.get_min_dances(datetime.date(2024, 5, 11)) == 4
    assert level_c.get_min_dances(datetime.date(2026, 1, 1)) == 5
