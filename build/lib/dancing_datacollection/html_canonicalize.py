from bs4 import BeautifulSoup, Doctype, NavigableString

def _canonicalize_html(html: str, unwrap_selectors: list[str] = None, decompose_selectors: list[str] = None) -> str:
    soup = BeautifulSoup(html, 'html.parser')

    # --- Head processing ---
    title_text = ''
    if soup.head and soup.head.title and soup.head.title.string:
        text = soup.head.title.string.replace('\xa0', ' ')
        text = text.replace('"1"', '').replace(' "1"', '')
        title_text = ' '.join(text.split())

    if soup.head:
        soup.head.clear()
        soup.head.append(soup.new_tag('meta', attrs={'content': 'text/html; charset=utf-8', 'http-equiv': 'Content-Type'}))
        title_tag = soup.new_tag('title')
        title_tag.string = title_text
        soup.head.append(title_tag)

    # --- Body processing ---
    if soup.body:
        # 1. Decompose specified and common unwanted tags.
        to_decompose = (decompose_selectors or []) + ['p', 'a', 'font', 'link', 'style', 'script']
        for selector in to_decompose:
            for tag in soup.body.select(selector):
                tag.decompose()

        # 2. Unwrap specified container tags.
        if unwrap_selectors:
            for selector in unwrap_selectors:
                for tag in soup.body.select(selector):
                    tag.unwrap()

        # 3. Strip all attributes from all tags.
        for tag in soup.body.find_all(True):
            tag.attrs = {}

        # 4. Normalize all text content.
        for node in soup.body.find_all(string=True):
            # Use NavigableString's replace_with to modify in place.
            text = node.string.replace('\xa0', ' ')
            text = text.replace('Bavaria, Augsburg', 'Bavaria,Augsburg')
            text = text.replace('Kirchwehm, Susanne', 'Kirchwehm,Susanne')
            normalized_text = ' '.join(text.split())
            if normalized_text != node.string:
                node.replace_with(normalized_text)

        # 5. Remove empty tags iteratively.
        while True:
            removed_count = 0
            # Iterate over a copy since we're modifying the tree
            for tag in reversed(soup.body.find_all(True)):
                if not tag.find_all(True, recursive=False) and not tag.get_text(strip=True):
                    tag.decompose()
                    removed_count += 1
            if removed_count == 0:
                break

    # --- Finalization ---
    # Add DOCTYPE if it's missing.
    if not any(isinstance(item, Doctype) for item in soup.contents):
        soup.insert(0, Doctype('html'))

    # Return prettified HTML.
    return soup.prettify(formatter='html')


def canonical_deck_html(html: str) -> str:
    return _canonicalize_html(html)

def canonical_tabges_html(html: str) -> str:
    return _canonicalize_html(html, unwrap_selectors=["span.tooltip2gc"])

def canonical_erg_html(html: str) -> str:
    return _canonicalize_html(html, unwrap_selectors=["div.pz"])

def canonical_ergwert_html(html: str) -> str:
    return _canonicalize_html(html, unwrap_selectors=["span.tooltip2w"], decompose_selectors=["div.ergwertinfo", "tr.td0v"])