from pydantic import BaseModel, Field
from .dances import Dance


class FinalRoundScore(BaseModel, frozen=True, extra='forbid'):
    number: int
    score: int
    round_number: int
    judge_code: str
    dance_name: Dance

    def partial_match(self, other: "FinalRoundScore") -> bool:
        return (
            isinstance(other, self.__class__)
            and self.number == other.number
            and self.judge_code == other.judge_code
            and self.dance_name == other.dance_name
        )


class Score(BaseModel, frozen=True, extra='forbid'):
    number: int
    score: bool
    round_number: int
    judge_code: str
    dance_name: Dance

    def partial_match(self, other: "Score") -> bool:
        return (
            isinstance(other, self.__class__)
            and self.number == other.number
            and self.judge_code == other.judge_code
            and self.dance_name == other.dance_name
        )