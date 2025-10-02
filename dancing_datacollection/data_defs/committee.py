from typing import List, Optional

from pydantic import BaseModel, ConfigDict


class CommitteeMember(BaseModel):
    """Represents a member of the tournament committee."""

    model_config = ConfigDict(frozen=True, extra="forbid")
    role: Optional[str] = None
    first_name: Optional[str] = None
    last_name: Optional[str] = None
    club: Optional[str] = None

    @property
    def name(self) -> Optional[str]:
        """Returns the full name of the committee member."""
        if self.first_name and self.last_name:
            return f"{self.first_name} {self.last_name}"
        return self.first_name

    @property
    def name_last_first(self) -> Optional[str]:
        """Returns the name in 'Last, First' format if applicable."""
        if self.first_name and self.last_name:
            return f"{self.last_name}, {self.first_name}"
        return self.first_name


class Committee(BaseModel):
    """Represents a tournament committee."""

    model_config = ConfigDict(frozen=True, extra="forbid")
    members: List[CommitteeMember]
