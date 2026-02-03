import re
from typing import Optional

from pydantic import (
    BaseModel,
    ConfigDict,
    field_validator,
    model_validator,
)


class Judge(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    code: str
    first_name: str
    last_name: str
    club: Optional[str] = None

    @property
    def name(self) -> str:
        return f"{self.first_name} {self.last_name}"

    @property
    def name_last_first(self) -> str:
        return f"{self.last_name}, {self.first_name}"

    @field_validator("code")
    @classmethod
    def _normalize_code(cls, v: str) -> str:
        normalized_code = v.strip().upper()
        if not (1 <= len(normalized_code) <= 3 and normalized_code.isalpha()):
            msg = f"Judge code must be 1-3 letters, got '{v}'"
            raise ValueError(msg)
        return normalized_code

    @field_validator("first_name", "last_name")
    @classmethod
    def _normalize_name_parts(cls, v: str) -> str:
        return v.strip()

    @field_validator("club")
    @classmethod
    def _normalize_club(cls, v: str) -> Optional[str]:
        if v is None:
            return None
        club = v.strip()
        club = re.sub(r"\s+", " ", club)
        if club == "":
            return None
        return club

    @model_validator(mode="after")
    def _validate_club_and_name(self) -> "Judge":
        if self.club and self.name and (self.club in self.name or self.name in self.club):
            msg = f"Club and name must not be substrings of each other: name='{self.name}', club='{self.club}'"
            raise ValueError(msg)
        return self

    def __str__(self) -> str:
        return f"Judge(code='{self.code}', name='{self.name}', club='{self.club}')"

    def matches_partial(self, other: "Judge") -> bool:
        """Return True if code and name are identical."""
        return self.code == other.code and self.name == other.name

    def matches_full(self, other: "Judge") -> bool:
        """Return True if code, name, and club are identical."""
        return self.code == other.code and self.name == other.name and self.club == other.club
