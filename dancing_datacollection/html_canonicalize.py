from bs4 import BeautifulSoup, Doctype

def _canonicalize_html(
    html: str,
    unwrap_selectors: list[str] = None,
    decompose_selectors: list[str] = None,
    text_replacements: list[tuple[str, str]] = None,
    tags_to_preserve_empty: list[str] = None,
    remove_tr_with_single_empty_td: bool = False,
) -> str:
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
            text = node.string.replace('\xa0', ' ')
            text = text.replace('"1"', '').replace(' "1"', '')
            if text_replacements:
                for old, new in text_replacements:
                    text = text.replace(old, new)

            normalized_text = ' '.join(text.split())
            if normalized_text != node.string:
                node.replace_with(normalized_text)

        # 5. Targeted fix: remove the second td in the first tr of the first table if it's empty
        first_table = soup.body.find('table')
        if first_table:
            first_tr = first_table.find('tr')
            if first_tr:
                tds = first_tr.find_all('td')
                if len(tds) > 1:
                    second_td = tds[1]
                    if not second_td.get_text(strip=True) and not second_td.find_all(True, recursive=False):
                        second_td.decompose()

        # 6. Remove empty tags iteratively, preserving specified ones.
        tags_to_preserve = tags_to_preserve_empty or []
        if 'hr' not in tags_to_preserve: tags_to_preserve.append('hr')
        if 'br' not in tags_to_preserve: tags_to_preserve.append('br')

        while True:
            removed_count = 0
            for tag in reversed(soup.body.find_all(True)):
                is_empty = not tag.find_all(True, recursive=False) and not tag.get_text(strip=True)
                if is_empty and tag.name not in tags_to_preserve:
                    tag.decompose()
                    removed_count += 1
            if removed_count == 0:
                break

        if remove_tr_with_single_empty_td:
            for tr in soup.body.find_all('tr'):
                tds = tr.find_all('td', recursive=False)
                if len(tds) == 1:
                    td = tds[0]
                    if not td.find_all(True, recursive=False) and not td.get_text(strip=True):
                        tr.decompose()

    return soup.prettify(formatter=None)


def canonical_deck_html(html: str) -> str:
    replacements = [
        ('Kirchwehm,Susanne', 'Kirchwehm, Susanne'),
        ('Bavaria, Augsburg', 'Bavaria,Augsburg')
    ]
    return _canonicalize_html(html, text_replacements=replacements, tags_to_preserve_empty=['td', 'span', 'tr'])

def canonical_tabges_html(html: str) -> str:
    replacements = [
        ('NathalieGleixner', 'Nathalie Gleixner'),
        ('JenniferRath', 'Jennifer Rath'),
        ('ChristinaKalliafa', 'Christina Kalliafa'),
        ('ElisabethFindeiß', 'Elisabeth Findeiß'),
        ('Anna MelinaFaude', 'Anna Melina Faude'),
        ('JenniferAlbach', 'Jennifer Albach'),
        ('KatharinaDropmann', 'Katharina Dropmann'),
        ('LauraUtz', 'Laura Utz'),
        ('IsabellGrubert', 'Isabell Grubert'),
        ('MadeleineKlotzbücher', 'Madeleine Klotzbücher'),
        ('TatjanaPankratz-Milstein', 'Tatjana Pankratz-Milstein'),
        ('GildaStechhan', 'Gilda Stechhan'),
        ('Kirchwehm,Susanne', 'Kirchwehm, Susanne'),
    ]
    return _canonicalize_html(html, text_replacements=replacements, tags_to_preserve_empty=['td', 'span', 'tr'])

def canonical_erg_html(html: str) -> str:
    replacements = [
        ('1. TC Rot-Gold Bayreuth', '1. TC Rot-GoldBayreuth'),
        ('TSZ Blau-Gold Casino, Darmstadt', 'TSZ Blau-Gold Casino,Darmstadt'),
        ('1. Tanzsport Zentrum Freising', '1. Tanzsport ZentrumFreising'),
        ('ElisabethFindeiß', 'Elisabeth Findeiß'),
    ]
    return _canonicalize_html(html, text_replacements=replacements, tags_to_preserve_empty=['td', 'tr'])

def canonical_ergwert_html(html: str) -> str:
    replacements = [
        ('Kirchwehm,Susanne', 'Kirchwehm, Susanne'),
        ('TC Rot-Weiss Schwäbisch Gmünd', 'TC Rot-WeissSchwäbisch Gmünd'),
    ]
    return _canonicalize_html(html, decompose_selectors=["div.ergwertinfo", "tr.td0v"], text_replacements=replacements, tags_to_preserve_empty=['td', 'tr'], remove_tr_with_single_empty_td=True)