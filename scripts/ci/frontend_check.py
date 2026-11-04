#!/usr/bin/env python3
"""Run the frontend gate with one bun install.

``frontend-build`` still installs again because its recipe line is the
hosted/local contract. A warm install returns immediately.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

try:
    from scripts.common import REPO_ROOT
except ModuleNotFoundError:
    _scripts_dir = Path(__file__).resolve().parent
    while _scripts_dir.name != "scripts":
        _scripts_dir = _scripts_dir.parent
    sys.path.insert(0, str(_scripts_dir.parent))
    sys.modules.pop("scripts", None)
    from scripts.common import REPO_ROOT

RECIPES = (
    "frontend-typecheck",
    "frontend-lint",
    "frontend-check-cycles",
    "frontend-check-arch-boundaries",
    "frontend-test",
    "frontend-build",
    "docs-check",
)


def main() -> int:
    env = os.environ.copy()
    env["CCR_FRONTEND_DEPS"] = "1"
    subprocess.run(
        ["bun", "install", "--frozen-lockfile"],
        cwd=REPO_ROOT / "ccr-ui",
        check=True,
    )
    for recipe in RECIPES:
        subprocess.run(["just", recipe], cwd=REPO_ROOT, env=env, check=True)
    subprocess.run(
        ["just", "success", "前端检查全部通过"],
        cwd=REPO_ROOT,
        env=env,
        check=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
