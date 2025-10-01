import os
from typing import Dict, List

import pytest

from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.parsing.erg import extract_participants_from_erg
from dancing_datacollection.parsing.ergwert import extract_participants_from_ergwert
from dancing_datacollection.parsing.parsing_utils import get_soup
from dancing_datacollection.parsing.tabges import extract_participants_from_tabges
from dancing_datacollection.parsing.wert_er import extract_participants_from_wert_er


def get_html(path: str) -> str:
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


# Exhaustive ground truth participants for each test case, as Participant objects
@pytest.fixture(scope="module")
def true_participants() -> Dict[str, List[Participant]]:
    return {
        "51-1105_ot_hgr2dstd": [
            Participant(
                name_one="Jonathan Kummetz",
                name_two="Elisabeth Findeiß",
                number=610,
                ranks=[1],
                club="1. TC Rot-Gold Bayreuth",
            ),
            Participant(
                name_one="Konstantin Plöger",
                name_two="Laura Utz",
                number=616,
                ranks=[2],
                club="TSZ Blau-Gold Casino, Darmstadt",
            ),
            Participant(
                name_one="Maik Rau",
                name_two="Carina Rau",
                number=617,
                ranks=[3],
                club="Flensburger TC",
            ),
            Participant(
                name_one="Raphael Michel",
                name_two="Carolin Kimmig",
                number=611,
                ranks=[4],
                club="TSC Grün-Gold Heidelberg",
            ),
            Participant(
                name_one="Sullivan Sadzik",
                name_two="Laura Mayer",
                number=619,
                ranks=[5],
                club="TC Rot-Weiß Kaiserslautern",
            ),
            Participant(
                name_one="Emanuel Ostermaier",
                name_two="Katharina Dropmann",
                number=615,
                ranks=[6],
                club="1. Tanzsport Zentrum Freising",
            ),
            Participant(
                name_one="Thilo Schmid",
                name_two="Katharina Zierer",
                number=621,
                ranks=[7],
                club="Dance Unlimited",
            ),
            Participant(
                name_one="Tobias Knop",
                name_two="Cathrin Rube",
                number=607,
                ranks=[8, 9],
                club="TSC Rot-Gold Sinsheim",
            ),
            Participant(
                name_one="Stefan Mühl",
                name_two="Eva Horlebein",
                number=613,
                ranks=[8, 9],
                club="TC Rot-Gold Würzburg",
            ),
            Participant(
                name_one="David Krause",
                name_two="Sophia Maier",
                number=609,
                ranks=[10],
                club="TC Rot-Gold Würzburg",
            ),
            Participant(
                name_one="Simon Junski",
                name_two="Carolin Schilpp",
                number=606,
                ranks=[11],
                club="TanzZentrum Ludwigshafen",
            ),
            Participant(
                name_one="Felix Gasteiger",
                name_two="Nathalie Gleixner",
                number=604,
                ranks=[12, 13],
                club="Tanz-Club Laaber",
            ),
            Participant(
                name_one="Linus Witascheck",
                name_two="Gilda Stechhan",
                number=624,
                ranks=[12, 13],
                club="Rot-Weiss-Klub Kassel",
            ),
            Participant(
                name_one="Jan Dingerkus",
                name_two="Diana Vorst",
                number=602,
                ranks=[14, 15],
                club="TSC Blau-Gold-Rondo Bonn",
            ),
            Participant(
                name_one="Jonas Dreier",
                name_two="Johanna Grebe",
                number=603,
                ranks=[14, 15],
                club="Gießener Tanz-Club 74",
            ),
            Participant(
                name_one="Georg Arndt",
                name_two="Anika Johlke",
                number=600,
                ranks=[16],
                club="1. TSC Grün-Gold Leipzig 1947",
            ),
            Participant(
                name_one="Sebastian Moch",
                name_two="Anna Melina Faude",
                number=612,
                ranks=[17],
                club="TSC Residenz Ludwigsburg",
            ),
            Participant(
                name_one="Waldemar Schilke",
                name_two="Isabell Grubert",
                number=620,
                ranks=[18],
                club="TSC dancepoint, Königsbrunn",
            ),
            Participant(
                name_one="Marco Conrad",
                name_two="Alina Tempelmann",
                number=601,
                ranks=[19, 21],
                club="Tanzsportakademie Ludwigsburg",
            ),
            Participant(
                name_one="Carsten Giersberg",
                name_two="Jennifer Rath",
                number=605,
                ranks=[19, 21],
                club="TSA d. TUS Stuttgart 1867",
            ),
            Participant(
                name_one="Daniel Vitt",
                name_two="Tatjana Pankratz-Milstein",
                number=623,
                ranks=[19, 21],
                club="UTC Münster",
            ),
            Participant(
                name_one="Nicolas Koch",
                name_two="Christina Kalliafa",
                number=608,
                ranks=[22, 23],
                club="Tanzsportclub Solitude Kornwestheim",
            ),
            Participant(
                name_one="Thomas Rösch",
                name_two="Ganna Kovtun",
                number=618,
                ranks=[22, 23],
                club="TSC Rot-Gold-Casino Nürnberg",
            ),
            Participant(
                name_one="Lukas Thürmer",
                name_two="Madeleine Klotzbücher",
                number=622,
                ranks=[24],
                club="TC Rot-Weiss Schwäbisch Gmünd",
            ),
            Participant(
                name_one="Thorsten Olemotz",
                name_two="Jennifer Albach",
                number=614,
                ranks=[25],
                club="Gießener Tanz-Club 74",
            ),
        ],
        "52-1105_ot_hgr2cstd": [
            Participant(
                name_one="Kai Klede",
                name_two="Amke Beenen",
                number=519,
                ranks=[1],
                club="TSC Erlangen d. TB 1888",
            ),
            Participant(
                name_one="Gregor Kobsik",
                name_two="Angelina Kleiber",
                number=521,
                ranks=[2],
                club="TSC Grün-Weiß Aquisgrana Aachen",
            ),
            Participant(
                name_one="Sebastian Hauber",
                name_two="Amelie Goldfuß",
                number=516,
                ranks=[3],
                club="TSA Schwarz-Gold d. ESV Ingolstadt",
            ),
            Participant(
                name_one="Lukas Kuschel",
                name_two="Katharina Hölzchen",
                number=523,
                ranks=[4],
                club="TSC Schwarz-Gold im ASC Göttingen v 1846 e.V",
            ),
            Participant(
                name_one="Dr. Felix Prihoda",
                name_two="Dr. Annemarie Prihoda",
                number=528,
                ranks=[5],
                club="TTC Erlangen",
            ),
            Participant(
                name_one="Peter Brantsch",
                name_two="Luisa Böck",
                number=505,
                ranks=[6],
                club="TSC Astoria Karlsruhe",
            ),
            Participant(
                name_one="Fionn Woghen Dr. Wentorp",
                name_two="Aila Meschgbu",
                number=509,
                ranks=[7],
                club="TSC Olsberg",
            ),
            Participant(
                name_one="Martin Günther",
                name_two="Sarah Pätow",
                number=514,
                ranks=[8],
                club="TSC Astoria Karlsruhe",
            ),
            Participant(
                name_one="Christian Peters",
                name_two="Ronja Hormes",
                number=526,
                ranks=[9],
                club="TSZ Blau-Gold Casino, Darmstadt",
            ),
            Participant(
                name_one="Nicklas Benedikt Neufang",
                name_two="Eva Eisenhardt, Laura",
                number=525,
                ranks=[10],
                club="TSC Rot-Weiß Böblingen",
            ),
            Participant(
                name_one="Christoph Schlüter",
                name_two="Franziska Gerlach",
                number=532,
                ranks=[11],
                club="Tanzsportclub Dortmund",
            ),
            Participant(
                name_one="Sebastian Damm",
                name_two="Jantje Rippe",
                number=506,
                ranks=[12],
                club="TSA d. TV Schwanewede v. 1903",
            ),
            Participant(
                name_one="Bernd Krauss",
                name_two="Jennifer Steuer",
                number=522,
                ranks=[13],
                club="TSA d. TSV Schmiden",
            ),
            Participant(
                name_one="Maximilian Dörner",
                name_two="Anita Dörner",
                number=508,
                ranks=[14, 15],
                club="TanzZentrum Ludwigshafen",
            ),
            Participant(
                name_one="Klaus Raab",
                name_two="Katalin Veszpremi",
                number=529,
                ranks=[14, 15],
                club="TC Blau-Orange Wiesbaden",
            ),
            Participant(
                name_one="Johannes Pfeiffer",
                name_two="Tamara Pfeiffer",
                number=527,
                ranks=[16, 17],
                club="Tanzsportclub Trier",
            ),
            Participant(
                name_one="Frank Thiemicke",
                name_two="Lea Offermann",
                number=534,
                ranks=[16, 17],
                club="TSC Astoria Karlsruhe",
            ),
            Participant(
                name_one="Marcel Rose",
                name_two="Imke Schwan",
                number=530,
                ranks=[18],
                club="TC Kristall Jena",
            ),
            Participant(
                name_one="Carsten Beck",
                name_two="Jennifer Arnold",
                number=501,
                ranks=[19, 21],
                club="TC Rot-Weiss Casino Mainz",
            ),
            Participant(
                name_one="Markus Hajek",
                name_two="Eva Hajek",
                number=515,
                ranks=[19, 21],
                club="TC Rot-Weiss Casino Mainz",
            ),
            Participant(
                name_one="Patrick Keller",
                name_two="Stefanie Schenker",
                number=518,
                ranks=[19, 21],
                club="TSC dancepoint, Königsbrunn",
            ),
            Participant(
                name_one="Stefan Fischer",
                name_two="Mareike Maass",
                number=511,
                ranks=[22, 23],
                club="Rot-Weiß-Club Gießen",
            ),
            Participant(
                name_one="Jakob Zwicker",
                name_two="Magdalena Bedner",
                number=536,
                ranks=[22, 23],
                club="Tanzclub Konstanz",
            ),
            Participant(
                name_one="Alexander Behmer",
                name_two="Dr. Juliane Scheil",
                number=502,
                ranks=[24],
                club="Tanzsportzentrum Wetter-Ruhr",
            ),
            Participant(
                name_one="David Schneider",
                name_two="Sonja Wendenburg",
                number=533,
                ranks=[25],
                club="TSZ Blau-Gold Casino, Darmstadt",
            ),
            Participant(
                name_one="Andreas Baumeister",
                name_two="Cäcilia Benzin",
                number=500,
                ranks=[26, 28],
                club="TSC Astoria Karlsruhe",
            ),
            Participant(
                name_one="Achim Besler",
                name_two="Kathrin Besler",
                number=504,
                ranks=[26, 28],
                club="TSA d. TSG 1861 Grünstadt",
            ),
            Participant(
                name_one="Paul Wehle",
                name_two="Melanie Höschele",
                number=535,
                ranks=[26, 28],
                club="Tanzsportclub Balance Berlin",
            ),
            Participant(
                name_one="Pascal Gerbert",
                name_two="Nelly Gerbert",
                number=512,
                ranks=[29, 30],
                club="TSC Welfen Weingarten",
            ),
            Participant(
                name_one="Dirk Schäfer",
                name_two="Gertrud Lembke",
                number=531,
                ranks=[29, 30],
                club="TSZ Blau-Gold Casino, Darmstadt",
            ),
            Participant(
                name_one="Alexander Kober",
                name_two="Diana Bühren",
                number=520,
                ranks=[31],
                club="Tanzsportclub Dortmund",
            ),
            Participant(
                name_one="Martin Erhardt",
                name_two="Svenja Mozian",
                number=510,
                ranks=[32],
                club="Tanz- u. Sportzentr. Mittelrhein, Koblenz",
            ),
            Participant(
                name_one="Patrick Hiebl",
                name_two="Sylvia Kißmehl",
                number=517,
                ranks=[33],
                club="WTC Friedberg",
            ),
            Participant(
                name_one="Christian Deike",
                name_two="Patrycja Krohn",
                number=507,
                ranks=[34],
                club="1. TSZ im Turn-Klubb zu Hannover",
            ),
        ],
        "53-1105_ot_hgr2bstd": [
            Participant(
                name_one="Maximilian Beichter",
                name_two="Melissa Hagel",
                number=401,
                ranks=[1],
                club="TSC Astoria Karlsruhe",
            ),
            Participant(
                name_one="Leif-Erik Montag",
                name_two="Johanna Wille",
                number=421,
                ranks=[2],
                club="Tanzsportteam im ASC Göttingen v. 1846",
            ),
            Participant(
                name_one="Marcus Nguyen Ngoc",
                name_two="Lea Teßmer",
                number=423,
                ranks=[3],
                club="Club Céronne im ETV Hamburg",
            ),
            Participant(
                name_one="Patrick Dahm",
                name_two="Sandra Schwarz",
                number=408,
                ranks=[4],
                club="TC Der Frankfurter Kreis",
            ),
            Participant(
                name_one="Christopher Buchloh-Rosenthal",
                name_two="Analena Koch",
                number=405,
                ranks=[5],
                club="Rot-Weiss-Klub Kassel",
            ),
            Participant(
                name_one="Jakob Hinz",
                name_two="Vivien Bachmann",
                number=415,
                ranks=[6],
                club="TC Kristall Jena",
            ),
            Participant(
                name_one="Mariusz Budek",
                name_two="Marta Budek",
                number=406,
                ranks=[7],
                club="TSC Villingen-Schwenningen",
            ),
            Participant(
                name_one="Christoph Hanisch",
                name_two="Kaja Zoé Pfüller",
                number=412,
                ranks=[8],
                club="UTSC Choice Styria",
            ),
            Participant(
                name_one="Sebastian Hellmann",
                name_two="Melanie Oberhauser",
                number=414,
                ranks=[9],
                club="TTC Oldenburg",
            ),
            Participant(
                name_one="Oliver Neumann",
                name_two="Anna-Maria Ehinger",
                number=422,
                ranks=[10],
                club="TSC Astoria Karlsruhe",
            ),
            Participant(
                name_one="Ruslan Wellner",
                name_two="Tabea Kilian",
                number=430,
                ranks=[11, 12],
                club="Braunschweig Dance Company",
            ),
            Participant(
                name_one="Leo Werner",
                name_two="Fabienne Theobalt",
                number=432,
                ranks=[11, 12],
                club="TC Rot-Weiss Casino Mainz",
            ),
            Participant(
                name_one="Uli Kunz",
                name_two="Saskia Morcinczyk",
                number=420,
                ranks=[13, 14],
                club="TSC Grün-Gold Speyer",
            ),
            Participant(
                name_one="Martin Wenhart",
                name_two="Lisa Harrell",
                number=431,
                ranks=[13, 14],
                club="TSC dancepoint, Königsbrunn",
            ),
            Participant(
                name_one="René Kaczorowski",
                name_two="Cindy Hebert",
                number=417,
                ranks=[15],
                club="Tanzsportverein Schwarz-Weiß Freiberg",
            ),
            Participant(
                name_one="Benedikt Ernst",
                name_two="Tanja Esche",
                number=411,
                ranks=[16],
                club="TSC Rot-Gold-Casino Nürnberg",
            ),
            Participant(
                name_one="Fabian Beckmann",
                name_two="Katrin Langert",
                number=400,
                ranks=[17],
                club="TSC Schwarz-Gelb Aachen",
            ),
            Participant(
                name_one="Patrick Rach",
                name_two="Lorena Kimmel",
                number=427,
                ranks=[18, 19],
                club="TC Rot-Weiss Casino Mainz",
            ),
            Participant(
                name_one="Enrico Weber",
                name_two="Anne-Kathrin Nitt-Weber",
                number=429,
                ranks=[18, 19],
                club="Tanzsportzentrum Dresden",
            ),
            Participant(
                name_one="Max Kirchenberger",
                name_two="Friederike Rust",
                number=418,
                ranks=[20],
                club="TTC München",
            ),
            Participant(
                name_one="Johannes Kreim",
                name_two="Rebecca Gonzalez-Ringer",
                number=419,
                ranks=[21, 22],
                club="TC Rot-Weiss Casino Mainz",
            ),
            Participant(
                name_one="Patrik Pollak",
                name_two="Pia Feischen",
                number=426,
                ranks=[21, 22],
                club="TSC Grün-Gold Heidelberg",
            ),
            Participant(
                name_one="Christopher Brix",
                name_two="Sandra Kuhfus",
                number=403,
                ranks=[23, 24],
                club="TSC Grün-Weiß Aquisgrana Aachen",
            ),
            Participant(
                name_one="Michael Wrusch",
                name_two="Dan Feng Tian Helena Liang",
                number=434,
                ranks=[23, 24],
                club="OTK Schwarz-Weiß 1922 im SCS Berlin",
            ),
            Participant(
                name_one="Dr. Michel Oelschlägel",
                name_two="Dagmar Lina Ganz",
                number=424,
                ranks=[25, 26],
                club="Tanzsportverein Schwarz-Weiß Freiberg",
            ),
            Participant(
                name_one="Matty Schiller",
                name_two="Anne Wienhold",
                number=428,
                ranks=[25, 26],
                club="TSA d. TTC Allround Rostock",
            ),
            Participant(
                name_one="Christoph Hellings",
                name_two="Maria Korosteleva",
                number=413,
                ranks=[27, 28],
                club="Switzerland",
            ),
            Participant(
                name_one="Michael Hopf",
                name_two="Iris Hopf",
                number=416,
                ranks=[27, 28],
                club="TSC Unterschleißheim",
            ),
            Participant(
                name_one="Thomas Brunnengräber",
                name_two="Mirjam Tittlus",
                number=404,
                ranks=[29],
                club="TC Blau-Orange Wiesbaden",
            ),
            Participant(
                name_one="Robert Podgajny",
                name_two="Olesya Oshchepkova",
                number=425,
                ranks=[30],
                club="TTC Rot-Weiß Freiburg",
            ),
            Participant(
                name_one="Florian Dammeyer",
                name_two="Yasmin Deborah Gers",
                number=409,
                ranks=[31],
                club="Die Residenz Münster",
            ),
        ],
    }


