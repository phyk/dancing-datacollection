from typing import Optional

from pydantic import BaseModel, ConfigDict


from typing import List


class CommitteeMember(BaseModel):
    """Represents a member of the tournament committee."""

    model_config = ConfigDict(frozen=True, extra="forbid")
    role: Optional[str] = None
    name: Optional[str] = None
    club: Optional[str] = None


class Committee(BaseModel):
    """Represents a tournament committee."""

    model_config = ConfigDict(frozen=True, extra="forbid")
    members: List[CommitteeMember]
