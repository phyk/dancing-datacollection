from enum import Enum
from typing import Optional

from dancing_datacollection.config import get_age_groups


class AgeGroup(str, Enum):
    JUV_1 = "juv_1"
    JUV_2 = "juv_2"
    JUN_1 = "jun_1"
    JUN_2 = "jun_2"
    YOUTH = "youth"
    ADULT = "adult"
    ADULT_2 = "adult_2"
    SEN_1 = "sen_1"
    SEN_2 = "sen_2"
    SEN_3 = "sen_3"
    SEN_4 = "sen_4"
    SEN_5 = "sen_5"
    SENIOR = "senior" # Added for generic senior

    @classmethod
    def from_german(cls, german_name: str) -> Optional["AgeGroup"]:
        age_groups_config = get_age_groups()
        if german_name in age_groups_config:
            return cls(age_groups_config[german_name]["id"])
        return None

    def to_english(self) -> str:
        age_groups_config = get_age_groups()
        for german, info in age_groups_config.items():
            if info["id"] == self.value and "english" in info:
                return info["english"]
        return self.value
