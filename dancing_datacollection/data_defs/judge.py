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
    name: str
    club: Optional[str] = None

    @field_validator("code")
    @classmethod
    def _normalize_code(cls, v: str) -> str:
        normalized_code = v.strip().upper()
        if not (1 <= len(normalized_code) <= 3 and normalized_code.isalpha()):
            msg = f"Judge code must be 1-3 letters, got '{v}'"
            raise ValueError(msg)
        return normalized_code

    @field_validator("name")
    @classmethod
    def _normalize_name(cls, v: str) -> str:
        name = v.strip()
        m = re.match(r"^([^,]+),\s*(.+)$", name)
        if m:
            last = m.group(1).strip()
            first = m.group(2).strip()
            name = f"{first} {last}"
        name = re.sub(r"\s+", " ", name)
        if len(name.split()) < 2:
            msg = f"Judge name must contain at least two words, got '{v}'"
            raise ValueError(msg)
        return name

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
