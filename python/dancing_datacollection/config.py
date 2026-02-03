import os
from typing import Any, Dict

import toml

CONFIG_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.dirname(__file__))), "config.toml"
)


def load_config(path: str = CONFIG_PATH) -> Dict[str, Any]:
    if not os.path.exists(path):
        return {}
    with open(path, "r", encoding="utf-8") as f:
        return toml.load(f)


_CONFIG = load_config()


def get_config() -> Dict[str, Any]:
    return _CONFIG


def get_age_groups() -> Dict[str, Dict[str, str]]:
    return _CONFIG.get("age_groups", {})


def get_disciplines() -> Dict[str, Dict[str, str]]:
    return _CONFIG.get("disciplines", {})


def get_levels() -> Dict[str, Dict[str, Any]]:
    return _CONFIG.get("levels", {})
