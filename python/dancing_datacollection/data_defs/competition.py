import datetime
from typing import Optional

from pydantic import BaseModel, ConfigDict

from .age_group import AgeGroup
from .discipline import Discipline
from .level import Level


class CompetitionInfo(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    name: str
    comp_date: Optional[datetime.date] = None
    age_group: Optional[AgeGroup] = None
    discipline: Optional[Discipline] = None
    level: Optional[Level] = None

    @property
    def min_dances(self) -> int:
        if self.level:
            return self.level.get_min_dances(self.comp_date)
        return 0