@pytest.mark.parametrize(
    "sample_dir",
    [
        pytest.param(d)
        for d in ["51-1105_ot_hgr2dstd", "52-1105_ot_hgr2cstd", "53-1105_ot_hgr2bstd"]
    ],
)
def test_extract_participants_from_erg(
    sample_dir: str, test_dir: str, true_participants: Dict[str, List[Participant]]
) -> None:
    erg_path = os.path.join(test_dir, sample_dir, "erg.htm")
    if not os.path.exists(erg_path):
        pytest.skip(f"Missing {erg_path}")
    html = get_html(erg_path)
    participants = extract_participants_from_erg(html)
    assert isinstance(participants, list)
    assert all(isinstance(p, Participant) for p in participants)
    assert participants, f"No participants extracted from {erg_path}"
    keys = set()
    for p in participants:
        key = (p.number, p.name_one, p.club)
        assert key not in keys, f"Duplicate participant {key} in {erg_path}"
        keys.add(key)
        assert p.name_one is not None
        assert p.number is not None
    # Check all ground truth participants are present (full match)
    expected = true_participants[sample_dir]
    for tp in expected:
        assert any(p.matches_full(tp) for p in participants), (
            f"Missing participant: {tp}"
        )


@pytest.mark.parametrize(
    "sample_dir",
    [
        pytest.param(d)
        for d in ["51-1105_ot_hgr2dstd", "52-1105_ot_hgr2cstd", "53-1105_ot_hgr2bstd"]
    ],
)
def test_extract_participants_from_ergwert(
    sample_dir: str, test_dir: str, true_participants: Dict[str, List[Participant]]
) -> None:
    ergwert_path = os.path.join(test_dir, sample_dir, "ergwert.htm")
    if not os.path.exists(ergwert_path):
        pytest.skip(f"Missing {ergwert_path}")
    html = get_html(ergwert_path)
    soup = get_soup(html)
    participants = extract_participants_from_ergwert(soup)
    assert isinstance(participants, list)
    assert all(isinstance(p, Participant) for p in participants)
    assert participants, f"No participants extracted from {ergwert_path}"
    keys = set()
    for p in participants:
        key = (p.number, p.name_one, p.club)
        assert key not in keys, f"Duplicate participant {key} in {ergwert_path}"
        keys.add(key)
        assert p.name_one is not None
        assert p.number is not None
    # Check all ground truth participants are present (full match)
    expected = true_participants[sample_dir]
    for tp in expected:
        assert any(p.matches_full(tp) for p in participants), (
            f"Missing participant: {tp}"
        )


