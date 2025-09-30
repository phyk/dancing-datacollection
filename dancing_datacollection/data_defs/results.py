from dataclasses import dataclass
from typing import List, Optional, Dict, Union
from .participant import Participant

@dataclass(frozen=True)
class DanceScore:
    """Represents the scores for a single dance in a final round."""
    marks: str
    place: float

@dataclass(frozen=True)
class FinalRoundPlacing:
    """Represents a single placing in a final round table."""
    rank: str
    participant: Participant
    dance_scores: Dict[str, DanceScore]
    total_score: float

@dataclass(frozen=True)
class PreliminaryRoundPlacing:
    """Represents a single placing in a preliminary round table."""
    rank: str
    participant: Participant

@dataclass(frozen=True)
class ResultRound:
    """Represents the results for a single round of a competition."""
    name: str
    placings: List[Union[FinalRoundPlacing, PreliminaryRoundPlacing]]