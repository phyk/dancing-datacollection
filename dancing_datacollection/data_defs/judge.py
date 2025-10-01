from typing import Optional, Any
import re
from pydantic import (
    BaseModel,
    field_validator,
    model_validator,
    ConfigDict,
)


class Judge(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    code: str
    name: str
    club: Optional[str] = None

    @field_validator("code", mode="before")
    @classmethod
    def _normalize_code(cls, v: Any) -> str:
        if not isinstance(v, str):
            return v
        normalized_code = v.strip().upper()
        if not (1 <= len(normalized_code) <= 3 and normalized_code.isalpha()):
            raise ValueError(f"Judge code must be 1-3 letters, got '{v}'")
        return normalized_code

    @field_validator("name", mode="before")
    @classmethod
    def _normalize_name(cls, v: Any) -> str:
        if not isinstance(v, str):
            return v
        name = v.strip()
        m = re.match(r"^([^,]+),\s*(.+)$", name)
        if m:
            last = m.group(1).strip()
            first = m.group(2).strip()
            name = f"{first} {last}"
        name = re.sub(r"\s+", " ", name)
        if len(name.split()) < 2:
            raise ValueError(
                f"Judge name must contain at least two words, got '{v}'"
            )
        return name

    @field_validator("club", mode="before")
    @classmethod
    def _normalize_club(cls, v: Any) -> Optional[str]:
        if v is None:
            return None
        if not isinstance(v, str):
            return v
        club = v.strip()
        club = re.sub(r"\s+", " ", club)
        if club == "":
            return None
        return club

    @model_validator(mode="after")
    def _validate_club_and_name(self) -> "Judge":
        if self.club and self.name:
            if self.club in self.name or self.name in self.club:
                raise ValueError(
                    f"Club and name must not be substrings of each other: name='{self.name}', club='{self.club}'"
                )
        return self

    def __str__(self) -> str:
        return f"Judge(code='{self.code}', name='{self.name}', club='{self.club}')"

    def matches_partial(self, other: "Judge") -> bool:
        """Return True if code and name are identical."""
        return self.code == other.code and self.name == other.name

    def matches_full(self, other: "Judge") -> bool:
        """Return True if code, name, and club are identical."""
        return (
            self.code == other.code
            and self.name == other.name
            and self.club == other.club
        )