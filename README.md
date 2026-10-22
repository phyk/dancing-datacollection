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

Invalid filter values (e.g. an unknown `age_group`, `style`, or `level`) raise `ValueError`.

Logs are emitted through Python's standard `logging` module; logger names start with `dancing_datacollection`, so you can configure them with e.g. `logging.getLogger("dancing_datacollection").setLevel(logging.DEBUG)`.

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

The function handles discovery of individual competitions from an event index, respects `robots.txt` and crawl delays (pausing 0.5 s between requests when none is set), performs math verification (Safety Shield), and stores the results in a structured JSON format as `{Event_Name}_{Year}/{AgeGroup}_{Level}_{Style}_{YYYY-MM-DD}.json`. Floor notes like `"Fläche 3/4"` are left out of folder names, and rerunning the same URL writes to the same files.

## Development

Build and install the Python extension into the project's virtual environment (built with [maturin](https://www.maturin.rs/); Python sources live in `python/`):

```sh
uv run maturin develop
```

Run the Rust tests locally:

```sh
PYO3_PYTHON=.venv/bin/python3 cargo test --no-default-features
```

`--no-default-features` disables PyO3's `extension-module` feature so the test binaries can link against libpython. `PYO3_PYTHON` points PyO3 at the project's interpreter; otherwise it may pick up a system Python that can't be linked.

Lint (mirrors `.github/workflows/ci.yml`):

```sh
cargo fmt --check
cargo clippy --no-default-features --all-targets -- -D warnings
```

Regenerate the golden files:

```sh
UPDATE_GOLDEN=1 PYO3_PYTHON=.venv/bin/python3 cargo test --no-default-features --test integration_tests
```

This overwrites `tests/*/tabges.json` and writes `tests/*/reconstructed_ergwert.html` (git-ignored) for visual inspection. Review the diff before committing.

Pre-commit hooks (whitespace/EOF fixers, JSON/TOML checks, `cargo fmt`, `cargo check`, `clippy`) are configured in `prek.toml`; install them with `prek install`.
