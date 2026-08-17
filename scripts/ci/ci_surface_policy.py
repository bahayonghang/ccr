#!/usr/bin/env python3
"""Resolve which hosted CI product surfaces a pull request affects."""

from __future__ import annotations

import argparse
import fnmatch
import os
import subprocess
import sys
from pathlib import Path
from typing import Iterable

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

SURFACE_PATHS: dict[str, tuple[str, ...]] = {
    "root": (
        "crates/**",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "justfile",
        "scripts/**",
        ".github/workflows/**",
        ".github/dependabot.yml",
    ),
    "frontend": (
        "ccr-ui/**",
        "docs/**",
        "justfile",
        "scripts/ci/ci_surface_policy.py",
        ".github/workflows/frontend-ci.yml",
    ),
    "tauri": (
        "ccr-ui/src-tauri/**",
        "ccr-ui/src/types/generated/**",
        "ccr-ui/justfile",
        "crates/**",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "justfile",
        "scripts/**",
        "docs/reference/tauri-command-inventory.md",
        ".trellis/spec/ccr/backend/**",
        ".github/workflows/tauri-rust-ci.yml",
    ),
    "vscode": (
        "ccr-vscode/**",
        "justfile",
        "scripts/ci/ci_surface_policy.py",
        ".github/workflows/vscode-ci.yml",
    ),
}


def path_matches(path: str, patterns: Iterable[str]) -> bool:
    normalized = path.replace("\\", "/")
    return any(fnmatch.fnmatchcase(normalized, pattern) for pattern in patterns)


def is_relevant(surface: str, paths: Iterable[str]) -> bool:
    return any(path_matches(path, SURFACE_PATHS[surface]) for path in paths)


def changed_paths(base: str, head: str) -> list[str]:
    result = subprocess.run(
        ["git", "diff", "--name-only", "-z", f"{base}...{head}"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    )
    return [os.fsdecode(path) for path in result.stdout.split(b"\0") if path]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--surface", choices=sorted(SURFACE_PATHS), required=True)
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    args = parser.parse_args()

    relevant = is_relevant(args.surface, changed_paths(args.base, args.head))
    print(f"relevant={str(relevant).lower()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
