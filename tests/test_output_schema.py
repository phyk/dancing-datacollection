import os
import sys
from dancing_datacollection.output import validate_schema

data_dir = "data"
expected_files = {
    "participants.parquet": [
        "placement",
        "names",
        "number",
        "club",
        "round",
        "score_LW",
        "score_TG",
        "score_QU",
        "score_PZ",
    ],
    "judges.parquet": ["code", "name", "club"],
    "committee.parquet": ["role", "name", "club", "raw_value"],
    "scores.parquet": ["round", "number", "names"],
    "final_scoring.parquet": [
        "placement",
        "names",
        "number",
        "club",
        "score_LW",
        "score_TG",
        "score_QS",
        "total",
    ],
}

failures = []


def validate_all():
    for comp_dir in os.listdir(data_dir):
        comp_path = os.path.join(data_dir, comp_dir)
        if not os.path.isdir(comp_path):
            continue
        print(f"Validating {comp_dir}...")
        for fname, columns in expected_files.items():
            fpath = os.path.join(comp_path, fname)
            if os.path.exists(fpath):
                ok = validate_schema(fpath, columns)
                if not ok:
                    failures.append(f"{comp_dir}/{fname}")
            else:
                print(f"  (skipped missing {fname})")
    if failures:
        print("\nFAILED schema validation for:")
        for f in failures:
            print("  ", f)
        sys.exit(1)
    else:
        print("\nAll output files passed schema validation.")


if __name__ == "__main__":
    validate_all()
