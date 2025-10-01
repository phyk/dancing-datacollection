from typing import Dict, Optional

from pydantic import BaseModel, ConfigDict

from .dances import Dance


class FinalScoring(BaseModel):
    """Represents a final scoring entry."""
    model_config = ConfigDict(frozen=True, extra='forbid')

    placement: Optional[str] = None
    names: Optional[str] = None
    number: Optional[str] = None
    club: Optional[str] = None
    scores: Dict[Dance, str] = {}
    total: Optional[str] = None