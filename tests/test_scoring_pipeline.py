import pytest
from dancing_datacollection import extract_competitions, validate_extracted_competitions

def test_dtv_scoring_integration():
    data_dir = "tests/51-1105_ot_hgr2dstd"
    event_wrapper = extract_competitions(data_dir)
    assert event_wrapper is not None

    # Since PyEvent is opaque, we can't easily check internals without exposing them
    # But we can check if it passes validation
    is_valid = validate_extracted_competitions(event_wrapper)
    assert is_valid, "Fidelity Gate failed for 51-1105_ot_hgr2dstd"

    # Deep check
    # In PyO3, we might need to expose some fields or just trust the validation
    # For now, if validate_extracted_competitions returns True, it means:
    # 1. judges were found
    # 2. participants were found
    # 3. rounds were found
