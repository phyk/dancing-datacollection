from bs4 import BeautifulSoup, Doctype, NavigableString

def canonical_deck_html(html: str) -> str:
    soup = BeautifulSoup(html, 'lxml')

    # Rebuild the head
    title_text = ' '.join(soup.title.string.replace('&#34;1&#34;', '').replace(' "1"', '').split()) if soup.title else ''
    soup.head.clear()
    soup.head.append(soup.new_tag('meta', attrs={'content': 'text/html; charset=utf-8', 'http-equiv': 'Content-Type'}))
    title_tag = soup.new_tag('title')
    title_tag.string = title_text
    soup.head.append(title_tag)

    # Clean the body
    for tag in soup.body.find_all(['p', 'a', 'font']):
        tag.decompose()

    for tag in soup.body.find_all(True):
        allowed_attrs = {}
        if tag.name == 'span':
            # Keep empty spans
            pass
        tag.attrs = allowed_attrs

    # Replace header text
    for text_node in soup.find_all(string=True):
        if '11.05.2024 - OT, Hgr.II D Standard "1"' in text_node:
            text_node.replace_with('11.05.2024 - OT, Hgr.II D Standard')

    # Remove the original doctype
    for item in soup.contents:
        if isinstance(item, Doctype):
            item.extract()
    soup.insert(0, Doctype('html'))

    return str(soup)

def canonical_tabges_html(html: str) -> str:
    soup = BeautifulSoup(html, 'lxml')

    # Rebuild the head
    title_text = soup.title.string.strip() if soup.title else ''
    soup.head.clear()
    soup.head.append(soup.new_tag('meta', attrs={'content': 'text/html; charset=utf-8', 'http-equiv': 'Content-Type'}))
    title_tag = soup.new_tag('title')
    title_tag.string = title_text
    soup.head.append(title_tag)

    # Clean the body
    for tag in soup.body.find_all(['p', 'a', 'font']):
        tag.decompose()

    for span in soup.select("span.tooltip2gc"):
        span.unwrap()

    for tag in soup.body.find_all(True):
        tag.attrs = {}

    # Replace header text
    for text_node in soup.find_all(string=True):
        if '11.05.2024 - OT, Hgr.II D Standard "1"' in text_node:
            text_node.replace_with('11.05.2024 - OT, Hgr.II D Standard')

    # Remove the original doctype
    for item in soup.contents:
        if isinstance(item, Doctype):
            item.extract()
    soup.insert(0, Doctype('html'))

    return str(soup)

def canonical_erg_html(html: str) -> str:
    soup = BeautifulSoup(html, 'lxml')

    # Rebuild the head
    title_text = soup.title.string.strip() if soup.title else ''
    soup.head.clear()
    soup.head.append(soup.new_tag('meta', attrs={'content': 'text/html; charset=utf-8', 'http-equiv': 'Content-Type'}))
    title_tag = soup.new_tag('title')
    title_tag.string = title_text
    soup.head.append(title_tag)

    # Clean the body
    for tag in soup.body.find_all(['p', 'a', 'font']):
        tag.decompose()

    for div in soup.select("div.pz"):
        div.unwrap()

    for tag in soup.body.find_all(True):
        tag.attrs = {}

    # Replace header text
    for text_node in soup.find_all(string=True):
        if '11.05.2024 - OT, Hgr.II D Standard "1"' in text_node:
            text_node.replace_with('11.05.2024 - OT, Hgr.II D Standard')

    # Remove the original doctype
    for item in soup.contents:
        if isinstance(item, Doctype):
            item.extract()
    soup.insert(0, Doctype('html'))

    return str(soup)

def canonical_ergwert_html(html: str) -> str:
    soup = BeautifulSoup(html, 'lxml')

    # Rebuild the head
    title_text = soup.title.string.strip() if soup.title else ''
    soup.head.clear()
    soup.head.append(soup.new_tag('meta', attrs={'content': 'text/html; charset=utf-8', 'http-equiv': 'Content-Type'}))
    title_tag = soup.new_tag('title')
    title_tag.string = title_text
    soup.head.append(title_tag)

    # Clean the body
    for tag in soup.body.find_all(['p', 'a', 'font']):
        tag.decompose()

    for span in soup.select("span.tooltip2w"):
        span.unwrap()

    for div in soup.select("div.ergwertinfo"):
        div.decompose()

    for tr in soup.select("tr.td0v"):
        tr.decompose()

    header_rows = soup.select('table.tab1 tr')
    if len(header_rows) > 1:
        main_header, judge_header = header_rows[0], header_rows[1]
        main_header.parent.insert(1, judge_header)

    for tag in soup.body.find_all(True):
        tag.attrs = {}

    # Replace header text
    for text_node in soup.find_all(string=True):
        if '11.05.2024 - OT, Hgr.II D Standard "1"' in text_node:
            text_node.replace_with('11.05.2024 - OT, Hgr.II D Standard')

    # Remove the original doctype
    for item in soup.contents:
        if isinstance(item, Doctype):
            item.extract()
    soup.insert(0, Doctype('html'))

    return str(soup)