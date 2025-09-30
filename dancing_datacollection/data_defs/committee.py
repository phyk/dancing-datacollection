from dataclasses import dataclass
from typing import Optional

@dataclass(frozen=True)
class CommitteeMember:
    """Represents a member of the tournament committee."""
    role: Optional[str] = None
    name: Optional[str] = None
    club: Optional[str] = None