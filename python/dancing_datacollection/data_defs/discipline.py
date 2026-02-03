from enum import Enum
from typing import Optional

from dancing_datacollection.config import get_disciplines


class Discipline(str, Enum):
    BALLROOM = "std"
    LATIN = "lat"

    @classmethod
    def from_german(cls, german_name: str) -> Optional["Discipline"]:
        disciplines_config = get_disciplines()
        if german_name in disciplines_config:
            return cls(disciplines_config[german_name]["id"])

        # Common abbreviations
        if german_name == "Std":
            return cls.BALLROOM
        if german_name == "Lat":
            return cls.LATIN

        return None

    def to_english(self) -> str:
        disciplines_config = get_disciplines()
        for german, info in disciplines_config.items():
            if info["id"] == self.value:
                return info["english"]
        return self.value
