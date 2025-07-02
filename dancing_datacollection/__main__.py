import argparse
import toml
import os
import logging
from tqdm import tqdm
import urllib.request
import tempfile
from urllib.parse import urljoin
import polars as pl
import re
from bs4 import BeautifulSoup

CONFIG_PATH = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'config.toml')
LOG_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'logs')
APP_LOG_PATH = os.path.join(LOG_DIR, 'app.log')
ERROR_LOG_PATH = os.path.join(LOG_DIR, 'error.log')
DATA_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'data')

# Ensure log and data directories exist
os.makedirs(LOG_DIR, exist_ok=True)
os.makedirs(DATA_DIR, exist_ok=True)

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s %(levelname)s %(message)s',
    handlers=[
        logging.FileHandler(APP_LOG_PATH),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

# Placeholder for error logging
error_logger = logging.getLogger('error')
error_handler = logging.FileHandler(ERROR_LOG_PATH)
error_logger.addHandler(error_handler)
error_logger.setLevel(logging.ERROR)


def load_config():
    with open(CONFIG_PATH, 'r') as f:
        return toml.load(f)

def download_html(url):
    try:
        logger.info(f"Downloading: {url}")
        with urllib.request.urlopen(url) as response:
            html = response.read().decode('utf-8')
        logger.info(f"Downloaded {len(html)} characters from {url}")
        return html
    except Exception as e:
        error_logger.error(f"Failed to download {url}: {e}")
        return None

class CompetitionLinkParser:
    def __init__(self, base_url):
        self.base_url = base_url
    def extract_links(self, html):
        soup = BeautifulSoup(html, "html.parser")
        links = []
        for a in soup.find_all('a', href=True):
            href = a['href']
            if href.endswith('.htm') or href.endswith('.html'):
                full_url = urljoin(self.base_url, href)
                links.append(full_url)
        return links

def extract_competition_links(html, base_url):
    parser = CompetitionLinkParser(base_url)
    return parser.extract_links(html)

class CompetitionParser:
    def __init__(self, html):
        self.soup = BeautifulSoup(html, "html.parser")
        self.title = self.extract_title()
        self.organization = None
        self.participants = []
        self.judges = []
        self.scores = []
    def extract_title(self):
        title_tag = self.soup.find('title')
        if title_tag:
            return title_tag.get_text(strip=True)
        return None
    def extract_participants(self):
        # Look for the first table and extract rows as participants
        table = self.soup.find('table')
        participants = []
        if table:
            rows = table.find_all('tr')
            if not rows:
                return participants
            # Try to use the first row as headers
            header_cells = [cell.get_text(strip=True) for cell in rows[0].find_all(['td', 'th'])]
            has_header = all(header_cells) and len(header_cells) > 1
            headers = header_cells if has_header else [f'field_{i}' for i in range(len(rows[0].find_all(['td', 'th'])))]
            start_idx = 1 if has_header else 0
            for row in rows[start_idx:]:
                cells = [cell.get_text(strip=True) for cell in row.find_all(['td', 'th'])]
                if cells:
                    # Map cells to headers, fill missing with None
                    row_dict = {headers[i]: cells[i] if i < len(cells) else None for i in range(len(headers))}
                    participants.append(row_dict)
        return participants
    def extract(self):
        # Placeholder extraction logic
        self.organization = {'chairperson': 'N/A'}
        self.participants = self.extract_participants()
        self.judges = [{'name': 'N/A'}]
        self.scores = [{'round': 1, 'score': 'N/A'}]
        return {
            'organization': self.organization,
            'participants': self.participants,
            'judges': self.judges,
            'scores': self.scores,
            'event_name': self.sanitize_event_name(self.title)
        }
    @staticmethod
    def sanitize_event_name(name):
        if not name:
            return 'unknown_event'
        name = re.sub(r'[^\w\d-]+', '_', name)
        return name[:64]

def save_competition_data(event_name, data):
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    # Save organization (single row)
    org_df = pl.DataFrame([data['organization']])
    org_path = os.path.join(comp_dir, 'organization.parquet')
    org_df.write_parquet(org_path)
    # Save participants
    part_df = pl.DataFrame(data['participants'])
    part_path = os.path.join(comp_dir, 'participants.parquet')
    part_df.write_parquet(part_path)
    # Save judges
    judges_df = pl.DataFrame(data['judges'])
    judges_path = os.path.join(comp_dir, 'judges.parquet')
    judges_df.write_parquet(judges_path)
    # Save scores
    scores_df = pl.DataFrame(data['scores'])
    scores_path = os.path.join(comp_dir, 'scores.parquet')
    scores_df.write_parquet(scores_path)
    logger.info(f"Saved organization to {org_path}")
    logger.info(f"Saved participants to {part_path}")
    logger.info(f"Saved judges to {judges_path}")
    logger.info(f"Saved scores to {scores_path}")
    print(f"Saved organization to {org_path}")
    print(f"Saved participants to {part_path}")
    print(f"Saved judges to {judges_path}")
    print(f"Saved scores to {scores_path}")
    logger.info(f"Competition data saved in directory: {comp_dir}")
    print(f"Competition data saved in directory: {comp_dir}")

def main():
    parser = argparse.ArgumentParser(description='Dancing Competition Data Collection')
    args = parser.parse_args()

    config = load_config()
    urls = config.get('sources', {}).get('urls', [])
    logger.info(f"Loaded {len(urls)} base URLs from config.")
    print(f"Checking {len(urls)} base URLs...")
    for url in tqdm(urls, desc="Processing URLs", unit="url"):
        html = download_html(url)
        if html is not None:
            logger.info(f"Successfully downloaded HTML from {url}")
            comp_links = extract_competition_links(html, url)
            logger.info(f"Found {len(comp_links)} competition/event links at {url}")
            print(f"Found {len(comp_links)} competition/event links at {url}")
            success_count = 0
            for link in tqdm(comp_links, desc="Downloading competitions", unit="comp"):
                comp_html = download_html(link)
                if comp_html is not None:
                    success_count += 1
                    # Parse competition page
                    comp_parser = CompetitionParser(comp_html)
                    data = comp_parser.extract()
                    logger.info(f"Parsed competition: {link}")
                    logger.info(f"  Organization: {data['organization']}")
                    logger.info(f"  Participants: {len(data['participants'])}")
                    logger.info(f"  Judges: {len(data['judges'])}")
                    logger.info(f"  Scores: {len(data['scores'])}")
                    print(f"Parsed competition: {link}")
                    print(f"  Organization: {data['organization']}")
                    print(f"  Participants: {len(data['participants'])}")
                    print(f"  Judges: {len(data['judges'])}")
                    print(f"  Scores: {len(data['scores'])}")
                    # Save to Parquet
                    event_name = data.get('event_name', 'unknown_event')
                    save_competition_data(event_name, data)
                else:
                    logger.warning(f"Failed to download competition page: {link}")
            logger.info(f"Successfully downloaded {success_count}/{len(comp_links)} competition pages from {url}")
            print(f"Successfully downloaded {success_count}/{len(comp_links)} competition pages from {url}")
        else:
            logger.warning(f"Skipping {url} due to download error.")

if __name__ == '__main__':
    main()
