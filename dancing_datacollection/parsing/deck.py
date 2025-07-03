from dancing_datacollection.data_defs.judge import Judge
from typing import List
import logging


def extract_judges_from_deck(soup) -> List[Judge]:
    """
    Extract judges from deck.htm using the annotated structure:
    Table 1, rows 6-10: cell 0 is judge code (remove colon), cell 1 is 'Last, First Club'.
    Parse name as last name, first name, and the rest as club.
    Return a list of Judge dataclasses with code, name, club.
    """
    logger = logging.getLogger("parsing_debug")
    tables = soup.find_all("table")
    judges = []
    if len(tables) < 2:
        logger.warning("Expected at least 2 tables in deck.htm")
        return judges
    rows = tables[1].find_all("tr")
    for row in rows:
        cells = row.find_all(["td", "th"])
        if len(cells) < 2:
            continue
        # Only process rows where the first cell has class 'td2r' (judge rows)
        if "td2r" in (cells[0].get("class") or []):
            code = cells[0].get_text(strip=True).replace(":", "")
            # Use spans to extract name and club
            spans = cells[1].find_all("span")
            if len(spans) >= 2:
                name_raw = (
                    spans[0]
                    .get_text(strip=True)
                    .replace("\xa0", "")
                    .replace("\u00a0", "")
                    .strip()
                )
                club = (
                    spans[1]
                    .get_text(strip=True)
                    .replace("\xa0", "")
                    .replace("\u00a0", "")
                    .strip()
                )
                if "," in name_raw:
                    last, first = [x.strip() for x in name_raw.split(",", 1)]
                    name = f"{first} {last}"
                else:
                    name = name_raw
            else:
                name = cells[1].get_text(strip=True)
                club = ""
            logger.debug(f"  Judge: code={code}, name={name}, club={club}")
            try:
                judge = Judge(code=code, name=name, club=club)
                judges.append(judge)
            except Exception as e:
                logger.warning(
                    f"Invalid judge skipped: code={code}, name={name}, club={club}, error={e}"
                )
    # Deduplicate by (code, name)
    unique = {}
    for j in judges:
        key = (j.code, j.name)
        if key not in unique:
            unique[key] = j
    return list(unique.values())
