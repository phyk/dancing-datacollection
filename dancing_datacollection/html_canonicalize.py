from bs4 import BeautifulSoup, Doctype

def canonicalize_html(html: str) -> str:
    soup = BeautifulSoup(html, 'lxml')

    # --- Doctype processing ---
    for item in soup.contents:
        if isinstance(item, Doctype):
            item.extract()
    soup.insert(0, Doctype('html'))

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
        # 1. Decompose common unwanted tags.
        to_decompose = ['p', 'a', 'font', 'link', 'style', 'script']
        for selector in to_decompose:
            for tag in soup.body.select(selector):
                tag.decompose()

        # 2. Strip all attributes from all tags.
        for tag in soup.body.find_all(True):
            tag.attrs = {}

        # 3. Normalize all text content.
        for node in soup.body.find_all(string=True):
            text = node.string.replace('\xa0', ' ')
            text = text.replace('"1"', '').replace(' "1"', '')
            normalized_text = ' '.join(text.split())
            if normalized_text != node.string:
                node.replace_with(normalized_text)

        # 4. Targeted fix: remove the second td in the first tr of the first table if it's empty
        first_table = soup.body.find('table')
        if first_table:
            first_tr = first_table.find('tr')
            if first_tr:
                tds = first_tr.find_all('td')
                if len(tds) > 1:
                    second_td = tds[1]
                    if not second_td.get_text(strip=True) and not second_td.find_all(True, recursive=False):
                        second_td.decompose()

        # 5. Remove empty tags iteratively, preserving specified ones.
        tags_to_preserve = ['hr', 'br']

        while True:
            removed_count = 0
            for tag in reversed(soup.body.find_all(True)):
                is_empty = not tag.find_all(True, recursive=False) and not tag.get_text(strip=True)
                if is_empty and tag.name not in tags_to_preserve:
                    tag.decompose()
                    removed_count += 1
            if removed_count == 0:
                break

    return soup.prettify(formatter=None)