@pytest.mark.parametrize(
    "sample_dir",
    [
        pytest.param(d)
        for d in ["51-1105_ot_hgr2dstd", "52-1105_ot_hgr2cstd", "53-1105_ot_hgr2bstd"]
    ],
)
def test_extract_participants_from_tabges(
    sample_dir: str, test_dir: str, true_participants: Dict[str, List[Participant]]
) -> None:
    tabges_path = os.path.join(test_dir, sample_dir, "tabges.htm")
    if not os.path.exists(tabges_path):
        pytest.skip(f"Missing {tabges_path}")
    html = get_html(tabges_path)
    soup = get_soup(html)
    participants = extract_participants_from_tabges(soup)
    assert isinstance(participants, list)
    assert all(isinstance(p, Participant) for p in participants)
    assert participants, f"No participants extracted from {tabges_path}"
    keys = set()
    for p in participants:
        key = (p.number, p.name_one, p.club)
        assert key not in keys, f"Duplicate participant {key} in {tabges_path}"
        keys.add(key)
        assert p.number is not None, f"Missing number in {p}"
    # Check all ground truth participants are present (partial match)
    expected = true_participants[sample_dir]
    for tp in expected:
        assert any(p.matches_partial(tp) for p in participants), (
            f"Missing participant: {tp}"
        )


