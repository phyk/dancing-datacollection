from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class FinalScoring:
    """Represents a final scoring entry."""
    placement: Optional[str] = None
    names: Optional[str] = None
    number: Optional[str] = None
    club: Optional[str] = None
    score_LW: Optional[str] = None
    score_TG: Optional[str] = None
    score_QS: Optional[str] = None
    total: Optional[str] = None