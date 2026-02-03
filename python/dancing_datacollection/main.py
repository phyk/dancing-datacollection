# Orchestration logic for the dancing datacollection project
import argparse
import logging
import os
import urllib.parse
import urllib.robotparser
from typing import Any, Dict, List

from tqdm import tqdm

from dancing_datacollection.config import load_config
from dancing_datacollection.output import (
    save_committee,
    save_competition_data,
    save_final_scoring,
    save_judges,
    save_scores,
)
from dancing_datacollection.parsing.deck import (
    extract_committee_from_deck,
    extract_judges_from_deck,
)
from dancing_datacollection.parsing.ergwert import (
    extract_final_scoring,
    extract_scores_from_ergwert,
)
from dancing_datacollection.parsing.parsing_utils import (
    deduplicate_participants,
    download_html,
    extract_competition_links,
    get_soup,
    setup_logging,
)
from dancing_datacollection.parsing.participant import (
    extract_participants_and_event_name,
)

# Unified logging setup
setup_logging()

CONFIG_PATH = os.path.join(os.path.dirname(os.path.dirname(__file__)), "config.toml")
LOG_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "logs")

logger = logging.getLogger(__name__)
error_logger = logging.getLogger("error")


def is_allowed_by_robots(url: str, user_agent: str = "*") -> bool:
    parsed = urllib.parse.urlparse(url)
    robots_url = f"{parsed.scheme}://{parsed.netloc}/robots.txt"
    rp = urllib.robotparser.RobotFileParser()
    try:
        rp.set_url(robots_url)
        rp.read()
        allowed = rp.can_fetch(user_agent, url)
        if not allowed:
            logging.warning(
                "robots.txt at %s disallows access to %s for user-agent %s",
                robots_url,
                url,
                user_agent,
            )
        return allowed
    except OSError as e:
        logging.error("Failed to check robots.txt at %s: %s", robots_url, e)
        return True


def process_local_dir(local_dir: str) -> None:
    all_participants: List[Any] = []
    event_name = os.path.basename(local_dir)
    comp_info = None
    judges: List[Any] = []
    committee: List[Any] = []
    scores: List[Any] = []
    final_scores: List[Any] = []
    deck_path = os.path.join(local_dir, "deck.htm")
    tabges_path = os.path.join(local_dir, "tabges.htm")
    ergwert_path = os.path.join(local_dir, "ergwert.htm")
    # Check for cancellation in deck.htm
    if os.path.exists(deck_path):
        with open(deck_path, "r", encoding="utf-8") as f:
            deck_html = f.read()
        if ("abgesagt" in deck_html.lower()) or ("canceled" in deck_html.lower()):
            logging.info("Competition %s is canceled. Skipping all output files.", event_name)
            return
        soup = get_soup(deck_html)
        judges = extract_judges_from_deck(soup)
        logger.info("Judges found: %d", len(judges))
        save_judges(event_name, judges)
        committee = extract_committee_from_deck(soup)
        logger.info("Committee entries found: %d", len(committee))
        save_committee(event_name, committee)
    if os.path.exists(tabges_path):
        scores = []
        logger.info("Score entries found: %d", len(scores))
        save_scores(event_name, scores)
    if os.path.exists(ergwert_path):
        with open(ergwert_path, "r", encoding="utf-8") as f:
            ergwert_html = f.read()
        final_scores = extract_final_scoring(ergwert_html)
        logger.info("Final scoring entries found: %d", len(final_scores))
        save_final_scoring(event_name, final_scores)
        # Also extract scores from ergwert
        scores.extend(extract_scores_from_ergwert(get_soup(ergwert_html)))
        save_scores(event_name, scores)

    htm_files = []
    participants_by_file: Dict[str, set[Any]] = {}
    for root, _dirs, files in os.walk(local_dir):
        for fname in files:
            if fname.endswith(".htm"):
                htm_files.append(os.path.join(root, fname))
    if not htm_files:
        logger.warning("No .htm files found in %s.", local_dir)
    else:
        logger.info("Processing %d .htm files in %s...", len(htm_files), local_dir)
        for fpath in tqdm(htm_files, desc="Parsing .htm files", unit="file"):
            try:
                with open(fpath, "r", encoding="utf-8") as f:
                    html = f.read()
                participants, _, info = extract_participants_and_event_name(
                    html, os.path.basename(fpath)
                )
                if info and not comp_info:
                    comp_info = info
                if participants:
                    logger.info(
                        "  Participants found in %s: %d",
                        os.path.basename(fpath),
                        len(participants),
                    )
                    logger.debug(
                        "Participant numbers in %s: %s",
                        os.path.basename(fpath),
                        [p.number for p in participants if p.number],
                    )
                    participants_by_file[os.path.basename(fpath)] = {
                        p.number for p in participants if p.number
                    }
                    all_participants.extend(participants)
            except Exception:
                error_logger.exception("Error processing file %s", fpath)

    unique_participants = deduplicate_participants(all_participants)
    logger.info("Total unique participants in %s: %d", local_dir, len(unique_participants))
    logger.debug(
        "Unique participant numbers: %s",
        [p.number for p in unique_participants if p.number],
    )
    # Check for consistency of participant numbers across files
    if participants_by_file:
        all_sets = list(participants_by_file.values())
        base_set = all_sets[0]
        consistent = all(s == base_set for s in all_sets[1:])
        if not consistent:
            logger.warning(
                "Inconsistent participant numbers across files: %s",
                participants_by_file,
            )
        else:
            logger.info("Participant numbers are consistent across all files.")
    save_competition_data(event_name, unique_participants, comp_info)
    logger.info("Summary:")
    logger.info("  Judges: %d", len(judges))
    logger.info("  Committee: %d", len(committee))
    logger.info("  Scores: %d", len(scores))
    logger.info("  Final scoring: %d", len(final_scores))
    logger.info("  Unique participants: %d", len(unique_participants))