@pytest.mark.parametrize(
    "sample_dir",
    [
        pytest.param(d)
        for d in ["51-1105_ot_hgr2dstd", "52-1105_ot_hgr2cstd", "53-1105_ot_hgr2bstd"]
    ],
)
def test_extract_participants_from_wert_er(
    sample_dir: str, test_dir: str, true_participants: Dict[str, List[Participant]]
) -> None:
    wert_er_path = os.path.join(test_dir, sample_dir, "wert_er.htm")
    if not os.path.exists(wert_er_path):
        pytest.skip(f"Missing {wert_er_path}")
    html = get_html(wert_er_path)
    soup = get_soup(html)
    participants = extract_participants_from_wert_er(soup)
    assert isinstance(participants, list)
    assert all(isinstance(p, Participant) for p in participants)
    assert participants, f"No participants extracted from {wert_er_path}"
    keys = set()
    for p in participants:
        key = (p.number, p.name_one, p.club)
        assert key not in keys, f"Duplicate participant {key} in {wert_er_path}"
        keys.add(key)
        assert p.name_one is not None
        assert p.number is not None
    # Only check that each found participant is present in the ground truth (partial match)
    expected = true_participants[sample_dir]
    for p in participants:
        assert any(p.matches_partial(tp) for tp in expected), (
            f"Extracted participant not in ground truth: {p}"
        )