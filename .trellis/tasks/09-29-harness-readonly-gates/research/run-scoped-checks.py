"""Record T06 scoped checks and hashes without running the full repository CI."""
from __future__ import annotations

import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
sys.stderr.reconfigure(encoding="utf-8", errors="replace")
ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = (
    "justfile", ".github/workflows/ci.yml",
    ".github/workflows/frontend-ci.yml", ".github/workflows/vscode-ci.yml",
    "scripts/ci/ci_surface_policy.py", "scripts/ci/check_workflow_governance.py",
    "scripts/ci/test_check_workflow_governance.py",
)


def hashes() -> dict[str, str]:
    return {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in SOURCES}


def main() -> int:
    commands = [
        ("02-governance-tests", [sys.executable, "-m", "unittest", "scripts.ci.test_check_workflow_governance", "scripts.ci.test_architecture_contract_gates"]),
        ("03-workflow-governance", ["just", "workflow-governance-check"]),
        ("04-actionlint", ["actionlint"]),
        ("05-omp-check", ["just", "omp-check"]),
        ("06-harness-check", ["just", "harness-check"]),
        ("07-copilot-check", ["just", "copilot-check"]),
        ("08-frontend-audit", ["just", "frontend-audit"]),
        ("09-cargo-audit", ["just", "audit"]),
        ("10-diff-check", ["git", "diff", "--check", "--", *SOURCES]),
    ]
    report = {
        "started_utc": datetime.now(timezone.utc).isoformat(),
        "platform": platform.platform(),
        "python": sys.version,
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "scope": "T06 scoped local checks; full just ci and hosted/native gates are owned by the parent task",
        "sources_before": hashes(),
        "checks": [],
    }
    env = {**os.environ, "PYTHONIOENCODING": "utf-8"}
    for label, command in commands:
        resolved = [shutil.which(command[0]) or command[0], *command[1:]]
        start = time.monotonic()
        with (EVIDENCE / f"{label}.log").open("wb") as log:
            result = subprocess.run(resolved, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        check = {"name": label, "command": command, "resolved_command": resolved,
                 "exit_code": result.returncode, "seconds": round(time.monotonic() - start, 3)}
        report["checks"].append(check)
        print(json.dumps(check, ensure_ascii=False), flush=True)
        (EVIDENCE / "scoped-check-results.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    report["sources_after"] = hashes()
    report["source_drift"] = [path for path in SOURCES if report["sources_before"][path] != report["sources_after"][path]]
    report["finished_utc"] = datetime.now(timezone.utc).isoformat()
    (EVIDENCE / "scoped-check-results.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"source_drift": report["source_drift"]}), flush=True)
    return int(bool(report["source_drift"]) or any(check["exit_code"] for check in report["checks"]))


if __name__ == "__main__":
    raise SystemExit(main())
