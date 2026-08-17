#!/usr/bin/env python3
"""Enforce overall and security-gateway line coverage from llvm-cov JSON."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from scripts.common import REPO_ROOT
except ModuleNotFoundError:
    # pywin32 在 site-packages 注册了同名 namespace package `scripts`
    _scripts_dir = Path(__file__).resolve().parent
    while _scripts_dir.name != "scripts":
        _scripts_dir = _scripts_dir.parent
    sys.path.insert(0, str(_scripts_dir.parent))
    sys.modules.pop("scripts", None)
    from scripts.common import REPO_ROOT


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("report", type=Path)
    parser.add_argument("--overall", type=float)
    parser.add_argument("--gateway", type=float, required=True)
    parser.add_argument("--gateway-pattern", action="append", required=True)
    args = parser.parse_args()

    report = args.report if args.report.is_absolute() else REPO_ROOT / args.report
    payload = json.loads(report.read_text(encoding="utf-8"))
    data = payload.get("data", [])
    if len(data) != 1:
        print("coverage report must contain exactly one data record", file=sys.stderr)
        return 1
    report = data[0]
    overall = float(report["totals"]["lines"]["percent"])
    failures: list[str] = []
    if args.overall is not None and overall < args.overall:
        failures.append(f"overall line coverage {overall:.2f}% < {args.overall:.2f}%")

    matched: dict[str, float] = {}
    for item in report.get("files", []):
        filename = str(item.get("filename", "")).replace("\\", "/")
        if any(pattern in filename for pattern in args.gateway_pattern):
            matched[filename] = float(item["summary"]["lines"]["percent"])
    if not matched:
        failures.append("no security-gateway files matched the configured patterns")
    for filename, percent in sorted(matched.items()):
        if percent < args.gateway:
            failures.append(
                f"security gateway {filename} line coverage {percent:.2f}% < {args.gateway:.2f}%"
            )

    print(f"Overall line coverage: {overall:.2f}%")
    for filename, percent in sorted(matched.items()):
        print(f"Gateway line coverage: {percent:.2f}% {filename}")
    if failures:
        for failure in failures:
            print(f"ERROR: {failure}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
