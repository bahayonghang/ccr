"""Capture non-fixing architecture audit checks in the task research folder."""
from __future__ import annotations

import argparse
import json
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
    "workflow": (ROOT, ["python", "scripts/ci/check_workflow_governance.py"]),
    "secrets": (ROOT, ["python", "scripts/quality/check_secret_writes.py"]),
    "types": (ROOT / "ccr-ui", ["bun", "run", "type-check"]),
    "cycles": (ROOT / "ccr-ui", ["bun", "run", "check:cycles"]),
    "boundaries": (ROOT / "ccr-ui", ["bun", "run", "check:arch-boundaries"]),
    "lint": (ROOT / "ccr-ui", ["bun", "run", "lint:ci"]),
    "lint-tracked-diagnostic": (ROOT / "ccr-ui", ["bun", "node_modules/eslint/bin/eslint.js", ".", "--quiet", "--ignore-pattern", ".tmp-desktop-probe.mjs", "--ignore-pattern", ".tmp-insights-visual.mjs"]),
    "styles": (ROOT / "ccr-ui", ["bun", "run", "lint:style"]),
    "style-lines": (ROOT / "ccr-ui", ["bun", "run", "check:style-lines"]),
}

parser = argparse.ArgumentParser()
parser.add_argument("checks", nargs="+", choices=CHECKS)
args = parser.parse_args()
for key in args.checks:
    cwd, command = CHECKS[key]
    argv = [shutil.which(command[0]) or command[0], *command[1:]]
    started = time.monotonic()
    log = HERE / ("baseline-" + key + ".log")
    with log.open("wb") as stream:
        result = subprocess.run(argv, cwd=cwd, stdout=stream, stderr=subprocess.STDOUT, check=False)
    record = {"check": key, "command": command, "cwd": str(cwd),
              "exit_code": result.returncode, "seconds": round(time.monotonic() - started, 2),
              "log": log.name}
    (HERE / ("baseline-" + key + ".json")).write_text(
        json.dumps(record, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")
    print(json.dumps(record, ensure_ascii=False), flush=True)
    output = log.read_text(encoding="utf-8", errors="replace").splitlines()
    print(chr(10).join(output[-12:]), flush=True)
