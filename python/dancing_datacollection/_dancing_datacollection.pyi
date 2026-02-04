from typing import Any, List, Optional
import datetime

class Level: ...
class Style: ...
class Dance: ...
class AgeGroup: ...
class IdentityType: ...

class Judge:
    code: str
    name: str
    club: Optional[str]

class CommitteeMember:
    name: str
    club: Optional[str]

class Officials:
    responsible_person: Optional[CommitteeMember]
    assistant: Optional[CommitteeMember]
    judges: List[Judge]

class Participant:
    identity_type: IdentityType
    name_one: str
    bib_number: int
    name_two: Optional[str]
    affiliation: Optional[str]
    final_rank: Optional[int]

class WDSFScore:
    technical_quality: float
    movement_to_music: float
    partnering_skills: float
    choreography: float

class Round:
    name: str
    marking_crosses: Any
    dtv_ranks: Any
    wdsf_scores: Any

class Competition:
    level: Level
    age_group: AgeGroup
    style: Style
    dances: List[Dance]
    min_dances: int
    officials: Officials
    participants: List[Participant]
    rounds: List[Round]

class Event:
    name: str
    date: Optional[datetime.date]
    organizer: Optional[str]
    hosting_club: Optional[str]
    competitions_list: List[Competition]

class StorageManager:
    def __init__(self, base_path: str) -> None: ...
    def save_event(self, event: Event) -> None: ...

def download_sources(config_path: str) -> None: ...
def extract_competitions(data_dir: str) -> Event: ...
def validate_extracted_competitions(event: Event) -> bool: ...
def collect_dancing_data(config_path: str) -> List[Event]: ...
