from typing import Optional

def load_competition_results(
    target_folder: str,
    url: str,
    date: Optional[str] = None,
    age_group: Optional[str] = None,
    style: Optional[str] = None,
    level: Optional[str] = None,
    download_html: bool = True,
    output_format: str = "json"
) -> None: ...
