# Dancing Data Collection

This library provides a high-performance Rust-based engine with Python bindings to download, parse, and validate dancing competition results (TopTurnier/DTV-native format).

## Main Entrypoint

The library exposes a single primary orchestrator function:

### `load_competition_results`

Downloads, parses, validates, and stores competition results from a given URL.

#### Parameters

- `target_folder` (str): The directory where the results will be stored.
- `url` (str): The URL of the competition or event index.
- `date` (str, optional): Filter by date (YYYY-MM-DD).
- `age_group` (str, optional): Filter by age group (e.g., "Hgr", "Sen I").
- `style` (str, optional): Filter by style/discipline (e.g., "Std", "Lat").
- `level` (str, optional): Filter by level/class (e.g., "D", "S").
- `download_html` (bool, optional): Whether to archive the raw HTML source files. Defaults to `True`.
- `output_format` (str, optional): The format of the output file. Currently only "json" is supported.

#### Usage Example

```python
from dancing_datacollection import load_competition_results

load_competition_results(
    target_folder="data/results",
    url="https://www.dancecomp.de/2024/ergebnisse/index.htm",
    age_group="Hgr",
    style="Std",
    level="S"
)
```

The function handles discovery of individual competitions from an event index, respects `robots.txt` and crawl delays, performs math verification (Safety Shield), and stores the results in a structured JSON format.
