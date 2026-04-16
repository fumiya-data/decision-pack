#!/usr/bin/env python3
"""Validate simulation report JSON against a JSON Schema."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from jsonschema import Draft202012Validator


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate report JSON with JSON Schema")
    parser.add_argument("--schema", required=True, type=Path, help="Path to JSON Schema file")
    parser.add_argument("--input", required=True, type=Path, help="Path to JSON file to validate")
    args = parser.parse_args()

    schema = json.loads(args.schema.read_text(encoding="utf-8"))
    payload = json.loads(args.input.read_text(encoding="utf-8"))

    validator = Draft202012Validator(schema)
    errors = sorted(validator.iter_errors(payload), key=lambda e: e.json_path)

    if not errors:
        print(f"OK: {args.input} is valid against {args.schema}")
        return 0

    print(f"NG: {args.input} is NOT valid against {args.schema}")
    for err in errors:
        print(f"- {err.json_path}: {err.message}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
