from datetime import date
from enum import Enum
from typing import Optional

from dancing_datacollection.config import get_levels


class Level(str, Enum):
    E = "E"
    D = "D"
    C = "C"
    B = "B"
    A = "A"
    S = "S"

    def get_min_dances(self, competition_date: Optional[date] = None) -> int:
        levels_config = get_levels()
        config = levels_config.get(self.value, {})

        if "min_dances" in config:
            return config["min_dances"]

        is_2026_or_later = False
        if competition_date and competition_date.year >= 2026:
            is_2026_or_later = True

        if is_2026_or_later:
            return config.get("min_dances_2026", config.get("min_dances_legacy", 0))
        else:
            return config.get("min_dances_legacy", 0)
