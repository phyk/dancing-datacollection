import re
from typing import Any, List, Optional, Union

from pydantic import BaseModel, ConfigDict, field_validator


class Participant(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    name_one: str
    number: int
    name_two: Optional[str] = None
    ranks: Optional[List[int]] = None
    club: Optional[str] = None

    @field_validator("name_one", "name_two", "club")
    @classmethod
    def _normalize_str(cls, v: Optional[str]) -> Optional[str]:
        if v is None:
            return None
        normalized = re.sub(r"\s+", " ", v.strip())
        return normalized or None

    @field_validator("ranks", mode="before")
    @classmethod
    def _parse_ranks(cls, v: Optional[Union[str, int, List[Any]]]) -> Optional[List[int]]:
        if v is None:
            return None

        if isinstance(v, str):
            nums = re.findall(r"\d+", v)
            return [int(n) for n in nums] if nums else None

        if isinstance(v, int):
            return [v]

        if isinstance(v, list):
            return [int(r) for r in v if r is not None]

        return v

    def matches_partial(self, other: "Participant") -> bool:
        """Return True if number, name_one, and name_two match. Ignores club."""
        if not isinstance(other, Participant):
            return False
        return (
            self.number == other.number
            and self.name_one == other.name_one
            and self.name_two == other.name_two
        )

    def matches_full(self, other: "Participant") -> bool:
        """Return True if number, name_one, name_two, club, and ranks all match."""
        if not isinstance(other, Participant):
            return False
        return (
            self.number == other.number
            and self.name_one == other.name_one
            and self.name_two == other.name_two
            and self.club == other.club
            and self.ranks == other.ranks
        )
