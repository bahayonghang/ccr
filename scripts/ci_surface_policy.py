#!/usr/bin/env python3
"""Resolve which hosted CI product surfaces a pull request affects."""

from __future__ import annotations

import argparse
import fnmatch
import os
import subprocess
from pathlib import Path
from typing import Iterable


ROOT = Path(__file__).resolve().parents[1]

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
        "scripts/ci_surface_policy.py",
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
        "scripts/ci_surface_policy.py",
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
        cwd=ROOT,
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
