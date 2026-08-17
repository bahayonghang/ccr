#!/usr/bin/env python3
"""Check or rewrite the repository's human-authored JSON configuration files."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable

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

# Package/config manifests are included. Locks, generated bindings, JSONC
# (including tsconfig files), data catalogs, third-party assets, and
# whitespace-sensitive fixtures are excluded.
JSON_CONFIG_PATHS = (
    ".mcp.json",
    "ccr-ui/.stylelintrc.json",
    "ccr-ui/package.json",
    "ccr-ui/scripts/dev-warm-targets.json",
    "ccr-ui/src-tauri/capabilities/codex-tray-panel.json",
    "ccr-ui/src-tauri/capabilities/default.json",
    "ccr-ui/src-tauri/capabilities/main.json",
    "ccr-ui/src-tauri/tauri.conf.json",
    "ccr-vscode/package.json",
    "docs/package.json",
    "scripts/drift/dependency-drift-allowlist.json",
)


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, indent=2) + "\n"


def process_json_configs(
    root: Path = REPO_ROOT,
    paths: Iterable[str] = JSON_CONFIG_PATHS,
    *,
    write: bool = False,
) -> list[str]:
    failures: list[str] = []
    for relative in paths:
        path = root / relative
        if not path.is_file():
            failures.append(f"missing configured JSON file: {relative}")
            continue
        try:
            source = path.read_text(encoding="utf-8")
            formatted = canonical_json(json.loads(source))
        except (OSError, UnicodeError, json.JSONDecodeError) as error:
            failures.append(f"{relative}: {error}")
            continue
        if source == formatted:
            continue
        if write:
            path.write_text(formatted, encoding="utf-8")
        else:
            failures.append(f"noncanonical JSON formatting: {relative}")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    failures = process_json_configs(write=args.write)
    if failures:
        print("JSON format check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    mode = "formatted" if args.write else "checked"
    print(f"JSON configuration formatting {mode}: {len(JSON_CONFIG_PATHS)} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
