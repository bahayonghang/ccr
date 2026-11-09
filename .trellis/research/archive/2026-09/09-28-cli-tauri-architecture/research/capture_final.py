"""Record final integration commands without changing their exit status."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

sys.stdout.reconfigure(encoding="utf-8")
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
CHECKS = {
    "version": (ROOT, ["just", "version-check"]),
    "format": (ROOT, ["just", "fmt-check"]),
    "strict": (ROOT, ["just", "lint-strict"]),
    "workspace": (ROOT, ["just", "check-workspace"]),
    "test": (ROOT, ["just", "test"]),
    "tauri": (ROOT, ["just", "tauri-ci"]),
    "bindings": (ROOT, ["just", "tauri-bindings-check"]),
    "frontend": (ROOT, ["just", "frontend-check"]),
    "frontend-tests": (ROOT / "ccr-ui", ["bun", "run", "test"]),
    "coverage": (ROOT, ["just", "frontend-coverage"]),
    "vscode": (ROOT, ["just", "vscode-ci"]),
    "docs": (ROOT, ["just", "docs-check"]),
    "ci": (ROOT, ["just", "ci"]),
}

parser = argparse.ArgumentParser()
parser.add_argument("checks", nargs="+", choices=CHECKS)
parser.add_argument("--prefix", default="root-final")
args = parser.parse_args()
failed = False
for key in args.checks:
    cwd, command = CHECKS[key]
    argv = [shutil.which(command[0]) or command[0], *command[1:]]
    started = time.monotonic()
    timestamp = datetime.now(timezone.utc).isoformat()
    log = HERE / (args.prefix + "-" + key + ".log")
    env = dict(os.environ, CCR_SKIP_ICON_GENERATION="1")
    print(json.dumps({"starting": key, "command": command, "log": str(log)}), flush=True)
    with log.open("wb") as stream:
        result = subprocess.run(argv, cwd=cwd, env=env, stdout=stream, stderr=subprocess.STDOUT, check=False)
    record = {
        "check": key, "command": command, "cwd": str(cwd),
        "started_at": timestamp, "exit_code": result.returncode,
        "seconds": round(time.monotonic() - started, 2), "log": log.name,
        "command_environment_override": {"CCR_SKIP_ICON_GENERATION": "1"},
    }
    (HERE / (args.prefix + "-" + key + ".json")).write_text(
        json.dumps(record, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")
    print(json.dumps(record, ensure_ascii=False), flush=True)
    print(chr(10).join(log.read_text(encoding="utf-8", errors="replace").splitlines()[-15:]), flush=True)
    failed = failed or result.returncode != 0
sys.exit(1 if failed else 0)
