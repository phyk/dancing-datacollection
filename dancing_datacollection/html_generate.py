from typing import List, Dict, Tuple, Optional
from html import escape

from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.score import FinalRoundScore, GERMAN_TO_ENGLISH_DANCE_NAME
from dancing_datacollection.data_defs.results import (
    ResultRound,
    FinalRoundPlacing,
    PreliminaryRoundPlacing,
)


def _html_page(title: str, body: str) -> str:
    return (
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\">"
        f"<title>{escape(title)}</title>"
        "</head><body>" + body + "</body></html>"
    )


from dancing_datacollection.data_defs.committee import CommitteeMember

ROLE_KEY_TO_GERMAN = {
    "organizer": "Veranstalter:",
    "host": "Ausrichter:",
    "chairperson": "Turnierleiter:",
    "committee_member": "Beisitzer:",
    "protocol": "Protokoll:",
}


def generate_deck_html(
    judges: List[Judge], committee: List[CommitteeMember], title: str = "deck"
) -> str:
    # The title string in the first table is slightly different from the page title.
    # This transformation seems to work for the sample data.
    table_title = title.replace(" OT,", " - OT,")
    title_table = f"<table><tr><td>{escape(table_title)}</td></tr></table>"

    # Main table with committee and judges
    main_rows = []
    for member in committee:
        role_german = ROLE_KEY_TO_GERMAN.get(member.role or "", member.role or "")
        name = escape(member.name or "")
        club = escape(member.club or "")
        # This logic is based on reverse-engineering the golden files.
        # Some roles have spans even without a club.
        if member.role == "protocol":
            content = f"<span>{name}</span>"
        elif club:
            content = f"<span>{name}</span><span>{club}</span>"
        else:
            content = name
        row = f"<tr><td>{escape(role_german)}</td><td>{content}</td></tr>"
        main_rows.append(row)

    # Judges header
    main_rows.append("<tr><td>Wertungsrichter:</td></tr>")

    # Judge rows
    for j in judges:
        code = escape(j.code or "")
        raw_name = j.name or ""
        # The golden files expect "Last, First" format.
        if ", " not in raw_name and " " in raw_name:
            parts = raw_name.rsplit(" ", 1)
            name = f"{parts[1]}, {parts[0]}"
        else:
            name = raw_name
        escaped_name = escape(name)
        club = escape(j.club or "")
        row = (
            "<tr>"
            f"<td>{code}:</td>"
            f"<td><span>{escaped_name}</span><span>{club}</span></td>"
            "</tr>"
        )
        main_rows.append(row)

    main_table = "<table>" + "".join(main_rows) + "</table>"

    body = (
        "<div>"
        f"<div>{title_table}</div>"
        "<div>"
        "<div>Deckblatt</div>"
        "<hr/>"
        f"<div>{main_table}<br/></div>"
        "</div>"
        "</div>"
    )
    return _html_page(title, body)


def generate_tabges_html(tables_data: List[List[List[str]]], title: str = "tabges") -> str:
    table_title = title.replace(" Hgr.", " - OT, Hgr.")
    title_table = f"<table><tr><td>{escape(table_title)}</td></tr></table>"

    tables_html = []
    for table_data in tables_data:
        rows_html = []
        for row_data in table_data:
            cells_html = [f"<td>{cell_content}</td>" for cell_content in row_data]
            rows_html.append("<tr>" + "".join(cells_html) + "</tr>")
        tables_html.append("<table>" + "".join(rows_html) + "</table>")

    main_content = "<br/>".join(tables_html)

    body = (
        "<div>"
        f"<div>{title_table}</div>"
        "<div>"
        "<div>Wertungstabelle Gesamt</div>"
        "<hr/>"
        f"<div>{main_content}<br/><br/></div>"
        "</div>"
        "</div>"
    )
    return _html_page(title, body)


