from dataclasses import dataclass, field
from typing import Optional
import re

@dataclass(frozen=True, eq=True)
class Judge:
    code: str
    name: str
    club: Optional[str] = field(default=None)

    def __post_init__(self):
        # Normalize code: uppercase and strip whitespace
        object.__setattr__(self, 'code', self.code.strip().upper())
        # Normalize name: strip, collapse spaces, convert 'Lastname, Firstname' to 'Firstname Lastname'
        name = self.name.strip()
        # If name is in 'Lastname, Firstname' format, convert to 'Firstname Lastname'
        m = re.match(r'^([^,]+),\s*(.+)$', name)
        if m:
            # Allow multiple first or last names: everything before first comma is last name, after is first name(s)
            last = m.group(1).strip()
            first = m.group(2).strip()
            name = f"{first} {last}"
        # Collapse multiple spaces
        name = re.sub(r'\s+', ' ', name)
        object.__setattr__(self, 'name', name)
        # Normalize club: strip and collapse spaces
        if self.club is not None:
            club = self.club.strip()
            club = re.sub(r'\s+', ' ', club)
            if club == '':
                club = None
            object.__setattr__(self, 'club', club)
        else:
            object.__setattr__(self, 'club', None)
        # Validate code: 1-3 letters
        if not (1 <= len(self.code) <= 3 and self.code.isalpha()):
            raise ValueError(f"Judge code must be 1-3 letters, got '{self.code}'")
        # Validate name: at least two words
        if len(self.name.split()) < 2:
            raise ValueError(f"Judge name must contain at least two words, got '{self.name}'")
        # Validate club: allow empty or any string
        if self.club and (self.club in self.name or self.name in self.club):
            raise ValueError(f"Club and name must not be substrings of each other: name='{self.name}', club='{self.club}'")

    def verify(self) -> bool:
        # Code: 1-3 letters
        if not (1 <= len(self.code) <= 3 and self.code.isalpha()):
            return False
        # Name: at least two words
        if len(self.name.split()) < 2:
            return False
        # Club and name must not be substrings of each other
        if self.club and (self.club in self.name or self.name in self.club):
            return False
        return True

    def __str__(self):
        return f"Judge(code='{self.code}', name='{self.name}', club='{self.club}')"

    def matches_partial(self, other: 'Judge') -> bool:
        """Return True if code and name are identical."""
        return self.code == other.code and self.name == other.name

    def matches_full(self, other: 'Judge') -> bool:
        """Return True if code, name, and club are identical."""
        return self.code == other.code and self.name == other.name and self.club == other.club 