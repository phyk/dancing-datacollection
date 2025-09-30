import urllib.request
from bs4 import BeautifulSoup
from urllib.parse import urljoin
import logging
import os
import re
from typing import Any, List, Optional


def setup_logging(log_dir=None):
    """
    Set up logging for the application. Configures:
    - Root logger (INFO to app.log and console)
    - Error logger (ERROR to error.log)
    - Parsing debug logger (DEBUG to parsing_debug.log)
    Call this once at program start or before any logging is used.
    """
    if log_dir is None:
        log_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), "logs")
    os.makedirs(log_dir, exist_ok=True)
    app_log_path = os.path.join(log_dir, "app.log")
    error_log_path = os.path.join(log_dir, "error.log")
    parsing_debug_path = os.path.join(log_dir, "parsing_debug.log")

    # Root logger
    root_logger = logging.getLogger()
    if not any(
        isinstance(h, logging.FileHandler)
        and getattr(h, "baseFilename", None) == app_log_path
        for h in root_logger.handlers
    ):
        root_logger.setLevel(logging.INFO)
        app_handler = logging.FileHandler(app_log_path, mode="w")
        app_handler.setLevel(logging.INFO)
        app_handler.setFormatter(
            logging.Formatter("%(asctime)s %(levelname)s %(message)s")
        )
        root_logger.addHandler(app_handler)
        console_handler = logging.StreamHandler()
        console_handler.setLevel(logging.INFO)
        console_handler.setFormatter(
            logging.Formatter("%(asctime)s %(levelname)s %(message)s")
        )
        root_logger.addHandler(console_handler)

    # Error logger
    error_logger = logging.getLogger("error")
    if not any(
        isinstance(h, logging.FileHandler)
        and getattr(h, "baseFilename", None) == error_log_path
        for h in error_logger.handlers
    ):
        error_logger.setLevel(logging.ERROR)
        error_handler = logging.FileHandler(error_log_path, mode="w")
        error_handler.setLevel(logging.ERROR)
        error_handler.setFormatter(
            logging.Formatter("%(asctime)s %(levelname)s %(message)s")
        )
        error_logger.addHandler(error_handler)
        error_logger.propagate = False

    # Parsing debug logger
    parsing_logger = logging.getLogger("parsing_debug")
    if not any(
        isinstance(h, logging.FileHandler)
        and getattr(h, "baseFilename", None) == parsing_debug_path
        for h in parsing_logger.handlers
    ):
        parsing_logger.setLevel(logging.DEBUG)
        parsing_handler = logging.FileHandler(parsing_debug_path, mode="w")
        parsing_handler.setLevel(logging.DEBUG)
        parsing_handler.setFormatter(
            logging.Formatter("%(asctime)s %(levelname)s %(message)s")
        )
        parsing_logger.addHandler(parsing_handler)
        parsing_logger.propagate = False
    parsing_logger.debug("TEST: parsing_debug logger setup complete")


def download_html(url):
    try:
        logging.info(f"Downloading: {url}")
        with urllib.request.urlopen(url) as response:
            html = response.read().decode("utf-8")
        logging.info(f"Downloaded {len(html)} characters from {url}")
        return html
    except Exception as e:
        logging.error(f"Failed to download {url}: {e}")
        return None


def extract_competition_links(html, base_url):
    soup = BeautifulSoup(html, "html.parser")
    links = []
    for a in soup.find_all("a", href=True):
        href = a["href"]
        if href.endswith(".htm") or href.endswith(".html"):
            full_url = urljoin(base_url, href)
            links.append(full_url)
    return links


def deduplicate_participants(participants):
    seen = set()
    unique = []
    for p in participants:
        key = (p.get("number"), p.get("names"), p.get("club"))
        if key not in seen:
            seen.add(key)
            unique.append(p)
    return unique


def get_soup(html):
    """Return a BeautifulSoup object for the given HTML."""
    return BeautifulSoup(html, "html.parser")


def extract_club_and_number(cell):
    """Extract club (from <i>) and number (from (number) in text) from a table cell."""
    club_tag = cell.find("i")
    club = club_tag.get_text(strip=True) if club_tag else None
    text = cell.get_text(" ", strip=True)
    number_match = re.search(r"\((\d+)\)", text)
    number = number_match.group(1) if number_match else None
    return club, number


def split_names(names):
    """Split names string into name_one and name_two using common delimiters or whitespace."""
    for delim in [" / ", " & ", " und ", " and "]:
        if delim in names:
            parts = names.split(delim)
            if len(parts) == 2:
                return parts[0].strip(), parts[1].strip()
    # Fallback: try splitting on whitespace
    parts = names.split()
    if len(parts) >= 2:
        return parts[0].strip(), " ".join(parts[1:]).strip()
    return None, None


def extract_name_and_club_from_spans(cell):
    """Extract name and club from <span> tags in a cell, or fallback to cell text."""
    spans = cell.find_all("span")
    name = ""
    club = ""
    if len(spans) >= 2:
        name = spans[0].get_text(strip=True)
        club = spans[1].get_text(strip=True)
    elif len(spans) == 1:
        name = spans[0].get_text(strip=True)
    else:
        name = cell.get_text(strip=True)
    return name, club


# ---------- Helper utilities for bs4 parsing and deduplication ----------

def as_class_list(classes: Any) -> List[str]:
    """Normalize a bs4 'class' attribute to a list of strings."""
    if isinstance(classes, list):
        return [str(c) for c in classes]
    if isinstance(classes, str):
        return [classes]
    return []


def element_has_class(element: Any, class_name: str) -> bool:
    """Return True if a bs4 element has the given class name, robust to None/str/list."""
    return class_name in as_class_list(getattr(element, 'attrs', {}).get('class')) if hasattr(element, 'attrs') else False


def first_line_text(element: Any) -> str:
    """Return the first logical line of text from a cell/tag."""
    lines = element.get_text(separator="\n", strip=True).splitlines() if hasattr(element, 'get_text') else []
    return lines[0] if lines else ""


def deduplicate_judges(judges: List["Judge"]) -> List["Judge"]:
    """Deduplicate judges by (code, name). Prefer entries with a non-empty club."""
    best_by_key: dict[tuple[str, str], "Judge"] = {}
    for j in judges:
        key = (getattr(j, 'code', ''), getattr(j, 'name', ''))
        if key not in best_by_key:
            best_by_key[key] = j
        else:
            current = best_by_key[key]
            current_club = getattr(current, 'club', None) or ""
            new_club = getattr(j, 'club', None) or ""
            if new_club and not current_club:
                best_by_key[key] = j
    return list(best_by_key.values())


def extract_event_name_from_soup(soup: BeautifulSoup) -> str:
    """Extracts and sanitizes the event name from the <title> tag of a BeautifulSoup object."""
    title_tag = soup.find("title")
    event_name = title_tag.get_text(strip=True) if title_tag else "unknown_event"
    # Sanitize the event name to be used as a directory/file name
    event_name = re.sub(r"[^\w\d-]+", "_", event_name)[:64]
    return event_name
