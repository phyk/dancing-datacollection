from enum import Enum


class Dance(str, Enum):
    CHA_CHA = "ChaCha"
    SAMBA = "Samba"
    RUMBA = "Rumba"
    PASO_DOBLE = "PasoDoble"
    JIVE = "Jive"
    SLOW_WALTZ = "SlowWaltz"
    TANGO = "Tango"
    VIENNESE_WALTZ = "VienneseWaltz"
    SLOW_FOXTROT = "SlowFoxtrott"
    QUICKSTEP = "Quickstep"


# Mapping from German/abbreviated names to allowed English names
GERMAN_TO_ENGLISH_DANCE_NAME = {
    "Langsamer Walzer": Dance.SLOW_WALTZ,
    "LW": Dance.SLOW_WALTZ,
    "Wiener Walzer": Dance.VIENNESE_WALTZ,
    "WW": Dance.VIENNESE_WALTZ,
    "Tango": Dance.TANGO,
    "TG": Dance.TANGO,
    "Quickstep": Dance.QUICKSTEP,
    "QS": Dance.QUICKSTEP,
    "QU": Dance.QUICKSTEP,
    "Slow Foxtrott": Dance.SLOW_FOXTROT,
    "Slowfox": Dance.SLOW_FOXTROT,
    "Foxtrott": Dance.SLOW_FOXTROT,
    "SF": Dance.SLOW_FOXTROT,
    "Samba": Dance.SAMBA,
    "SB": Dance.SAMBA,
    "Cha Cha Cha": Dance.CHA_CHA,
    "ChaChaCha": Dance.CHA_CHA,
    "CC": Dance.CHA_CHA,
    "Rumba": Dance.RUMBA,
    "RB": Dance.RUMBA,
    "Paso Doble": Dance.PASO_DOBLE,
    "PD": Dance.PASO_DOBLE,
    "Jive": Dance.JIVE,
    "JV": Dance.JIVE,
}

ENGLISH_TO_GERMAN_DANCE_NAME = {
    Dance.SLOW_WALTZ: "LW",
    Dance.VIENNESE_WALTZ: "WW",
    Dance.TANGO: "TG",
    Dance.QUICKSTEP: "QU",
    Dance.SLOW_FOXTROT: "SF",
    Dance.SAMBA: "SB",
    Dance.CHA_CHA: "CC",
    Dance.RUMBA: "RB",
    Dance.PASO_DOBLE: "PD",
    Dance.JIVE: "JV",
}
