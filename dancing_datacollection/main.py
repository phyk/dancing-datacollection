# Orchestration logic for the dancing datacollection project
import argparse
import logging
import os
import toml
from tqdm import tqdm
from dancing_datacollection.parsing_utils import (
    download_html,
    extract_competition_links,
    deduplicate_participants,
    setup_logging,
    get_soup,
)
from dancing_datacollection.output import (
    save_competition_data,
    save_judges,
    save_committee,
    save_scores,
    save_final_scoring,
)
from dancing_datacollection.parsing.deck import (
    extract_judges_from_deck,
    extract_committee_from_deck,
)
from dancing_datacollection.parsing.ergwert import (
    extract_scores_from_ergwert,
    extract_final_scoring,
)
from dancing_datacollection.parsing.participant import (
    extract_participants_and_event_name,
)
import urllib.robotparser
import urllib.parse

# Unified logging setup
setup_logging()

CONFIG_PATH = os.path.join(os.path.dirname(os.path.dirname(__file__)), "config.toml")
LOG_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "logs")

logger = logging.getLogger(__name__)
error_logger = logging.getLogger("error")


def load_config():
    with open(CONFIG_PATH, "r") as f:
        return toml.load(f)


def is_allowed_by_robots(url, user_agent="*"):
    parsed = urllib.parse.urlparse(url)
    robots_url = f"{parsed.scheme}://{parsed.netloc}/robots.txt"
    rp = urllib.robotparser.RobotFileParser()
    try:
        rp.set_url(robots_url)
        rp.read()
        allowed = rp.can_fetch(user_agent, url)
        if not allowed:
            logging.warning(
                f"robots.txt at {robots_url} disallows access to {url} for user-agent {user_agent}"
            )
        return allowed
    except Exception as e:
        logging.error(f"Failed to check robots.txt at {robots_url}: {e}")
        return True


def process_local_dir(local_dir):
    all_participants = []
    event_name = os.path.basename(local_dir)
    judges = []
    committee = []
    scores = []
    final_scores = []
    deck_path = os.path.join(local_dir, "deck.htm")
    tabges_path = os.path.join(local_dir, "tabges.htm")
    ergwert_path = os.path.join(local_dir, "ergwert.htm")
    canceled = False
    # Check for cancellation in deck.htm
    if os.path.exists(deck_path):
        with open(deck_path, "r", encoding="utf-8") as f:
            deck_html = f.read()
        if ("abgesagt" in deck_html.lower()) or ("canceled" in deck_html.lower()):
            canceled = True
            logging.info(
                f"Competition {event_name} is canceled. Skipping all output files."
            )
            print(f"Competition {event_name} is canceled. Skipping all output files.")
            return
        soup = get_soup(deck_html)
        judges = extract_judges_from_deck(soup)
        logger.info(f"Judges found: {len(judges)}")
        print(f"Judges found: {len(judges)}")
        save_judges(event_name, judges)
        committee = extract_committee_from_deck(soup)
        logger.info(f"Committee entries found: {len(committee)}")
        print(f"Committee entries found: {len(committee)}")
        save_committee(event_name, committee)
    if os.path.exists(tabges_path):
        scores = []
        logger.info(f"Score entries found: {len(scores)}")
        print(f"Score entries found: {len(scores)}")
        save_scores(event_name, scores)
    if os.path.exists(ergwert_path):
        with open(ergwert_path, "r", encoding="utf-8") as f:
            ergwert_html = f.read()
        final_scores = extract_final_scoring(ergwert_html)
        logger.info(f"Final scoring entries found: {len(final_scores)}")
        print(f"Final scoring entries found: {len(final_scores)}")
        save_final_scoring(event_name, final_scores)
        # Also extract scores from ergwert
        scores.extend(extract_scores_from_ergwert(get_soup(ergwert_html)))
        save_scores(event_name, scores)

    htm_files = []
    participants_by_file = {}
    for root, dirs, files in os.walk(local_dir):
        for fname in files:
            if fname.endswith(".htm"):
                htm_files.append(os.path.join(root, fname))
    if not htm_files:
        print(f"No .htm files found in {local_dir}.")
    else:
        print(f"Processing {len(htm_files)} .htm files in {local_dir}...")
        for fpath in tqdm(htm_files, desc="Parsing .htm files", unit="file"):
            try:
                with open(fpath, "r", encoding="utf-8") as f:
                    html = f.read()
                participants, _ = extract_participants_and_event_name(
                    html, os.path.basename(fpath)
                )
                if participants:
                    logger.info(
                        f"  Participants found in {os.path.basename(fpath)}: {len(participants)}"
                    )
                    logger.debug(
                        f"Participant numbers in {os.path.basename(fpath)}: {[p.number for p in participants if p.number]}"
                    )
                    participants_by_file[os.path.basename(fpath)] = set(
                        p.number for p in participants if p.number
                    )
                    all_participants.extend(participants)
            except Exception:
                error_logger.error(f"Error processing file {fpath}", exc_info=True)
                print(f"Error processing file {fpath}. See error.log for details.")
    unique_participants = deduplicate_participants(all_participants)
    logger.info(f"Total unique participants in {local_dir}: {len(unique_participants)}")
    logger.debug(
        f"Unique participant numbers: {[p.number for p in unique_participants if p.number]}"
    )
    # Check for consistency of participant numbers across files
    if participants_by_file:
        all_sets = list(participants_by_file.values())
        base_set = all_sets[0]
        consistent = all(base_set == s for s in all_sets[1:])
        if not consistent:
            logger.warning(
                f"Inconsistent participant numbers across files: {participants_by_file}"
            )
        else:
            logger.info("Participant numbers are consistent across all files.")
    save_competition_data(event_name, unique_participants)
    print("Summary:")
    print(f"  Judges: {len(judges)}")
    print(f"  Committee: {len(committee)}")
    print(f"  Scores: {len(scores)}")
    print(f"  Final scoring: {len(final_scores)}")
    print(f"  Unique participants: {len(unique_participants)}")


