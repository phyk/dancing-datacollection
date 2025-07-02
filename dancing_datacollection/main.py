# Orchestration logic for the dancing datacollection project
import argparse
from dancing_datacollection.parsing_participants import extract_participants, extract_judges, extract_committee, extract_scores_from_tabges
from dancing_datacollection.parsing_utils import download_html, extract_competition_links, deduplicate_participants
from dancing_datacollection.output import save_competition_data, save_judges, save_committee, save_scores, save_final_scoring
import toml
import logging
from tqdm import tqdm
import os
from dancing_datacollection.parsing_topturnier import TopTurnierParser
import urllib.robotparser
import urllib.parse

CONFIG_PATH = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'config.toml')
LOG_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'logs')
APP_LOG_PATH = os.path.join(LOG_DIR, 'app.log')
ERROR_LOG_PATH = os.path.join(LOG_DIR, 'error.log')

os.makedirs(LOG_DIR, exist_ok=True)

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s %(levelname)s %(message)s',
    handlers=[
        logging.FileHandler(APP_LOG_PATH),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

error_logger = logging.getLogger('error')
error_handler = logging.FileHandler(ERROR_LOG_PATH)
error_logger.addHandler(error_handler)
error_logger.setLevel(logging.ERROR)

def load_config():
    with open(CONFIG_PATH, 'r') as f:
        return toml.load(f)

def is_allowed_by_robots(url, user_agent='*'):
    parsed = urllib.parse.urlparse(url)
    robots_url = f"{parsed.scheme}://{parsed.netloc}/robots.txt"
    rp = urllib.robotparser.RobotFileParser()
    try:
        rp.set_url(robots_url)
        rp.read()
        allowed = rp.can_fetch(user_agent, url)
        if not allowed:
            logging.warning(f"robots.txt at {robots_url} disallows access to {url} for user-agent {user_agent}")
        return allowed
    except Exception as e:
        logging.error(f"Failed to check robots.txt at {robots_url}: {e}")
        # If robots.txt cannot be fetched, default to allowed
        return True

def process_local_dir(local_dir):
    import glob
    import codecs
    parser = TopTurnierParser()
    all_participants = []
    event_name = os.path.basename(local_dir)
    judges = []
    committee = []
    scores = []
    final_scores = []
    deck_path = os.path.join(local_dir, 'deck.htm')
    tabges_path = os.path.join(local_dir, 'tabges.htm')
    ergwert_path = os.path.join(local_dir, 'ergwert.htm')
    canceled = False
    # Check for cancellation in deck.htm
    if os.path.exists(deck_path):
        with open(deck_path, 'r', encoding='utf-8') as f:
            deck_html = f.read()
        if ('abgesagt' in deck_html.lower()) or ('canceled' in deck_html.lower()):
            canceled = True
            logging.info(f"Competition {event_name} is canceled. Skipping all output files.")
            print(f"Competition {event_name} is canceled. Skipping all output files.")
            # Optionally, write a minimal canceled.parquet or log entry here
            return  # Skip writing any output files
        judges = parser.extract_judges(deck_html)
        logger.info(f"Judges found: {len(judges)}")
        print(f"Judges found: {len(judges)}")
        save_judges(event_name, judges)
        committee = parser.extract_committee(deck_html)
        logger.info(f"Committee entries found: {len(committee)}")
        print(f"Committee entries found: {len(committee)}")
        save_committee(event_name, committee)
    if os.path.exists(tabges_path):
        with open(tabges_path, 'r', encoding='utf-8') as f:
            tabges_html = f.read()
        scores = parser.extract_scores(tabges_html)
        logger.info(f"Score entries found: {len(scores)}")
        print(f"Score entries found: {len(scores)}")
        save_scores(event_name, scores)
    if os.path.exists(ergwert_path):
        with open(ergwert_path, 'r', encoding='utf-8') as f:
            ergwert_html = f.read()
        final_scores = parser.extract_final_scoring(ergwert_html)
        logger.info(f"Final scoring entries found: {len(final_scores)}")
        print(f"Final scoring entries found: {len(final_scores)}")
        save_final_scoring(event_name, final_scores)
    htm_files = []
    for root, dirs, files in os.walk(local_dir):
        for fname in files:
            if fname.endswith('.htm'):
                htm_files.append(os.path.join(root, fname))
    if not htm_files:
        print(f"No .htm files found in {local_dir}.")
    else:
        print(f"Processing {len(htm_files)} .htm files in {local_dir}...")
        for fpath in tqdm(htm_files, desc="Parsing .htm files", unit="file"):
            try:
                with codecs.open(fpath, 'r', encoding='utf-8') as f:
                    html = f.read()
                participants, _ = parser.extract_participants(html)
                if participants:
                    logger.info(f"  Participants found in {os.path.basename(fpath)}: {len(participants)}")
                    all_participants.extend(participants)
            except Exception as e:
                error_logger.error(f"Error processing file {fpath}", exc_info=True)
                print(f"Error processing file {fpath}. See error.log for details.")
    unique_participants = deduplicate_participants(all_participants)
    logger.info(f"Total unique participants in {local_dir}: {len(unique_participants)}")
    print(f"Total unique participants in {local_dir}: {len(unique_participants)}")
    save_competition_data(event_name, unique_participants)
    print("Summary:")
    print(f"  Judges: {len(judges)}")
    print(f"  Committee: {len(committee)}")
    print(f"  Scores: {len(scores)}")
    print(f"  Final scoring: {len(final_scores)}")
    print(f"  Unique participants: {len(unique_participants)}")

def main():
    parser = argparse.ArgumentParser(description='Dancing Competition Data Collection')
    parser.add_argument('--local-dir', type=str, help='Process all .htm files in a local directory (for testing/offline)')
    args = parser.parse_args()
    if args.local_dir:
        process_local_dir(args.local_dir)
        return
    config = load_config()
    urls = config.get('sources', {}).get('urls', [])
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
                        from urllib.parse import urlparse
                        base_url = link.rsplit('/', 1)[0]
                        erg_url = f"{base_url}/erg.htm"
                        erg_html = download_html(erg_url)
                        if erg_html:
                            participants, event_name = extract_participants(erg_html)
                            logger.info(f"Parsed competition (erg.htm): {erg_url}")
                        else:
                            participants, event_name = extract_participants(comp_html)
                            logger.info(f"Parsed competition (index): {link}")
                        logger.info(f"  Participants: {len(participants)}")
                        print(f"Parsed competition: {link}")
                        print(f"  Participants: {len(participants)}")
                        save_competition_data(event_name, participants)
                        # After saving participants, try to download and save judges
                        deck_url = f"{base_url}/deck.htm"
                        deck_html = download_html(deck_url)
                        if deck_html:
                            parser = TopTurnierParser()
                            judges = parser.extract_judges(deck_html)
                            logger.info(f"Judges found: {len(judges)}")
                            print(f"Judges found: {len(judges)}")
                            save_judges(event_name, judges)
                            committee = parser.extract_committee(deck_html)
                            logger.info(f"Committee entries found: {len(committee)}")
                            print(f"Committee entries found: {len(committee)}")
                            save_committee(event_name, committee)
                        tabges_url = f"{base_url}/tabges.htm"
                        tabges_html = download_html(tabges_url)
                        if tabges_html:
                            parser = TopTurnierParser()
                            scores = parser.extract_scores(tabges_html)
                            logger.info(f"Score entries found: {len(scores)}")
                            print(f"Score entries found: {len(scores)}")
                            save_scores(event_name, scores)
                        ergwert_url = f"{base_url}/ergwert.htm"
                        ergwert_html = download_html(ergwert_url)
                        if ergwert_html:
                            final_scores = parser.extract_final_scoring(ergwert_html)
                            logger.info(f"Final scoring entries found: {len(final_scores)}")
                            print(f"Final scoring entries found: {len(final_scores)}")
                            save_final_scoring(event_name, final_scores)
                    else:
                        logger.warning(f"Failed to download competition page: {link}")
                except Exception as e:
                    error_logger.error(f"Error processing competition {link}", exc_info=True)
                    print(f"Error processing competition {link}. See error.log for details.")
            logger.info(f"Successfully downloaded {success_count}/{len(comp_links)} competition pages from {url}")
            print(f"Successfully downloaded {success_count}/{len(comp_links)} competition pages from {url}")
            print("Summary for this URL:")
            print(f"  Competitions found: {len(comp_links)}")
            print(f"  Competitions successfully processed: {success_count}")
        else:
            logger.warning(f"Skipping {url} due to download error.")
            print(f"Skipping {url} due to download error.")
    print("All URLs processed.")

if __name__ == '__main__':
    main() 