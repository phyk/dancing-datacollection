import os
from typing import List

import pytest


@pytest.fixture(scope="module")
def test_dir() -> str:
    return os.path.dirname(__file__)


@pytest.fixture(scope="module")
def sample_dirs() -> List[str]:
    return [
        "51-1105_ot_hgr2dstd",
        "52-1105_ot_hgr2cstd",
        "53-1105_ot_hgr2bstd",
    ]
