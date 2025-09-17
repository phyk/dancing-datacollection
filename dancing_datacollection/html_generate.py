from typing import List, Dict, Tuple, Optional
from html import escape

from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.score import FinalRoundScore, GERMAN_TO_ENGLISH_DANCE_NAME


def _html_page(title: str, body: str) -> str:
    return (
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\">"
        f"<title>{escape(title)}</title>"
        "</head><body>" + body + "</body></html>"
    )


def generate_deck_html(judges: List[Judge], title: str = "deck") -> str:
    # Build a second table with judge rows matching deck expectations
    rows = []
    for j in judges:
        code = escape(j.code or "")
        name = escape(j.name or "")
        club = escape(j.club or "")
        row = (
            "<tr>"
            f"<td class=\"td2r\">{code}:</td>"
            f"<td><span>{name}</span><span>{club}</span></td>"
            "</tr>"
        )
        rows.append(row)
    table2 = "<table>" + "".join(rows) + "</table>"
    body = "<table></table>" + table2
    return _html_page(title, body)


def generate_tabges_html(participants: List[Participant], judges: List[Judge], title: str = "tabges") -> str:
    # Minimal structure: one table class tab1 with Wertungsrichter block, and scattered td2gc cells for participants
    judge_lines = []
    for j in judges:
        line = f"{escape((j.code or '')[:2])}) {escape(j.name or '')}"
        judge_lines.append(line)
    # Judges section
    header = (
        "<tr><td>Wertungsrichter:</td></tr>"
        "<tr><td class=\"td3\">" + "<br>".join(judge_lines) + "</td></tr>"
    )
    table = "<table class=\"tab1\">" + header + "</table>"

    # Participant cells
    part_cells = []
    for p in participants:
        num = escape(str(p.number) if p.number is not None else "")
        names = escape(
            f"{p.name_one or ''} / {p.name_two or ''}".strip().strip("/").strip()
        )
        cell = (
            f"<td class=\"td2gc\">{num}<span class=\"tooltip2gc\">{names}</span></td>"
        )
        part_cells.append(cell)
    parts_table = "<table>" + "<tr>" + "".join(part_cells) + "</tr></table>"
    return _html_page(title, table + parts_table)


def generate_erg_html(participants: List[Participant], title: str = "erg") -> str:
    rows = []
    # Header rows minimal
    rows.append("<tr><th>Platz</th><th>Paar/Club</th><th>Nr</th></tr>")
    for p in participants:
        rank_str = escape(" ".join(str(r) for r in (p.ranks or [])))
        names = f"{p.name_one or ''}"
        if p.name_two:
            names += f" / {p.name_two}"
        names = names.strip()
        number = f"({p.number})" if p.number is not None else ""
        club_html = f"<i>{escape(p.club or '')}</i>" if p.club else ""
        names_cell = f"{escape(names)} {escape(number)} {club_html}".strip()
        nr_cell = escape(str(p.number) if p.number is not None else "")
        row = (
            "<tr>"
            f"<td>{rank_str}</td>"
            f"<td>{names_cell}</td>"
            f"<td>{nr_cell}</td>"
            "</tr>"
        )
        rows.append(row)
    table = "<table class=\"tab1\">" + "".join(rows) + "</table>"
    return _html_page(title, table)