def generate_erg_html(results: List[ResultRound], title: str = "erg") -> str:
    table_title = title.replace(" Hgr.", " - OT, Hgr.")
    title_table = f"<table><tr><td>{escape(table_title)}</td></tr></table>"

    final_round = next((r for r in results if isinstance(r.placings[0], FinalRoundPlacing)), None)
    preliminary_rounds = [r for r in results if isinstance(r.placings[0], PreliminaryRoundPlacing)]

    tables_html = []

    # Generate final round table
    if final_round:
        rows_html = [f"<tr><td>{final_round.name}</td></tr>"]
        dance_names = list(final_round.placings[0].dance_scores.keys())
        header_cells = ["<td>Platz</td>", "<td>Paar/Club</td>"] + [
            f"<td>{dn}</td>" for dn in dance_names
        ] + ["<td>PZ</td>"]
        rows_html.append("<tr>" + "".join(header_cells) + "</tr>")
        for p in final_round.placings:
            cells = [f"<td>{p.rank}</td>"]
            name_html = f"{escape(p.participant.name_one or '')}"
            if p.participant.name_two:
                name_html += f" / {escape(p.participant.name_two)}"
            name_html += f" ({p.participant.number})<br/><i>{escape(p.participant.club or '')}</i>"
            cells.append(f"<td>{name_html}</td>")
            for dn in dance_names:
                ds = p.dance_scores[dn]
                cells.append(f"<td>{ds.marks}<br/><div>{ds.place}</div></td>")
            cells.append(f"<td><br/>{p.total_score}</td>")
            rows_html.append("<tr>" + "".join(cells) + "</tr>")
        tables_html.append("<table>" + "".join(rows_html) + "</table>")

    # Generate one table for all preliminary rounds
    if preliminary_rounds:
        prelim_rows_html = []
        for result_round in preliminary_rounds:
            prelim_rows_html.append(f"<tr><td>{result_round.name}</td></tr>")
            for p in result_round.placings:
                cells = [f"<td>{p.rank}</td>"]
                name_html = f"{escape(p.participant.name_one or '')}"
                if p.participant.name_two:
                    name_html += f" / {escape(p.participant.name_two)}"
                name_html += f" ({p.participant.number})"
                cells.append(f"<td>{name_html}</td>")
                cells.append(f"<td>{escape(p.participant.club or '')}</td>")
                prelim_rows_html.append("<tr>" + "".join(cells) + "</tr>")
        tables_html.append("<table>" + "".join(prelim_rows_html) + "</table>")

    main_content = "".join(tables_html)

    body = (
        "<div>"
        f"<div>{title_table}</div>"
        "<div>"
        "<div>Ergebnis</div>"
        "<hr/>"
        f"<div>{main_content}<br/></div>"
        "</div>"
        "</div>"
    )
    return _html_page(title, body)


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
    tables_data: List[List[List[str]]], title: str = "ergwert"
) -> str:
    # The first table in ergwert.htm is the title table, which we can regenerate.
    table_title = title.replace(" Hgr.", " - OT, Hgr.")
    title_table = f"<table><tr><td>{escape(table_title)}</td></tr></table>"

    # The main content is the second table.
    main_table_data = tables_data[1]
    rows_html = []
    for row_data in main_table_data:
        cells_html = [f"<td>{cell_content}</td>" for cell_content in row_data]
        rows_html.append("<tr>" + "".join(cells_html) + "</tr>")
    main_table_html = "<table>" + "".join(rows_html) + "</table>"

    main_content = (
        "<div>Zum Ausklappen der Einzelwertungen bitte auf die Tanzspalte drücken.</div>"
        + main_table_html
    )

    body = (
        "<div>"
        f"<div>{title_table}</div>"
        "<div>"
        "<div>Ergebnis mit Wertung</div>"
        "<hr/>"
        f"<div>{main_content}<br/></div>"
        "</div>"
        "</div>"
    )
    return _html_page(title, body)


