import logging
import os
from typing import Any, List, Optional

import polars as pl

from dancing_datacollection.data_defs.committee import CommitteeMember
from dancing_datacollection.data_defs.competition import CompetitionInfo
from dancing_datacollection.data_defs.final_scoring import FinalScoring
from dancing_datacollection.data_defs.judge import Judge
from dancing_datacollection.data_defs.participant import Participant

DATA_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "data")


def validate_schema(path: str, required_columns: List[str]) -> bool:
    try:
        df = pl.read_parquet(path)
        missing = [col for col in required_columns if col not in df.columns]
        if missing:
            logging.error("Schema validation failed for %s: missing columns %s", path, missing)
            return False
        # If this is participants.parquet, check number is int
        if os.path.basename(path) == "participants.parquet" and df["number"].dtype != pl.Int64:
            logging.error(
                "Schema validation failed for %s: 'number' column is not integer type",
                path,
            )
            return False
        # If this is scores.parquet, check number is int and voted is bool
        if os.path.basename(path) == "scores.parquet":
            if df["number"].dtype != pl.Int64:
                logging.error(
                    "Schema validation failed for %s: 'number' column is not integer type",
                    path,
                )
                return False
            if df["voted"].dtype != pl.Boolean:
                logging.error(
                    "Schema validation failed for %s: 'voted' column is not boolean type",
                    path,
                )
                return False
        logging.info("Schema validation passed for %s", path)
        return True
    except (pl.exceptions.PolarsError, IOError) as e:
        logging.error("Schema validation error for %s: %s", path, e)
        return False


def save_competition_info(event_name: str, info: CompetitionInfo) -> None:
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    info_path = os.path.join(comp_dir, "metadata.json")
    with open(info_path, "w", encoding="utf-8") as f:
        f.write(info.model_dump_json(indent=2))
    logging.info("Saved competition metadata to %s", info_path)


def save_competition_data(
    event_name: str, participants: List[Participant], info: Optional[CompetitionInfo] = None
) -> None:
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    if info:
        save_competition_info(event_name, info)
    expected_cols = ["name_one", "name_two", "club", "number", "ranks"]
    if participants:
        part_df = pl.DataFrame([p.model_dump() for p in participants])
        for col in expected_cols:
            if col not in part_df.columns:
                part_df = part_df.with_columns(pl.lit(None).alias(col))
        # Ensure correct column order and types
        part_df = part_df.select(expected_cols)
        part_df = part_df.with_columns([pl.col("number").cast(pl.Int64, strict=False)])
    else:
        part_df = pl.DataFrame(
            {col: [] for col in expected_cols},
            schema={
                "name_one": pl.Utf8,
                "name_two": pl.Utf8,
                "club": pl.Utf8,
                "number": pl.Int64,
                "ranks": pl.List(pl.Int64),
            },
        )
    part_path = os.path.join(comp_dir, "participants.parquet")
    part_df.write_parquet(part_path)
    logging.info("Saved participants to %s", part_path)
    validate_schema(part_path, expected_cols)


def save_judges(event_name: str, judges: List[Judge]) -> None:
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    expected_cols = ["code", "name", "club"]
    if judges:
        judges_df = pl.DataFrame(
            [{**j.model_dump(), "name": j.name} for j in judges]
        )
        for col in expected_cols:
            if col not in judges_df.columns:
                judges_df = judges_df.with_columns(pl.lit(None).alias(col))
        judges_df = judges_df.select(expected_cols)
    else:
        judges_df = pl.DataFrame(
            {col: [] for col in expected_cols},
            schema={"code": pl.Utf8, "name": pl.Utf8, "club": pl.Utf8},
        )
    judges_path = os.path.join(comp_dir, "judges.parquet")
    judges_df.write_parquet(judges_path)
    logging.info("Saved judges to %s", judges_path)
    validate_schema(judges_path, expected_cols)


def save_committee(event_name: str, committee: List[CommitteeMember]) -> None:
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    expected_cols = ["role", "name", "club"]
    if committee:
        committee_df = pl.DataFrame(
            [{**c.model_dump(), "name": c.name} for c in committee]
        )
        for col in expected_cols:
            if col not in committee_df.columns:
                committee_df = committee_df.with_columns(pl.lit(None).alias(col))
        committee_df = committee_df.select(expected_cols)
    else:
        committee_df = pl.DataFrame(
            {col: [] for col in expected_cols},
            schema={"role": pl.Utf8, "name": pl.Utf8, "club": pl.Utf8},
        )
    committee_path = os.path.join(comp_dir, "committee.parquet")
    committee_df.write_parquet(committee_path)
    logging.info("Saved committee to %s", committee_path)
    validate_schema(committee_path, expected_cols)


def save_scores(event_name: str, scores: List[Any]) -> None:
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
            entry["number"] = int(entry["number"]) if entry["number"] is not None else None
        except (ValueError, TypeError):
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
    logging.info("Saved scores to %s", scores_path)
    validate_schema(scores_path, expected_cols)


def save_final_scoring(event_name: str, final_scores: List[FinalScoring]) -> None:
    comp_dir = os.path.join(DATA_DIR, event_name)
    os.makedirs(comp_dir, exist_ok=True)
    expected_cols = [
        "placement",
        "names",
        "number",
        "club",
        "scores",
        "total",
    ]
    if final_scores:
        final_list = []
        for f in final_scores:
            row = f.model_dump()
            scores = row.pop("scores", {})
            for dance, score in scores.items():
                row[f"score_{dance.value}"] = score
            final_list.append(row)
        final_df = pl.DataFrame(final_list)
        # Dynamically create expected columns based on actual data
        dynamic_cols = [c for c in final_df.columns if c.startswith("score_")]
        all_expected_cols = expected_cols[:-2] + dynamic_cols + ["total"]

        for col in all_expected_cols:
            if col not in final_df.columns:
                final_df = final_df.with_columns(pl.lit(None).alias(col))
        final_df = final_df.select(all_expected_cols)
    else:
        final_df = pl.DataFrame(
            {col: [] for col in expected_cols},
            schema={
                "placement": pl.Utf8,
                "names": pl.Utf8,
                "number": pl.Utf8,
                "club": pl.Utf8,
                "scores": pl.Utf8,
                "total": pl.Utf8,
            },
        )
    final_path = os.path.join(comp_dir, "final_scoring.parquet")
    final_df.write_parquet(final_path)
    logging.info("Saved final scoring to %s", final_path)
    if "scores" not in final_df.columns:
        final_df = final_df.with_columns(pl.lit(None, dtype=pl.Utf8).alias("scores"))
    validate_schema(final_path, expected_cols)
