from enum import Enum
from typing import List, Optional

def run_scraper(config_path: str) -> None: ...

class Level(Enum):
    E = 0
    D = 1
    C = 2
    B = 3
    A = 4
    S = 5

class Style(Enum):
    Standard = 0
    Latein = 1

class Dance(Enum):
    SlowWaltz = 0
    Tango = 1
    VienneseWaltz = 2
    SlowFoxtrot = 3
    Quickstep = 4
    Samba = 5
    ChaChaCha = 6
    Rumba = 7
    PasoDoble = 8
    Jive = 9

class AgeGroup(Enum):
    Juv1 = 0
    Juv2 = 1
    Jun1 = 2
    Jun2 = 3
    Youth = 4
    Adult = 5
    Adult2 = 6
    Sen1 = 7
    Sen2 = 8
    Sen3 = 9
    Sen4 = 10
    Sen5 = 11
    Senior = 12
    @staticmethod
    def from_german(name: str) -> Optional["AgeGroup"]: ...

class IdentityType(Enum):
    Solo = 0
    Couple = 1

class Judge:
    code: str
    name: str
    club: Optional[str]
    def __init__(self, code: str, name: str, club: Optional[str] = None) -> None: ...

class CommitteeMember:
    name: str
    club: Optional[str]
    def __init__(self, name: str, club: Optional[str] = None) -> None: ...

class Officials:
    responsible_person: Optional[CommitteeMember]
    assistant: Optional[CommitteeMember]
    judges: List[Judge]
    def __init__(self, responsible_person: Optional[CommitteeMember], assistant: Optional[CommitteeMember], judges: List[Judge]) -> None: ...

class Participant:
    identity_type: IdentityType
    name_one: str
    name_two: Optional[str]
    affiliation: Optional[str]
    bib_number: int
    final_rank: Optional[int]
    def __init__(self, identity_type: IdentityType, name_one: str, bib_number: int, name_two: Optional[str] = None, affiliation: Optional[str] = None, final_rank: Optional[int] = None) -> None: ...

class WDSFScore:
    technical_quality: float
    movement_to_music: float
    partnering_skills: float
    choreography: float
    def __init__(self, technical_quality: float, movement_to_music: float, partnering_skills: float, choreography: float) -> None: ...

class Round:
    name: str
    marking_crosses: Optional[Dict[str, Dict[int, Dict[Dance, bool]]]]
    dtv_ranks: Optional[Dict[str, Dict[int, Dict[Dance, int]]]]
    wdsf_scores: Optional[Dict[str, Dict[int, WDSFScore]]]
    def __init__(self, name: str, marking_crosses: Optional[Dict[str, Dict[int, Dict[Dance, bool]]]] = None, dtv_ranks: Optional[Dict[str, Dict[int, Dict[Dance, int]]]] = None, wdsf_scores: Optional[Dict[str, Dict[int, WDSFScore]]] = None) -> None: ...

class Competition:
    level: Level
    age_group: AgeGroup
    style: Style
    dances: List[Dance]
    officials: Officials
    participants: List[Participant]
    rounds: List[Round]
    def __init__(self, level: Level, age_group: AgeGroup, style: Style, dances: List[Dance], officials: Officials, participants: List[Participant], rounds: List[Round]) -> None: ...

class Event:
    name: str
    organizer: Optional[str]
    hosting_club: Optional[str]
    competitions_list: List[Competition]
    def __init__(self, name: str, organizer: Optional[str], hosting_club: Optional[str], competitions_list: List[Competition]) -> None: ...
