from pydantic import BaseModel, ConfigDict
from typing import List, Optional, Dict, Union
from .participant import Participant
from .dances import Dance


class DanceScore(BaseModel):
    """Represents the scores for a single dance in a final round."""
    model_config = ConfigDict(frozen=True, extra='forbid')
    marks: str
    place: float


class FinalRoundPlacing(BaseModel):
    """Represents a single placing in a final round table."""
    model_config = ConfigDict(frozen=True, extra='forbid')
    rank: str
    participant: Participant
    dance_scores: Dict[Dance, DanceScore]
    total_score: float


class PreliminaryRoundPlacing(BaseModel):
    """Represents a single placing in a preliminary round table."""
    model_config = ConfigDict(frozen=True, extra='forbid')
    rank: str
    participant: Participant


class ResultRound(BaseModel):
    """Represents the results for a single round of a competition."""
    model_config = ConfigDict(frozen=True, extra='forbid')
    name: str
    placings: List[Union[FinalRoundPlacing, PreliminaryRoundPlacing]]