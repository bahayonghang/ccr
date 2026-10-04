"""Run the fixed approved local gates and preserve fresh evidence."""
from __future__ import annotations

import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tempfile
import time

RESEARCH = Path(__file__).resolve().parent
ROOT = RESEARCH.parents[3]
PREFIX = "resume-2026-10-04-manager"
HISTORICAL = json.loads((RESEARCH / "manager-lock-exact-hashes-before.json").read_text(encoding="utf-8"))
JSON_PATH = ".trellis/tasks/09-29-harness-ci-evidence/research/tauri-advisory-commands.json"
CANDIDATE = RESEARCH / "manager-lock-candidate.patch"
EXPECTED_CANDIDATE = "a7082c421018a9b54167b3069ec3000cc21210bdebe3ca135cf6d7f52e775738"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def hashes() -> dict[str, str]:
    return {name: digest(ROOT / name) for name in HISTORICAL}


def save(path: Path, value: object) -> None:
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def head() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()


variant = sys.argv[1]
receipt = RESEARCH / f"{PREFIX}-{variant}.json"
if receipt.exists():
    raise SystemExit("Refusing to replace an existing receipt")
if variant in {"baseline", "ignore"}:
    current = hashes()
    historical_comparisons = {}
    for name in ["exact", "suite"]:
        for phase in ["before", "after"]:
            previous = json.loads((RESEARCH / f"manager-lock-{name}-hashes-{phase}.json").read_text(encoding="utf-8"))
            historical_comparisons[f"{name}-{phase}"] = [path for path in current if current[path] != previous[path]]
    ignore = subprocess.run(["git", "check-ignore", "-v", "--", JSON_PATH], cwd=ROOT, capture_output=True, text=True, check=False)
    data = {
        "recorded_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "head": head(),
        "os": platform.platform(),
        "candidate_sha256": digest(CANDIDATE),
        "candidate_matches_approved_sha": digest(CANDIDATE) == EXPECTED_CANDIDATE,
        "diagnostic_input_count": len(current),
        "diagnostic_input_sha256": current,
        "differences_from_historical_inputs": historical_comparisons,
        "diagnostic_json_sha256": digest(ROOT / JSON_PATH),
        "diagnostic_json_bytes": (ROOT / JSON_PATH).stat().st_size,
        "check_ignore_argv": ["git", "check-ignore", "-v", "--", JSON_PATH],
        "check_ignore_exit_code": ignore.returncode,
        "check_ignore_stdout": ignore.stdout,
        "check_ignore_stderr": ignore.stderr,
        "historical_evidence_preserved": True,
    }
    if variant == "ignore":
        baseline = json.loads((RESEARCH / f"{PREFIX}-baseline.json").read_text(encoding="utf-8"))
        data["diagnostic_json_sha256_unchanged"] = data["diagnostic_json_sha256"] == baseline["diagnostic_json_sha256"]
    save(receipt, data)
    print(json.dumps(data, ensure_ascii=False, indent=2))
    raise SystemExit(0)

base = ["cargo", "test", "-p", "ccr", "--all-features", "--test", "managers"]
commands = {
    "exact": base + ["legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth", "--", "--exact", "--skip", "export_bindings"],
    "suite": base + ["--", "--skip", "export_bindings"],
    "platforms": ["cargo", "test", "-p", "ccr", "--test", "platforms", "--", "--nocapture", "--skip", "export_bindings"],
    "commands": ["cargo", "test", "-p", "ccr", "--test", "commands", "--", "sync_content", "--nocapture", "--skip", "export_bindings"],
    "default": ["cargo", "test", "-p", "ccr", "--test", "managers", "--", "--nocapture", "--skip", "export_bindings"],
    "clippy": ["cargo", "clippy", "-p", "ccr", "--all-targets", "--all-features", "--", "-D", "warnings"],
}
command = commands[variant]
stdout_path = RESEARCH / f"{PREFIX}-{variant}.stdout.log"
stderr_path = RESEARCH / f"{PREFIX}-{variant}.stderr.log"
before = hashes()
started = dt.datetime.now(dt.timezone.utc).isoformat()
started_head = head()
started_perf = time.monotonic()
with tempfile.TemporaryDirectory(prefix=f"{PREFIX}-{variant}-") as fixture_name:
    fixture = Path(fixture_name)
    (fixture / "locks").mkdir()
    (fixture / "ccr-root").mkdir()
    environment = os.environ.copy()
    overrides = {"CCR_ROOT": str(fixture / "ccr-root"), "CCR_LOCK_DIR": str(fixture / "locks"), "RUSTUP_AUTO_INSTALL": "0"}
    environment.update(overrides)
    with stdout_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
        result = subprocess.run(command, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr, check=False)
    fixture_files = sorted(str(path.relative_to(fixture)) for path in fixture.rglob("*") if path.is_file())
after = hashes()
output = stdout_path.read_text(encoding="utf-8", errors="replace")
counts = [{"status": status, "passed": int(passed), "failed": int(failed), "ignored": int(ignored), "measured": int(measured), "filtered_out": int(filtered)} for status, passed, failed, ignored, measured, filtered in re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", output)]
data = {
    "command": command,
    "working_directory": str(ROOT),
    "started_at_utc": started,
    "finished_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
    "duration_seconds": round(time.monotonic() - started_perf, 3),
    "exit_code": result.returncode,
    "status": "PASS" if result.returncode == 0 else "FAILED",
    "head_at_start": started_head,
    "head_at_end": head(),
    "os": platform.platform(),
    "child_process_environment_overrides": overrides,
    "global_environment_changed": False,
    "parallelism": "Cargo and Rust test harness defaults; no thread override",
    "retries": 0,
    "diagnostic_input_count": len(before),
    "source_hashes_before": before,
    "source_hashes_after": after,
    "source_hashes_equal": before == after,
    "changed_source_paths": [name for name in before if before[name] != after[name]],
    "stdout": stdout_path.name,
    "stdout_sha256": digest(stdout_path),
    "stdout_bytes": stdout_path.stat().st_size,
    "stderr": stderr_path.name,
    "stderr_sha256": digest(stderr_path),
    "stderr_bytes": stderr_path.stat().st_size,
    "test_counts": counts,
    "fixture_files_before_cleanup": fixture_files,
    "fixture_cleanup_complete": not fixture.exists(),
    "boundary": "Approved Windows local gate. No full CI, hosted, macOS, or fresh-client acceptance claim. Historical failure cause remains undetermined.",
}
save(receipt, data)
print(json.dumps({key: data[key] for key in ["command", "status", "exit_code", "duration_seconds", "test_counts", "source_hashes_equal", "stdout_sha256", "stderr_sha256"]}, ensure_ascii=False, indent=2))
raise SystemExit(result.returncode)