def main() -> None:
    arg_parser = argparse.ArgumentParser(description="Dancing Competition Data Collection")
    arg_parser.add_argument(
        "--local-dir",
        type=str,
        help="Process all .htm files in a local directory (for testing/offline)",
    )
    args = arg_parser.parse_args()
    if args.local_dir:
        process_local_dir(args.local_dir)
        return
    config = load_config()
    urls = config.get("sources", {}).get("urls", [])
    logger.info("Loaded %d base URLs from config.", len(urls))
    if not urls:
        logger.info("No URLs found in config. Exiting.")
        return
    for url in tqdm(urls, desc="Processing URLs", unit="url"):
        if not is_allowed_by_robots(url):
            logging.warning("robots.txt disallows scraping %s, skipping.", url)
            continue
        html = download_html(url)
        if html is not None:
            logger.info("Successfully downloaded HTML from %s", url)
            comp_links = extract_competition_links(html, url)
            logger.info("Found %d competition/event links at %s", len(comp_links), url)
            if not comp_links:
                logger.info("No competitions found at %s.", url)
                continue
            success_count = 0
            for link in tqdm(comp_links, desc="Downloading competitions", unit="comp"):
                try:
                    comp_html = download_html(link)
                    if comp_html is not None:
                        success_count += 1
                        base_url = link.rsplit("/", 1)[0]
                        erg_url = f"{base_url}/erg.htm"
                        erg_html = download_html(erg_url)
                        if erg_html:
                            participants, event_name, info = extract_participants_and_event_name(
                                erg_html, "erg.htm"
                            )
                            logger.info("Parsed competition (erg.htm): %s", erg_url)
                        else:
                            filename_from_link = link.rsplit("/", 1)[-1]
                            participants, event_name, info = extract_participants_and_event_name(
                                comp_html, filename_from_link
                            )
                            logger.info("Parsed competition (index): %s", link)
                        logger.info("  Participants: %d", len(participants))
                        save_competition_data(event_name, participants, info)
                        # After saving participants, try to download and save judges
                        deck_url = f"{base_url}/deck.htm"
                        deck_html = download_html(deck_url)
                        if deck_html:
                            soup = get_soup(deck_html)
                            judges = extract_judges_from_deck(soup)
                            logger.info("Judges found: %d", len(judges))
                            save_judges(event_name, judges)
                            committee = extract_committee_from_deck(soup)
                            logger.info("Committee entries found: %d", len(committee))
                            save_committee(event_name, committee)
                        tabges_url = f"{base_url}/tabges.htm"
                        tabges_html = download_html(tabges_url)
                        if tabges_html:
                            scores: List[Any] = []
                            logger.info("Score entries found: %d", len(scores))
                            save_scores(event_name, scores)
                        ergwert_url = f"{base_url}/ergwert.htm"
                        ergwert_html = download_html(ergwert_url)
                        if ergwert_html:
                            final_scores = extract_final_scoring(ergwert_html)
                            logger.info("Final scoring entries found: %d", len(final_scores))
                            save_final_scoring(event_name, final_scores)
                            scores = extract_scores_from_ergwert(get_soup(ergwert_html))
                            save_scores(event_name, scores)
                    else:
                        logger.warning("Failed to download competition page: %s", link)
                except Exception:
                    error_logger.exception("Error processing competition %s", link)
            logger.info(
                "Successfully downloaded %d/%d competition pages from %s",
                success_count,
                len(comp_links),
                url,
            )
            logger.info("Summary for this URL:")
            logger.info("  Competitions found: %d", len(comp_links))
            logger.info("  Competitions successfully processed: %d", success_count)
        else:
            logger.warning("Skipping %s due to download error.", url)
    logger.info("All URLs processed.")


if __name__ == "__main__":
    main()
