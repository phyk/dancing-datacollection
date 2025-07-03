import os
import polars as pl
import logging

DATA_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "data")


def validate_schema(path, required_columns):
    try:
        df = pl.read_parquet(path)
        missing = [col for col in required_columns if col not in df.columns]
        if missing:
            logging.error(
                f"Schema validation failed for {path}: missing columns {missing}"
            )
            print(f"Schema validation failed for {path}: missing columns {missing}")
            return False
        # If this is participants.parquet, check number is int
        if os.path.basename(path) == "participants.parquet":
            if df["number"].dtype != pl.Int64:
                logging.error(
                    f"Schema validation failed for {path}: 'number' column is not integer type"
                )
                print(
                    f"Schema validation failed for {path}: 'number' column is not integer type"
                )
                return False
        # If this is scores.parquet, check number is int and voted is bool
        if os.path.basename(path) == "scores.parquet":
            if df["number"].dtype != pl.Int64:
                logging.error(
                    f"Schema validation failed for {path}: 'number' column is not integer type"
                )
                print(
                    f"Schema validation failed for {path}: 'number' column is not integer type"
                )
                return False
            if df["voted"].dtype != pl.Boolean:
                logging.error(
                    f"Schema validation failed for {path}: 'voted' column is not boolean type"
                )
                print(
                    f"Schema validation failed for {path}: 'voted' column is not boolean type"
                )
                return False
        logging.info(f"Schema validation passed for {path}")
        print(f"Schema validation passed for {path}")
        return True
    except Exception as e:
        logging.error(f"Schema validation error for {path}: {e}")
        print(f"Schema validation error for {path}: {e}")
        return False


def save_competition_data(event_name, participants):
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    expected_cols = ["name_one", "name_two", "club", "number"]
    if not isinstance(participants, list):
        participants = []
    norm = []
    for p in participants:
        if not isinstance(p, dict):
            continue
        # Ensure number is int or None
        entry = {col: p.get(col, None) for col in expected_cols}
        try:
            entry["number"] = (
                int(entry["number"]) if entry["number"] is not None else None
            )
        except Exception:
            entry["number"] = None
        norm.append(entry)
    if norm:
        part_df = pl.DataFrame(norm)
        for col in expected_cols:
            if col not in part_df.columns:
                part_df = part_df.with_columns(pl.lit(None).alias(col))
        # Ensure correct column order and types
        part_df = part_df.select(expected_cols)
        part_df = part_df.with_columns([pl.col("number").cast(pl.Int64, strict=False)])
    else:
        part_df = pl.DataFrame({col: [] for col in expected_cols})
    part_path = os.path.join(comp_dir, "participants.parquet")
    part_df.write_parquet(part_path)
    logging.info(f"Saved participants to {part_path}")
    print(f"Saved participants to {part_path}")
    validate_schema(part_path, expected_cols)


def save_judges(event_name, judges):
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    expected_cols = ["code", "name", "club"]
    if not isinstance(judges, list):
        judges = []
    norm = []
    for j in judges:
        if not isinstance(j, dict):
            continue
        norm.append({col: j.get(col, None) for col in expected_cols})
    if norm:
        judges_df = pl.DataFrame(norm)
        for col in expected_cols:
            if col not in judges_df.columns:
                judges_df = judges_df.with_columns(pl.lit(None).alias(col))
        judges_df = judges_df.select(expected_cols)
    else:
        judges_df = pl.DataFrame({col: [] for col in expected_cols})
    judges_path = os.path.join(comp_dir, "judges.parquet")
    judges_df.write_parquet(judges_path)
    logging.info(f"Saved judges to {judges_path}")
    print(f"Saved judges to {judges_path}")
    validate_schema(judges_path, expected_cols)


def save_committee(event_name, committee):
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    expected_cols = ["role", "name", "club"]
    if not isinstance(committee, list):
        committee = []
    norm = []
    for c in committee:
        if not isinstance(c, dict):
            continue
        entry = {col: c.get(col, None) for col in expected_cols}
        norm.append(entry)
    if norm:
        committee_df = pl.DataFrame(norm)
        for col in expected_cols:
            if col not in committee_df.columns:
                committee_df = committee_df.with_columns(pl.lit(None).alias(col))
        committee_df = committee_df.select(expected_cols)
    else:
        committee_df = pl.DataFrame({col: [] for col in expected_cols})
    committee_path = os.path.join(comp_dir, "committee.parquet")
    committee_df.write_parquet(committee_path)
    logging.info(f"Saved committee to {committee_path}")
    print(f"Saved committee to {committee_path}")
    validate_schema(committee_path, expected_cols)


def save_scores(event_name, scores):
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    expected_cols = ["round", "number", "judge_code", "dance", "voted"]
    if not isinstance(scores, list):
        scores = []
    norm = []
    for s in scores:
        if not isinstance(s, dict):
            continue
        entry = {col: s.get(col, None) for col in expected_cols}
        # Ensure number is int and voted is bool
        try:
            entry["number"] = (
                int(entry["number"]) if entry["number"] is not None else None
            )
        except Exception:
            entry["number"] = None
        entry["voted"] = bool(entry["voted"]) if entry["voted"] is not None else False
        norm.append(entry)
    if norm:
        scores_df = pl.DataFrame(norm)
        for col in expected_cols:
            if col not in scores_df.columns:
                scores_df = scores_df.with_columns(pl.lit(None).alias(col))
        scores_df = scores_df.select(expected_cols)
        scores_df = scores_df.with_columns(
            [
                pl.col("number").cast(pl.Int64, strict=False),
                pl.col("voted").cast(pl.Boolean, strict=False),
            ]
        )
    else:
        scores_df = pl.DataFrame({col: [] for col in expected_cols})
    scores_path = os.path.join(comp_dir, "scores.parquet")
    scores_df.write_parquet(scores_path)
    logging.info(f"Saved scores to {scores_path}")
    print(f"Saved scores to {scores_path}")
    validate_schema(scores_path, expected_cols)


def save_final_scoring(event_name, final_scores):
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    expected_cols = [
        "placement",
        "names",
        "number",
        "club",
        "score_LW",
        "score_TG",
        "score_QS",
        "total",
    ]
    if not isinstance(final_scores, list):
        final_scores = []
    norm = []
    for f in final_scores:
        if not isinstance(f, dict):
            continue
        norm.append({col: f.get(col, None) for col in expected_cols})
    if norm:
        final_df = pl.DataFrame(norm)
        for col in expected_cols:
            if col not in final_df.columns:
                final_df = final_df.with_columns(pl.lit(None).alias(col))
        final_df = final_df.select(expected_cols)
    else:
        final_df = pl.DataFrame({col: [] for col in expected_cols})
    final_path = os.path.join(comp_dir, "final_scoring.parquet")
    final_df.write_parquet(final_path)
    logging.info(f"Saved final scoring to {final_path}")
    print(f"Saved final scoring to {final_path}")
    validate_schema(final_path, expected_cols)
