"""Verify T04 with explicitly selected, deliverable source files."""

from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
from tempfile import TemporaryDirectory
from time import monotonic

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent / "evidence"
EVIDENCE.mkdir(parents=True, exist_ok=True)
CHECKER = "scripts/quality/check-copilot-assets.mjs"
TEST = "scripts/quality/check-copilot-assets.test.mjs"
DOCS = ["docs/guide/github-copilot-workspace.md", "docs/en/guide/github-copilot-workspace.md"]
SHARED = ["AGENTS.md", ".codex/skills/ccr-gate-recovery/SKILL.md", ".codex/skills/ccr-ui-visual-workflow/SKILL.md"]
ASSETS = [
    ".github/copilot-instructions.md",
    *[f".github/instructions/{name}.instructions.md" for name in ("rust", "ui", "docs")],
    *[f".github/prompts/{name}-change.prompt.md" for name in ("rust", "ui", "docs")],
    *[f".github/agents/{name}.agent.md" for name in ("researcher", "implementer", "reviewer")],
]
SOURCES = [".gitignore", CHECKER, TEST, *DOCS, *SHARED, *ASSETS]
NODE = shutil.which("node")
JUST = shutil.which("just")
if NODE is None or JUST is None:
    raise SystemExit("Node and just are required")
checks = []


def run(name, args, cwd=ROOT):
    started = monotonic()
    result = subprocess.run(args, cwd=cwd, text=True, encoding="utf-8", errors="replace", capture_output=True, timeout=60, check=False)
    log = EVIDENCE / f"{name}.log"
    log.write_text(result.stdout + result.stderr, encoding="utf-8", newline="\n")
    record = {"name": name, "command": args, "cwd": str(cwd), "exit_code": result.returncode,
              "status": "PASS" if result.returncode == 0 else "FAIL", "seconds": round(monotonic() - started, 3),
              "log": log.name, "log_sha256": hashlib.sha256(log.read_bytes()).hexdigest()}
    checks.append(record)
    return result, record


selection, selected_check = run("selected-sources", ["git", "ls-files", "--cached", "--others", "--exclude-standard", "--", *SOURCES])
selected = selection.stdout.splitlines()
if set(selected) != set(SOURCES):
    raise SystemExit("The explicit delivery candidate contains a missing or ignored source")
run("worktree-copilot-check", [JUST, "copilot-check"])
run("checker-syntax", [NODE, "--check", CHECKER])
run("tests-syntax", [NODE, "--check", TEST])

with TemporaryDirectory(prefix="ccr-copilot-selected-source-") as temp:
    fixture = Path(temp).resolve()
    for source in selected:
        target = fixture / source
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / source, target)
    run("fixture-git-init", ["git", "init", "--quiet"], fixture)
    run("fixture-track-selected", ["git", "-c", "core.autocrlf=true", "add", "--force", "--", *selected], fixture)
    result, record = run("selected-source-assets", [NODE, CHECKER], fixture)
    record["claude_directory_present"] = (fixture / ".claude").exists()
    record["copied_files"] = sorted(selected)
    record["scope"] = "Working-tree bytes selected by Git; temporary Git index only; not a committed full clone or native client run."
    if record["claude_directory_present"]:
        record["status"] = "FAIL"
    run("selected-source-tests", [NODE, "--test", TEST], fixture)

run("tracked-diff-check", ["git", "diff", "--check", "--", CHECKER, *DOCS])
result, record = run("new-test-diff-check", ["git", "-c", "core.autocrlf=false",
    "-c", "core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol",
    "diff", "--no-index", "--check", "--", os.devnull, TEST])
record["status"] = "PASS" if result.returncode in (0, 1) and not result.stdout.strip() and not result.stderr.strip() else "FAIL"
record["exit_code_meaning"] = "No-index returns 1 for a new file; PASS also requires no whitespace diagnostics."
run("task-context-validate", ["python", ".trellis/scripts/task.py", "validate", ".trellis/tasks/09-29-harness-copilot-crlf"])
report = {
    "recorded_at_utc": datetime.now(timezone.utc).isoformat(),
    "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "os": platform.platform(),
    "node_version": subprocess.check_output([NODE, "--version"], cwd=ROOT, text=True).strip(),
    "core_autocrlf": subprocess.check_output(["git", "config", "--get", "core.autocrlf"], cwd=ROOT, text=True).strip(),
    "changed_source_sha256": {file: hashlib.sha256((ROOT / file).read_bytes()).hexdigest() for file in [CHECKER, TEST, *DOCS]},
    "checks": checks,
    "earlier_tool_receipts": [
        {"command": "node --test scripts/quality/check-copilot-assets.test.mjs", "exit_code": 0, "pass": 34, "fail": 0, "skipped": 0, "duration_ms": 13787.1032, "tool_session_id": 30546},
        {"command": "just docs-check", "exit_code": 0, "summary": "Frozen install made no changes; docs audit, VitePress build, and ccr-ui docs audit passed.", "tool_chunk_id": "abb6fc"},
    ],
    "native_clients": "UNVERIFIED",
}
(EVIDENCE / "verification.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"checks": [{key: check[key] for key in ("name", "status", "exit_code")} for check in checks], "selected_file_count": len(selected)}, indent=2))
raise SystemExit(0 if all(check["status"] == "PASS" for check in checks) else 1)
