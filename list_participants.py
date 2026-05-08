#!/usr/bin/env python3
"""List all participants from competitions in a folder, optionally filtered by club."""

import argparse
import json
from pathlib import Path


def load_participants(folder: Path) -> list[dict]:
    participants = []
    for json_file in sorted(folder.glob("*/*.json")):
        with open(json_file) as f:
            data = json.load(f)
        competition = data.get("name", json_file.parent.name)
        for p in data.get("participants", []):
            participants.append({
                "competition": competition,
                "name_one": p.get("name_one", ""),
                "name_two": p.get("name_two", ""),
                "club": p.get("affiliation", ""),
                "rank": p.get("final_rank"),
            })
    return participants


def main():
    parser = argparse.ArgumentParser(description="List participants from downloaded competitions.")
    parser.add_argument("folder", nargs="?", default="bb2026", help="Folder with competition data (default: bb2026)")
    parser.add_argument("--club", help="Filter by club name (case-insensitive substring match)")
    parser.add_argument("--name", help="Filter by participant name (case-insensitive substring match, searches both names)")
    args = parser.parse_args()

    folder = Path(args.folder)
    if not folder.exists():
        print(f"Folder not found: {folder}")
        return

    participants = load_participants(folder)

    if args.club:
        query = args.club.lower()
        participants = [p for p in participants if query in p["club"].lower()]

    if args.name:
        query = args.name.lower()
        participants = [p for p in participants if query in p["name_one"].lower() or query in p["name_two"].lower()]

    if not participants:
        print("No participants found.")
        return

    current_competition = None
    for p in participants:
        if p["competition"] != current_competition:
            current_competition = p["competition"]
            print(f"\n=== {current_competition} ===")
        rank = f"#{p['rank']}" if p["rank"] is not None else "  "
        names = p["name_one"]
        if p["name_two"]:
            names += f" / {p['name_two']}"
        print(f"  {rank:4s}  {names:<45}  {p['club']}")


if __name__ == "__main__":
    main()
