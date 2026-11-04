#!/usr/bin/env python3
"""Install ccr-ui dependencies once per frontend-check invocation."""

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


def main() -> int:
    if os.environ.get("CCR_FRONTEND_DEPS") == "1":
        print("ccr-ui dependencies already installed for this frontend-check")
        return 0
    subprocess.run(
        ["bun", "install", "--frozen-lockfile"],
        cwd=REPO_ROOT / "ccr-ui",
        check=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
