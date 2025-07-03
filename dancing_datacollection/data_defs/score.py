from dataclasses import dataclass

ALLOWED_DANCE_NAMES = {'SlowWaltz', 'VienneseWaltz', 'Tango', 'Quickstep', 'SlowFoxtrott', 'Samba', 'ChaCha', 'Rumba', 'PasoDoble', 'Jive'}

# Mapping from German/abbreviated names to allowed English names
GERMAN_TO_ENGLISH_DANCE_NAME = {
    'Langsamer Walzer': 'SlowWaltz',
    'LW': 'SlowWaltz',
    'Wiener Walzer': 'VienneseWaltz',
    'WW': 'VienneseWaltz',
    'Tango': 'Tango',
    'TG': 'Tango',
    'Quickstep': 'Quickstep',
    'QS': 'Quickstep',
    'Slow Foxtrott': 'SlowFoxtrott',
    'Foxtrott': 'SlowFoxtrott',
    'SF': 'SlowFoxtrott',
    'Samba': 'Samba',
    'SB': 'Samba',
    'Cha Cha Cha': 'ChaCha',
    'ChaChaCha': 'ChaCha',
    'CC': 'ChaCha',
    'Rumba': 'Rumba',
    'RB': 'Rumba',
    'Paso Doble': 'PasoDoble',
    'PD': 'PasoDoble',
    'Jive': 'Jive',
    'JV': 'Jive',
}

@dataclass(frozen=True, eq=True)
class FinalRoundScore:
    number: int
    score: int
    round_name: str
    round_number: int
    judge_code: str
    dance_name: str

    def __post_init__(self):
        if self.dance_name not in ALLOWED_DANCE_NAMES:
            raise ValueError(f"Invalid dance_name: {self.dance_name}. Must be one of {ALLOWED_DANCE_NAMES}")

    def __repr__(self):
        return (f"FinalRoundScore(number={self.number!r}, score={self.score!r}, "
                f"round_name={self.round_name!r}, round_number={self.round_number!r}, "
                f"judge_code={self.judge_code!r}, dance_name={self.dance_name!r})")

    def partial_match(self, other):
        return (
            isinstance(other, self.__class__)
            and self.number == other.number
            and self.judge_code == other.judge_code
            and self.round_name == other.round_name
            and self.dance_name == other.dance_name
        )

@dataclass(frozen=True, eq=True)
class Score:
    number: int
    score: bool
    round_name: str
    round_number: int
    judge_code: str
    dance_name: str

    def __post_init__(self):
        if self.dance_name not in ALLOWED_DANCE_NAMES:
            raise ValueError(f"Invalid dance_name: {self.dance_name}. Must be one of {ALLOWED_DANCE_NAMES}")

    def __repr__(self):
        return (f"Score(number={self.number!r}, score={self.score!r}, "
                f"round_name={self.round_name!r}, round_number={self.round_number!r}, "
                f"judge_code={self.judge_code!r}, dance_name={self.dance_name!r})")

    def partial_match(self, other):
        return (
            isinstance(other, self.__class__)
            and self.number == other.number
            and self.judge_code == other.judge_code
            and self.round_name == other.round_name
            and self.dance_name == other.dance_name
        ) 