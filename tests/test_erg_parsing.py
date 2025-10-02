import pytest
from dancing_datacollection.data_defs.dances import Dance
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.results import (
    DanceScore,
    FinalRoundPlacing,
    PreliminaryRoundPlacing,
    ResultRound,
)
from dancing_datacollection.html_canonicalize import canonicalize_html
from dancing_datacollection.html_generate import generate_erg_html
from dancing_datacollection.parsing.erg import extract_results_from_erg

EXPECTED_ERG_RESULTS_51 = [
    ResultRound(
        name="Endrunde",
        placings=[
            FinalRoundPlacing(
                rank="1.",
                participant=Participant(
                    name_one="Jonathan Kummetz",
                    name_two="Elisabeth Findeiß",
                    number=610,
                    club="1. TC Rot-Gold Bayreuth",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[1, 1, 1, 3, 1], place=1.0),
                    Dance.TANGO: DanceScore(marks=[1, 1, 1, 4, 1], place=1.0),
                    Dance.QUICKSTEP: DanceScore(marks=[1, 1, 1, 3, 1], place=1.0),
                },
                total_score=3.0,
            ),
            FinalRoundPlacing(
                rank="2.",
                participant=Participant(
                    name_one="Konstantin Plöger",
                    name_two="Laura Utz",
                    number=616,
                    club="TSZ Blau-Gold Casino, Darmstadt",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[2, 2, 2, 1, 5], place=2.0),
                    Dance.TANGO: DanceScore(marks=[3, 3, 2, 5, 3], place=2.0),
                    Dance.QUICKSTEP: DanceScore(marks=[3, 4, 2, 4, 4], place=3.0),
                },
                total_score=7.0,
            ),
            FinalRoundPlacing(
                rank="3.",
                participant=Participant(
                    name_one="Maik Rau",
                    name_two="Carina Rau",
                    number=617,
                    club="Flensburger TC",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[4, 3, 3, 5, 2], place=3.0),
                    Dance.TANGO: DanceScore(marks=[5, 6, 3, 1, 6], place=6.0),
                    Dance.QUICKSTEP: DanceScore(marks=[5, 3, 3, 2, 5], place=2.0),
                },
                total_score=11.0,
            ),
            FinalRoundPlacing(
                rank="4.",
                participant=Participant(
                    name_one="Raphael Michel",
                    name_two="Carolin Kimmig",
                    number=611,
                    club="TSC Grün-Gold Heidelberg",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[6, 4, 6, 4, 3], place=5.0),
                    Dance.TANGO: DanceScore(marks=[6, 2, 6, 3, 2], place=3.0),
                    Dance.QUICKSTEP: DanceScore(marks=[6, 2, 4, 5, 2], place=4.0),
                },
                total_score=12.0,
            ),
            FinalRoundPlacing(
                rank="5.",
                participant=Participant(
                    name_one="Sullivan Sadzik",
                    name_two="Laura Mayer",
                    number=619,
                    club="TC Rot-Weiß Kaiserslautern",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[3, 5, 4, 2, 6], place=4.0),
                    Dance.TANGO: DanceScore(marks=[2, 4, 4, 2, 5], place=4.0),
                    Dance.QUICKSTEP: DanceScore(marks=[2, 6, 5, 1, 6], place=5.0),
                },
                total_score=13.0,
            ),
            FinalRoundPlacing(
                rank="6.",
                participant=Participant(
                    name_one="Emanuel Ostermaier",
                    name_two="Katharina Dropmann",
                    number=615,
                    club="1. Tanzsport Zentrum Freising",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[5, 6, 5, 6, 4], place=6.0),
                    Dance.TANGO: DanceScore(marks=[4, 5, 5, 6, 4], place=5.0),
                    Dance.QUICKSTEP: DanceScore(marks=[4, 5, 6, 6, 3], place=6.0),
                },
                total_score=17.0,
            ),
        ],
    ),
    ResultRound(
        name="2. Zwischenrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="7.",
                participant=Participant(
                    name_one="Thilo Schmid",
                    name_two="Katharina Zierer",
                    number=621,
                    club="Dance Unlimited",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="8.- 9.",
                participant=Participant(
                    name_one="Tobias Knop",
                    name_two="Cathrin Rube",
                    number=607,
                    club="TSC Rot-Gold Sinsheim",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="8.- 9.",
                participant=Participant(
                    name_one="Stefan Mühl",
                    name_two="Eva Horlebein",
                    number=613,
                    club="TC Rot-Gold Würzburg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="10.",
                participant=Participant(
                    name_one="David Krause",
                    name_two="Sophia Maier",
                    number=609,
                    club="TC Rot-Gold Würzburg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="11.",
                participant=Participant(
                    name_one="Simon Junski",
                    name_two="Carolin Schilpp",
                    number=606,
                    club="TanzZentrum Ludwigshafen",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="12.- 13.",
                participant=Participant(
                    name_one="Felix Gasteiger",
                    name_two="Nathalie Gleixner",
                    number=604,
                    club="Tanz-Club Laaber",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="12.- 13.",
                participant=Participant(
                    name_one="Linus Witascheck",
                    name_two="Gilda Stechhan",
                    number=624,
                    club="Rot-Weiss-Klub Kassel",
                ),
            ),
        ],
    ),
    ResultRound(
        name="1. Zwischenrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="14.- 15.",
                participant=Participant(
                    name_one="Jan Dingerkus",
                    name_two="Diana Vorst",
                    number=602,
                    club="TSC Blau-Gold-Rondo Bonn",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="14.- 15.",
                participant=Participant(
                    name_one="Jonas Dreier",
                    name_two="Johanna Grebe",
                    number=603,
                    club="Gießener Tanz-Club 74",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="16.",
                participant=Participant(
                    name_one="Georg Arndt",
                    name_two="Anika Johlke",
                    number=600,
                    club="1. TSC Grün-Gold Leipzig 1947",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="17.",
                participant=Participant(
                    name_one="Sebastian Moch",
                    name_two="Anna Melina Faude",
                    number=612,
                    club="TSC Residenz Ludwigsburg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="18.",
                participant=Participant(
                    name_one="Waldemar Schilke",
                    name_two="Isabell Grubert",
                    number=620,
                    club="TSC dancepoint, Königsbrunn",
                ),
            ),
        ],
    ),
    ResultRound(
        name="Vorrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="19.- 21.",
                participant=Participant(
                    name_one="Marco Conrad",
                    name_two="Alina Tempelmann",
                    number=601,
                    club="Tanzsportakademie Ludwigsburg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="19.- 21.",
                participant=Participant(
                    name_one="Carsten Giersberg",
                    name_two="Jennifer Rath",
                    number=605,
                    club="TSA d. TUS Stuttgart 1867",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="19.- 21.",
                participant=Participant(
                    name_one="Daniel Vitt",
                    name_two="Tatjana Pankratz-Milstein",
                    number=623,
                    club="UTC Münster",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="22.- 23.",
                participant=Participant(
                    name_one="Nicolas Koch",
                    name_two="Christina Kalliafa",
                    number=608,
                    club="Tanzsportclub Solitude Kornwestheim",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="22.- 23.",
                participant=Participant(
                    name_one="Thomas Rösch",
                    name_two="Ganna Kovtun",
                    number=618,
                    club="TSC Rot-Gold-Casino Nürnberg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="24.",
                participant=Participant(
                    name_one="Lukas Thürmer",
                    name_two="Madeleine Klotzbücher",
                    number=622,
                    club="TC Rot-Weiss Schwäbisch Gmünd",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="25.",
                participant=Participant(
                    name_one="Thorsten Olemotz",
                    name_two="Jennifer Albach",
                    number=614,
                    club="Gießener Tanz-Club 74",
                ),
            ),
        ],
    ),
]

EXPECTED_ERG_RESULTS_52 = [
    ResultRound(
        name="Endrunde",
        placings=[
            FinalRoundPlacing(
                rank="1.",
                participant=Participant(
                    name_one="Kai Klede",
                    name_two="Amke Beenen",
                    number=519,
                    club="TSC Erlangen d. TB 1888",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[5, 1, 6, 2, 3], place=2.0),
                    Dance.TANGO: DanceScore(marks=[3, 2, 3, 1, 1], place=1.0),
                    Dance.SLOW_FOXTROT: DanceScore(marks=[6, 1, 5, 1, 3], place=1.0),
                    Dance.QUICKSTEP: DanceScore(marks=[5, 1, 5, 1, 2], place=1.0),
                },
                total_score=5.0,
            ),
            FinalRoundPlacing(
                rank="2.",
                participant=Participant(
                    name_one="Gregor Kobsik",
                    name_two="Angelina Kleiber",
                    number=521,
                    club="TSC Grün-Weiß Aquisgrana Aachen",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[4, 2, 1, 3, 2], place=1.0),
                    Dance.TANGO: DanceScore(marks=[5, 1, 6, 4, 3], place=3.0),
                    Dance.SLOW_FOXTROT: DanceScore(marks=[4, 3, 1, 4, 2], place=2.0),
                    Dance.QUICKSTEP: DanceScore(marks=[4, 2, 3, 3, 3], place=3.0),
                },
                total_score=9.0,
            ),
            FinalRoundPlacing(
                rank="3.",
                participant=Participant(
                    name_one="Sebastian Hauber",
                    name_two="Amelie Goldfuß",
                    number=516,
                    club="TSA Schwarz-Gold d. ESV Ingolstadt",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[1, 5, 4, 4, 1], place=3.0),
                    Dance.TANGO: DanceScore(marks=[1, 4, 2, 3, 2], place=2.0),
                    Dance.SLOW_FOXTROT: DanceScore(marks=[1, 4, 4, 5, 1], place=4.0),
                    Dance.QUICKSTEP: DanceScore(marks=[1, 6, 2, 5, 1], place=2.0),
                },
                total_score=11.0,
            ),
            FinalRoundPlacing(
                rank="4.",
                participant=Participant(
                    name_one="Lukas Kuschel",
                    name_two="Katharina Hölzchen",
                    number=523,
                    club="TSC Schwarz-Gold im ASC Göttingen v 1846 e.V",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[2, 4, 3, 6, 4], place=4.0),
                    Dance.TANGO: DanceScore(marks=[2, 6, 4, 6, 5], place=6.0),
                    Dance.SLOW_FOXTROT: DanceScore(marks=[3, 5, 2, 3, 5], place=3.0),
                    Dance.QUICKSTEP: DanceScore(marks=[2, 5, 4, 4, 4], place=4.0),
                },
                total_score=17.0,
            ),
            FinalRoundPlacing(
                rank="5.",
                participant=Participant(
                    name_one="Dr. Felix Prihoda",
                    name_two="Dr. Annemarie Prihoda",
                    number=528,
                    club="TTC Erlangen",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[6, 3, 5, 1, 5], place=5.0),
                    Dance.TANGO: DanceScore(marks=[6, 3, 5, 2, 6], place=5.0),
                    Dance.SLOW_FOXTROT: DanceScore(marks=[5, 2, 6, 2, 4], place=5.0),
                    Dance.QUICKSTEP: DanceScore(marks=[6, 3, 6, 2, 6], place=6.0),
                },
                total_score=21.0,
            ),
            FinalRoundPlacing(
                rank="6.",
                participant=Participant(
                    name_one="Peter Brantsch",
                    name_two="Luisa Böck",
                    number=505,
                    club="TSC Astoria Karlsruhe",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(marks=[3, 6, 2, 5, 6], place=6.0),
                    Dance.TANGO: DanceScore(marks=[4, 5, 1, 5, 4], place=4.0),
                    Dance.SLOW_FOXTROT: DanceScore(marks=[2, 6, 3, 6, 6], place=6.0),
                    Dance.QUICKSTEP: DanceScore(marks=[3, 4, 1, 6, 5], place=5.0),
                },
                total_score=21.0,
            ),
        ],
    ),
    ResultRound(
        name="2. Zwischenrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="7.",
                participant=Participant(
                    name_one="Fionn Woghen Dr. Wentorp",
                    name_two="Aila Meschgbu",
                    number=509,
                    club="TSC Olsberg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="8.",
                participant=Participant(
                    name_one="Martin Günther",
                    name_two="Sarah Pätow",
                    number=514,
                    club="TSC Astoria Karlsruhe",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="9.",
                participant=Participant(
                    name_one="Christian Peters",
                    name_two="Ronja Hormes",
                    number=526,
                    club="TSZ Blau-Gold Casino, Darmstadt",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="10.",
                participant=Participant(
                    name_one="Nicklas Benedikt Neufang",
                    name_two="Eva Eisenhardt, Laura",
                    number=525,
                    club="TSC Rot-Weiß Böblingen",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="11.",
                participant=Participant(
                    name_one="Christoph Schlüter",
                    name_two="Franziska Gerlach",
                    number=532,
                    club="Tanzsportclub Dortmund",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="12.",
                participant=Participant(
                    name_one="Sebastian Damm",
                    name_two="Jantje Rippe",
                    number=506,
                    club="TSA d. TV Schwanewede v. 1903",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="13.",
                participant=Participant(
                    name_one="Bernd Krauss",
                    name_two="Jennifer Steuer",
                    number=522,
                    club="TSA d. TSV Schmiden",
                ),
            ),
        ],
    ),
    ResultRound(
        name="1. Zwischenrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="14.- 15.",
                participant=Participant(
                    name_one="Maximilian Dörner",
                    name_two="Anita Dörner",
                    number=508,
                    club="TanzZentrum Ludwigshafen",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="14.- 15.",
                participant=Participant(
                    name_one="Klaus Raab",
                    name_two="Katalin Veszpremi",
                    number=529,
                    club="TC Blau-Orange Wiesbaden",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="16.- 17.",
                participant=Participant(
                    name_one="Johannes Pfeiffer",
                    name_two="Tamara Pfeiffer",
                    number=527,
                    club="Tanzsportclub Trier",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="16.- 17.",
                participant=Participant(
                    name_one="Frank Thiemicke",
                    name_two="Lea Offermann",
                    number=534,
                    club="TSC Astoria Karlsruhe",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="18.",
                participant=Participant(
                    name_one="Marcel Rose",
                    name_two="Imke Schwan",
                    number=530,
                    club="TC Kristall Jena",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="19.- 21.",
                participant=Participant(
                    name_one="Carsten Beck",
                    name_two="Jennifer Arnold",
                    number=501,
                    club="TC Rot-Weiss Casino Mainz",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="19.- 21.",
                participant=Participant(
                    name_one="Markus Hajek",
                    name_two="Eva Hajek",
                    number=515,
                    club="TC Rot-Weiss Casino Mainz",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="19.- 21.",
                participant=Participant(
                    name_one="Patrick Keller",
                    name_two="Stefanie Schenker",
                    number=518,
                    club="TSC dancepoint, Königsbrunn",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="22.- 23.",
                participant=Participant(
                    name_one="Stefan Fischer",
                    name_two="Mareike Maass",
                    number=511,
                    club="Rot-Weiß-Club Gießen",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="22.- 23.",
                participant=Participant(
                    name_one="Jakob Zwicker",
                    name_two="Magdalena Bedner",
                    number=536,
                    club="Tanzclub Konstanz",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="24.",
                participant=Participant(
                    name_one="Alexander Behmer",
                    name_two="Dr. Juliane Scheil",
                    number=502,
                    club="Tanzsportzentrum Wetter-Ruhr",
                ),
            ),
        ],
    ),
    ResultRound(
        name="Vorrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="25.",
                participant=Participant(
                    name_one="David Schneider",
                    name_two="Sonja Wendenburg",
                    number=533,
                    club="TSZ Blau-Gold Casino, Darmstadt",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="26.- 28.",
                participant=Participant(
                    name_one="Andreas Baumeister",
                    name_two="Cäcilia Benzin",
                    number=500,
                    club="TSC Astoria Karlsruhe",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="26.- 28.",
                participant=Participant(
                    name_one="Achim Besler",
                    name_two="Kathrin Besler",
                    number=504,
                    club="TSA d. TSG 1861 Grünstadt",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="26.- 28.",
                participant=Participant(
                    name_one="Paul Wehle",
                    name_two="Melanie Höschele",
                    number=535,
                    club="Tanzsportclub Balance Berlin",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="29.- 30.",
                participant=Participant(
                    name_one="Pascal Gerbert",
                    name_two="Nelly Gerbert",
                    number=512,
                    club="TSC Welfen Weingarten",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="29.- 30.",
                participant=Participant(
                    name_one="Dirk Schäfer",
                    name_two="Gertrud Lembke",
                    number=531,
                    club="TSZ Blau-Gold Casino, Darmstadt",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="31.",
                participant=Participant(
                    name_one="Alexander Kober",
                    name_two="Diana Bühren",
                    number=520,
                    club="Tanzsportclub Dortmund",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="32.",
                participant=Participant(
                    name_one="Martin Erhardt",
                    name_two="Svenja Mozian",
                    number=510,
                    club="Tanz- u. Sportzentr. Mittelrhein, Koblenz",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="33.",
                participant=Participant(
                    name_one="Patrick Hiebl",
                    name_two="Sylvia Kißmehl",
                    number=517,
                    club="WTC Friedberg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="34.",
                participant=Participant(
                    name_one="Christian Deike",
                    name_two="Patrycja Krohn",
                    number=507,
                    club="1. TSZ im Turn-Klubb zu Hannover",
                ),
            ),
        ],
    ),
]

EXPECTED_ERG_RESULTS_53 = [
    ResultRound(
        name="Endrunde",
        placings=[
            FinalRoundPlacing(
                rank="1.",
                participant=Participant(
                    name_one="Maximilian Beichter",
                    name_two="Melissa Hagel",
                    number=401,
                    club="TSC Astoria Karlsruhe",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(
                        marks=[2, 1, 1, 1, 1, 1, 4], place=1.0
                    ),
                    Dance.TANGO: DanceScore(marks=[2, 2, 1, 1, 2, 1, 3], place=1.0),
                    Dance.VIENNESE_WALTZ: DanceScore(
                        marks=[2, 2, 1, 1, 2, 1, 3], place=1.0
                    ),
                    Dance.SLOW_FOXTROT: DanceScore(
                        marks=[1, 1, 1, 1, 2, 1, 2], place=1.0
                    ),
                    Dance.QUICKSTEP: DanceScore(marks=[2, 2, 1, 1, 2, 1, 2], place=1.0),
                },
                total_score=5.0,
            ),
            FinalRoundPlacing(
                rank="2.",
                participant=Participant(
                    name_one="Leif-Erik Montag",
                    name_two="Johanna Wille",
                    number=421,
                    club="Tanzsportteam im ASC Göttingen v. 1846",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(
                        marks=[1, 2, 4, 4, 3, 3, 1], place=2.0
                    ),
                    Dance.TANGO: DanceScore(marks=[1, 1, 3, 5, 3, 4, 1], place=2.0),
                    Dance.VIENNESE_WALTZ: DanceScore(
                        marks=[1, 1, 4, 4, 5, 2, 1], place=2.0
                    ),
                    Dance.SLOW_FOXTROT: DanceScore(
                        marks=[2, 2, 4, 4, 3, 2, 1], place=2.0
                    ),
                    Dance.QUICKSTEP: DanceScore(marks=[1, 1, 3, 4, 3, 3, 1], place=2.0),
                },
                total_score=10.0,
            ),
            FinalRoundPlacing(
                rank="3.",
                participant=Participant(
                    name_one="Marcus Nguyen Ngoc",
                    name_two="Lea Teßmer",
                    number=423,
                    club="Club Céronne im ETV Hamburg",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(
                        marks=[6, 3, 6, 2, 2, 4, 5], place=3.0
                    ),
                    Dance.TANGO: DanceScore(marks=[5, 3, 5, 2, 1, 3, 5], place=3.0),
                    Dance.VIENNESE_WALTZ: DanceScore(
                        marks=[6, 3, 6, 3, 1, 3, 6], place=3.0
                    ),
                    Dance.SLOW_FOXTROT: DanceScore(
                        marks=[6, 3, 6, 3, 1, 4, 5], place=3.0
                    ),
                    Dance.QUICKSTEP: DanceScore(marks=[5, 3, 5, 2, 1, 2, 5], place=3.0),
                },
                total_score=15.0,
            ),
            FinalRoundPlacing(
                rank="4.",
                participant=Participant(
                    name_one="Patrick Dahm",
                    name_two="Sandra Schwarz",
                    number=408,
                    club="TC Der Frankfurter Kreis",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(
                        marks=[5, 5, 5, 3, 4, 2, 3], place=4.0
                    ),
                    Dance.TANGO: DanceScore(marks=[4, 5, 6, 3, 4, 2, 4], place=4.0),
                    Dance.VIENNESE_WALTZ: DanceScore(
                        marks=[5, 5, 5, 2, 3, 4, 2], place=4.0
                    ),
                    Dance.SLOW_FOXTROT: DanceScore(
                        marks=[4, 5, 5, 2, 5, 3, 3], place=4.0
                    ),
                    Dance.QUICKSTEP: DanceScore(marks=[4, 5, 6, 3, 4, 4, 4], place=4.0),
                },
                total_score=20.0,
            ),
            FinalRoundPlacing(
                rank="5.",
                participant=Participant(
                    name_one="Christopher Buchloh-Rosenthal",
                    name_two="Analena Koch",
                    number=405,
                    club="Rot-Weiss-Klub Kassel",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(
                        marks=[3, 4, 3, 5, 5, 5, 6], place=5.0
                    ),
                    Dance.TANGO: DanceScore(marks=[3, 4, 4, 6, 5, 6, 6], place=6.0),
                    Dance.VIENNESE_WALTZ: DanceScore(
                        marks=[3, 4, 3, 5, 4, 5, 5], place=5.0
                    ),
                    Dance.SLOW_FOXTROT: DanceScore(
                        marks=[3, 4, 3, 6, 4, 5, 6], place=5.0
                    ),
                    Dance.QUICKSTEP: DanceScore(marks=[3, 4, 2, 6, 5, 5, 6], place=5.0),
                },
                total_score=26.0,
            ),
            FinalRoundPlacing(
                rank="6.",
                participant=Participant(
                    name_one="Jakob Hinz",
                    name_two="Vivien Bachmann",
                    number=415,
                    club="TC Kristall Jena",
                ),
                dance_scores={
                    Dance.SLOW_WALTZ: DanceScore(
                        marks=[4, 6, 2, 6, 6, 6, 2], place=6.0
                    ),
                    Dance.TANGO: DanceScore(marks=[6, 6, 2, 4, 6, 5, 2], place=5.0),
                    Dance.VIENNESE_WALTZ: DanceScore(
                        marks=[4, 6, 2, 6, 6, 6, 4], place=6.0
                    ),
                    Dance.SLOW_FOXTROT: DanceScore(
                        marks=[5, 6, 2, 5, 6, 6, 4], place=6.0
                    ),
                    Dance.QUICKSTEP: DanceScore(marks=[6, 6, 4, 5, 6, 6, 3], place=6.0),
                },
                total_score=29.0,
            ),
        ],
    ),
    ResultRound(
        name="2. Zwischenrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="7.",
                participant=Participant(
                    name_one="Mariusz Budek",
                    name_two="Marta Budek",
                    number=406,
                    club="TSC Villingen-Schwenningen",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="8.",
                participant=Participant(
                    name_one="Christoph Hanisch",
                    name_two="Kaja Zoé Pfüller",
                    number=412,
                    club="UTSC Choice Styria",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="9.",
                participant=Participant(
                    name_one="Sebastian Hellmann",
                    name_two="Melanie Oberhauser",
                    number=414,
                    club="TTC Oldenburg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="10.",
                participant=Participant(
                    name_one="Oliver Neumann",
                    name_two="Anna-Maria Ehinger",
                    number=422,
                    club="TSC Astoria Karlsruhe",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="11.- 12.",
                participant=Participant(
                    name_one="Ruslan Wellner",
                    name_two="Tabea Kilian",
                    number=430,
                    club="Braunschweig Dance Company",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="11.- 12.",
                participant=Participant(
                    name_one="Leo Werner",
                    name_two="Fabienne Theobalt",
                    number=432,
                    club="TC Rot-Weiss Casino Mainz",
                ),
            ),
        ],
    ),
    ResultRound(
        name="1. Zwischenrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="13.- 14.",
                participant=Participant(
                    name_one="Uli Kunz",
                    name_two="Saskia Morcinczyk",
                    number=420,
                    club="TSC Grün-Gold Speyer",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="13.- 14.",
                participant=Participant(
                    name_one="Martin Wenhart",
                    name_two="Lisa Harrell",
                    number=431,
                    club="TSC dancepoint, Königsbrunn",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="15.",
                participant=Participant(
                    name_one="René Kaczorowski",
                    name_two="Cindy Hebert",
                    number=417,
                    club="Tanzsportverein Schwarz-Weiß Freiberg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="16.",
                participant=Participant(
                    name_one="Benedikt Ernst",
                    name_two="Tanja Esche",
                    number=411,
                    club="TSC Rot-Gold-Casino Nürnberg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="17.",
                participant=Participant(
                    name_one="Fabian Beckmann",
                    name_two="Katrin Langert",
                    number=400,
                    club="TSC Schwarz-Gelb Aachen",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="18.- 19.",
                participant=Participant(
                    name_one="Patrick Rach",
                    name_two="Lorena Kimmel",
                    number=427,
                    club="TC Rot-Weiss Casino Mainz",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="18.- 19.",
                participant=Participant(
                    name_one="Enrico Weber",
                    name_two="Anne-Kathrin Nitt-Weber",
                    number=429,
                    club="Tanzsportzentrum Dresden",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="20.",
                participant=Participant(
                    name_one="Max Kirchenberger",
                    name_two="Friederike Rust",
                    number=418,
                    club="TTC München",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="21.- 22.",
                participant=Participant(
                    name_one="Johannes Kreim",
                    name_two="Rebecca Gonzalez-Ringer",
                    number=419,
                    club="TC Rot-Weiss Casino Mainz",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="21.- 22.",
                participant=Participant(
                    name_one="Patrik Pollak",
                    name_two="Pia Feischen",
                    number=426,
                    club="TSC Grün-Gold Heidelberg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="23.- 24.",
                participant=Participant(
                    name_one="Christopher Brix",
                    name_two="Sandra Kuhfus",
                    number=403,
                    club="TSC Grün-Weiß Aquisgrana Aachen",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="23.- 24.",
                participant=Participant(
                    name_one="Michael Wrusch",
                    name_two="Dan Feng Tian Helena Liang",
                    number=434,
                    club="OTK Schwarz-Weiß 1922 im SCS Berlin",
                ),
            ),
        ],
    ),
    ResultRound(
        name="Vorrunde",
        placings=[
            PreliminaryRoundPlacing(
                rank="25.- 26.",
                participant=Participant(
                    name_one="Dr. Michel Oelschlägel",
                    name_two="Dagmar Lina Ganz",
                    number=424,
                    club="Tanzsportverein Schwarz-Weiß Freiberg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="25.- 26.",
                participant=Participant(
                    name_one="Matty Schiller",
                    name_two="Anne Wienhold",
                    number=428,
                    club="TSA d. TTC Allround Rostock",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="27.- 28.",
                participant=Participant(
                    name_one="Christoph Hellings",
                    name_two="Maria Korosteleva",
                    number=413,
                    club="Switzerland",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="27.- 28.",
                participant=Participant(
                    name_one="Michael Hopf",
                    name_two="Iris Hopf",
                    number=416,
                    club="TSC Unterschleißheim",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="29.",
                participant=Participant(
                    name_one="Thomas Brunnengräber",
                    name_two="Mirjam Tittlus",
                    number=404,
                    club="TC Blau-Orange Wiesbaden",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="30.",
                participant=Participant(
                    name_one="Robert Podgajny",
                    name_two="Olesya Oshchepkova",
                    number=425,
                    club="TTC Rot-Weiß Freiburg",
                ),
            ),
            PreliminaryRoundPlacing(
                rank="31.",
                participant=Participant(
                    name_one="Florian Dammeyer",
                    name_two="Yasmin Deborah Gers",
                    number=409,
                    club="Die Residenz Münster",
                ),
            ),
        ],
    ),
]


@pytest.mark.parametrize(
    "filepath, expected_results",
    [
        (
            "tests/51-1105_ot_hgr2dstd/erg.htm",
            EXPECTED_ERG_RESULTS_51,
        ),
        (
            "tests/52-1105_ot_hgr2cstd/erg.htm",
            EXPECTED_ERG_RESULTS_52,
        ),
        (
            "tests/53-1105_ot_hgr2bstd/erg.htm",
            EXPECTED_ERG_RESULTS_53,
        ),
    ],
)
def test_parse_erg_file_and_compare_with_true_values(filepath, expected_results):
    with open(filepath, "r", encoding="utf-8") as f:
        html_content = f.read()

    # Parse the file
    parsed_results = extract_results_from_erg(html_content)

    # Compare the extracted information against the true values
    assert parsed_results == expected_results


@pytest.mark.parametrize(
    "filepath, title",
    [
        ("tests/51-1105_ot_hgr2dstd/erg.htm", "11.05.2024 Hgr.II D Standard"),
        ("tests/52-1105_ot_hgr2cstd/erg.htm", "11.05.2024 Hgr.II C Standard"),
        ("tests/53-1105_ot_hgr2bstd/erg.htm", "11.05.2024 Hgr.II B Standard"),
    ],
)
def test_regenerate_erg_html_and_compare_with_original(filepath, title):
    with open(filepath, "r", encoding="utf-8") as f:
        html_content = f.read()

    # Parse the file to get the data structure
    parsed_results = extract_results_from_erg(html_content)

    # Regenerate the HTML from the data structure
    regenerated_html = generate_erg_html(parsed_results, title=title)

    # Canonicalize both the original and the regenerated HTML
    canonicalized_original = canonicalize_html(html_content)
    canonicalized_regenerated = canonicalize_html(regenerated_html)

    # Compare the canonicalized versions
    assert canonicalized_regenerated == canonicalized_original, (
        "Regenerated HTML does not match the canonicalized original. "
        "This indicates a potential issue in the HTML generation logic or the canonicalization process."
    )