def main():
    arg_parser = argparse.ArgumentParser(
        description="Dancing Competition Data Collection"
    )
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
    logger.info(f"Loaded {len(urls)} base URLs from config.")
    print(f"Checking {len(urls)} base URLs...")
    if not urls:
        print("No URLs found in config. Exiting.")
        return
    for url in tqdm(urls, desc="Processing URLs", unit="url"):
        if not is_allowed_by_robots(url):
            print(f"robots.txt disallows scraping {url}, skipping.")
            logging.warning(f"robots.txt disallows scraping {url}, skipping.")
            continue
        html = download_html(url)
        if html is not None:
            logger.info(f"Successfully downloaded HTML from {url}")
            comp_links = extract_competition_links(html, url)
            logger.info(f"Found {len(comp_links)} competition/event links at {url}")
            print(f"Found {len(comp_links)} competition/event links at {url}")
            if not comp_links:
                print(f"No competitions found at {url}.")
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
                            participants, event_name = (
                                extract_participants_and_event_name(
                                    erg_html, "erg.htm"
                                )
                            )
                            logger.info(f"Parsed competition (erg.htm): {erg_url}")
                        else:
                            filename_from_link = link.rsplit("/", 1)[-1]
                            participants, event_name = (
                                extract_participants_and_event_name(
                                    comp_html, filename_from_link
                                )
                            )
                            logger.info(f"Parsed competition (index): {link}")
                        logger.info(f"  Participants: {len(participants)}")
                        print(f"Parsed competition: {link}")
                        print(f"  Participants: {len(participants)}")
                        save_competition_data(event_name, participants)
                        # After saving participants, try to download and save judges
                        deck_url = f"{base_url}/deck.htm"
                        deck_html = download_html(deck_url)
                        if deck_html:
                            soup = get_soup(deck_html)
                            judges = extract_judges_from_deck(soup)
                            logger.info(f"Judges found: {len(judges)}")
                            print(f"Judges found: {len(judges)}")
                            save_judges(event_name, judges)
                            committee = extract_committee_from_deck(soup)
                            logger.info(f"Committee entries found: {len(committee)}")
                            print(f"Committee entries found: {len(committee)}")
                            save_committee(event_name, committee)
                        tabges_url = f"{base_url}/tabges.htm"
                        tabges_html = download_html(tabges_url)
                        if tabges_html:
                            scores = []
                            logger.info(f"Score entries found: {len(scores)}")
                            print(f"Score entries found: {len(scores)}")
                            save_scores(event_name, scores)
                        ergwert_url = f"{base_url}/ergwert.htm"
                        ergwert_html = download_html(ergwert_url)
                        if ergwert_html:
                            final_scores = extract_final_scoring(ergwert_html)
                            logger.info(
                                f"Final scoring entries found: {len(final_scores)}"
                            )
                            print(f"Final scoring entries found: {len(final_scores)}")
                            save_final_scoring(event_name, final_scores)
                            scores = extract_scores_from_ergwert(
                                get_soup(ergwert_html)
                            )
                            save_scores(event_name, scores)
                    else:
                        logger.warning(f"Failed to download competition page: {link}")
                except Exception:
                    error_logger.error(
                        f"Error processing competition {link}", exc_info=True
                    )
                    print(
                        f"Error processing competition {link}. See error.log for details."
                    )
            logger.info(
                f"Successfully downloaded {success_count}/{len(comp_links)} competition pages from {url}"
            )
            print(
                f"Successfully downloaded {success_count}/{len(comp_links)} competition pages from {url}"
            )
            print("Summary for this URL:")
            print(f"  Competitions found: {len(comp_links)}")
            print(f"  Competitions successfully processed: {success_count}")
        else:
            logger.warning(f"Skipping {url} due to download error.")
            print(f"Skipping {url} due to download error.")
    print("All URLs processed.")


if __name__ == "__main__":
    main()