#!/usr/bin/env python3
"""Resolve which hosted CI product surfaces a pull request affects.

README files and prose under ``docs/`` do not make a surface relevant.
Workflow files deliberately have no ``pull_request.paths`` filter: a skipped
workflow never reports its required check, so the pull request stays blocked.
The always-on relevance job reports success when the heavy jobs are skipped.
"""

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
        ".cargo/config.toml",
        ".cargo/audit.toml",
        "justfile",
        "scripts/**",
        ".github/workflows/**",
        ".github/dependabot.yml",
        ".github/copilot-instructions.md",
        ".github/instructions/**",
        ".github/prompts/**",
        ".github/agents/**",
        ".github/skills/**",
        "AGENTS.md",
        "CLAUDE.md",
        ".trellis/workflow.md",
        "docs/agents/harnesses.md",
        "docs/en/agents/harnesses.md",
        "docs/guide/github-copilot-workspace.md",
        "docs/en/guide/github-copilot-workspace.md",
        "ccr-ui/package.json",
        "ccr-ui/bun.lock",
        "ccr-ui/src-tauri/Cargo.lock",
        "docs/package.json",
        "docs/bun.lock",
        "ccr-vscode/package.json",
        "ccr-vscode/package-lock.json",
        "ccr-vscode/justfile",
    ),
    "frontend": (
        "ccr-ui/**",
        "docs/package.json",
        "docs/bun.lock",
        "docs/justfile",
        "docs/.vitepress/**",
        "docs/scripts/**",
        "justfile",
        "scripts/ci/ci_surface_policy.py",
        "scripts/ci/check_workflow_governance.py",
        "scripts/ci/test_check_workflow_governance.py",
        "scripts/ci/frontend_check.py",
        "scripts/ci/frontend_install.py",
        "scripts/ci/install_prebuilt_tool.py",
        "scripts/ci/install_prebuilt_tool.sh",
        "scripts/ci/test_install_prebuilt_tool.py",
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
        ".cargo/tauri-ci.toml",
        ".cargo/config.toml",
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
        "scripts/ci/check_workflow_governance.py",
        "scripts/ci/test_check_workflow_governance.py",
        "scripts/ci/install_prebuilt_tool.py",
        "scripts/ci/install_prebuilt_tool.sh",
        "scripts/ci/test_install_prebuilt_tool.py",
        ".github/workflows/vscode-ci.yml",
    ),
}


def path_matches(path: str, patterns: Iterable[str]) -> bool:
    normalized = path.replace("\\", "/")
    return any(fnmatch.fnmatchcase(normalized, pattern) for pattern in patterns)


def is_readme(path: str) -> bool:
    """README-only edits are documentation and must not select a product surface."""
    name = path.replace("\\", "/").rsplit("/", 1)[-1].lower()
    return name in {"readme.md", "readme_cn.md"}


def is_relevant(surface: str, paths: Iterable[str]) -> bool:
    return any(
        path_matches(path, SURFACE_PATHS[surface])
        for path in paths
        if not is_readme(path)
    )


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