def _group_scores(final_scores: List[FinalRoundScore]) -> Tuple[List[str], Dict[str, List[str]]]:
    """Return (ordered_dances_english, judge_codes_per_dance) in a deterministic layout.

    - Use stable dance order preference (Standard first): LW, TG, QS, WW, SF, then others alphabetical
    - Restrict to first 3 dances available
    - Exactly 6 judge codes per dance; if fewer, pad with last seen code or 'A'
    """
    per_dance: Dict[str, List[str]] = {}
    for s in final_scores:
        per_dance.setdefault(s.dance_name, [])
        if s.judge_code not in per_dance[s.dance_name]:
            per_dance[s.dance_name].append(s.judge_code)

    # Stable preferred order mapped from English names
    preferred = [
        "SlowWaltz",  # LW
        "Tango",      # TG
        "Quickstep",  # QS
        "VienneseWaltz",  # WW
        "SlowFoxtrott",   # SF
    ]
    others = sorted(name for name in per_dance.keys() if name not in preferred)
    ordered = [d for d in preferred if d in per_dance] + others
    dances = ordered[:3]

    normalized: Dict[str, List[str]] = {}
    for d in dances:
        codes = per_dance.get(d, [])
        if not codes:
            normalized[d] = ["A", "B", "C", "D", "E", "F"]
        else:
            six = list(codes[:6])
            if len(six) < 6:
                filler = six[-1] if six else "A"
                six += [filler] * (6 - len(six))
            normalized[d] = six
    return dances, normalized


def _english_to_german_abbrev(name: str) -> str:
    # Find a German key mapping to the English name, prefer abbreviations if present
    for german, eng in GERMAN_TO_ENGLISH_DANCE_NAME.items():
        if eng == name and len(german) <= 3:
            return german
    # fallback to known common abbreviations
    return name[:2].upper()


def generate_ergwert_html(
    participants: List[Participant],
    judges: List[Judge],
    final_scores: List[FinalRoundScore],
    title: str = "ergwert",
    dance_names_english: Optional[List[str]] = None,
    judge_codes_per_dance: Optional[Dict[str, List[str]]] = None,
) -> str:
    # Prepare headers
    if dance_names_english is not None and judge_codes_per_dance is not None:
        dances = list(dance_names_english)
        per_dance = dict(judge_codes_per_dance)
    else:
        dances, per_dance = _group_scores(final_scores)
    # Header row 0: after first 4 cells, put dance abbrev cells
    header0_cells = ["<th>Platz</th>", "<th>Paar/Club</th>", "<th>Nr</th>", "<th>R</th>"]
    for d in dances:
        header0_cells.append(f"<th>{escape(_english_to_german_abbrev(d))}</th>")
    header0 = "<tr>" + "".join(header0_cells) + "</tr>"

    # Header row 1: judge codes per dance separated by Su
    header1_cells = []
    for idx, d in enumerate(dances):
        codes = per_dance.get(d, [])
        for code in codes:
            header1_cells.append(f"<td>{escape(code)}</td>")
        if idx < len(dances) - 1:
            header1_cells.append("<td>Su</td>")
    header1 = "<tr>" + "".join(header1_cells) + "</tr>"

    # Map from (number, dance, judge_code) to score
    score_map: Dict[Tuple[int, str, str], int] = {}
    for s in final_scores:
        score_map[(s.number, s.dance_name, s.judge_code)] = s.score

    # Participant rows
    rows = []
    for p in participants:
        cls0 = "td3cv"
        platz = escape(str((p.ranks or [""])[0]))
        names = f"{p.name_one or ''}"
        if p.name_two:
            names += f" / {p.name_two}"
        names_cell = escape(names)
        nr_cell = escape(str(p.number) if p.number is not None else "")
        # Build score cells layout matching parser expectations
        score_cells = []
        base_index = 4
        for d in dances:
            for code in per_dance.get(d, []):
                val = ""
                if p.number is not None:
                    key = (p.number, d, code)
                    if key in score_map:
                        val = str(score_map[key])
                score_cells.append(f"<td>{escape(val)}</td>")
        row_html = (
            "<tr>"
            f"<td class=\"{cls0}\">{platz}</td>"
            f"<td>{names_cell}</td>"
            f"<td>{nr_cell}</td>"
            f"<td></td>"  # R column placeholder
            + "".join(score_cells)
            + "</tr>"
        )
        rows.append(row_html)

    table = "<table class=\"tab1\">" + header0 + header1 + "".join(rows) + "</table>"
    return _html_page(title, table)


