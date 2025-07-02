import urllib.request
from bs4 import BeautifulSoup
from urllib.parse import urljoin
import logging

def download_html(url):
    try:
        logging.info(f"Downloading: {url}")
        with urllib.request.urlopen(url) as response:
            html = response.read().decode('utf-8')
        logging.info(f"Downloaded {len(html)} characters from {url}")
        return html
    except Exception as e:
        logging.error(f"Failed to download {url}: {e}")
        return None

def extract_competition_links(html, base_url):
    soup = BeautifulSoup(html, "html.parser")
    links = []
    for a in soup.find_all('a', href=True):
        href = a['href']
        if href.endswith('.htm') or href.endswith('.html'):
            full_url = urljoin(base_url, href)
            links.append(full_url)
    return links

def deduplicate_participants(participants):
    seen = set()
    unique = []
    for p in participants:
        key = (p.get('number'), p.get('names'), p.get('club'))
        if key not in seen:
            seen.add(key)
            unique.append(p)
    return unique 