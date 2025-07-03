from dataclasses import dataclass, field
from typing import Optional, List
import re

@dataclass(frozen=True, eq=True)
class Participant:
    name_one: Optional[str] = field(default=None)
    name_two: Optional[str] = field(default=None)
    number: Optional[int] = field(default=None)
    ranks: Optional[List[int]] = field(default=None)
    club: Optional[str] = field(default=None)

    def __post_init__(self):
        # Normalize names: strip and collapse spaces
        if self.name_one is not None:
            name_one = re.sub(r'\s+', ' ', self.name_one.strip())
            object.__setattr__(self, 'name_one', name_one)
        if self.name_two is not None:
            name_two = re.sub(r'\s+', ' ', self.name_two.strip())
            object.__setattr__(self, 'name_two', name_two)
        # Normalize club: strip and collapse spaces
        if self.club is not None:
            club = re.sub(r'\s+', ' ', self.club.strip())
            if club == '':
                club = None
            object.__setattr__(self, 'club', club)
        else:
            object.__setattr__(self, 'club', None)
        # Normalize number: ensure int or None
        if self.number is not None:
            try:
                number = int(self.number)
            except Exception:
                number = None
            object.__setattr__(self, 'number', number)
        # Normalize ranks: ensure list of int or None
        ranks = self.ranks
        if ranks is not None:
            if isinstance(ranks, str):
                ranks = self._parse_ranks(ranks)
            elif isinstance(ranks, int):
                ranks = [ranks]
            elif isinstance(ranks, list):
                ranks = [int(r) for r in ranks if r is not None]
            else:
                ranks = None
            object.__setattr__(self, 'ranks', ranks)
        # Validation: at least name_one and number must be present
        if not self.name_one or self.number is None:
            raise ValueError(f"Participant must have name_one and number. Got: name_one={self.name_one}, number={self.number}")

    @staticmethod
    def _parse_ranks(rank_str):
        # Accept formats like '8.- 9.', '1.', '12', etc.
        if not rank_str:
            return None
        # Find all integer numbers in the string
        nums = re.findall(r'\d+', rank_str)
        return [int(n) for n in nums] if nums else None

    def verify(self) -> bool:
        if not self.name_one or self.number is None:
            return False
        return True

    def __str__(self):
        return f"Participant(name_one='{self.name_one}', name_two='{self.name_two}', number={self.number}, ranks={self.ranks}, club='{self.club}')" 