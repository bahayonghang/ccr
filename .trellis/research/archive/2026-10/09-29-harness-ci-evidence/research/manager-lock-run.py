"""Run one authorized manager diagnostic command and preserve separate evidence."""
from __future__ import annotations

import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

RESEARCH = Path(__file__).resolve().parent
ROOT = RESEARCH.parents[3]
SOURCES = [
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/ccr/Cargo.toml",
    "crates/ccr/tests/managers.rs",
    "crates/ccr/tests/managers/legacy_registry.rs",
    "crates/ccr/tests/managers/general.rs",
    "crates/ccr/tests/support/env.rs",
    "crates/ccr-config/src/managers/platform_config.rs",
    "crates/ccr-config/src/managers/config/manager.rs",
    "crates/ccr-config/src/managers/config_file_handler.rs",
    "crates/ccr-cli/src/managers/settings.rs",
    "crates/ccr-core/src/core/fileio.rs",
    "crates/ccr-core/src/core/guarded_write.rs",
    "crates/ccr-core/src/core/lock.rs",
]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def hashes() -> dict[str, str]:
    return {name: digest(ROOT / name) for name in SOURCES}


def save_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")


variant = sys.argv[1]
base = ["cargo", "test", "-p", "ccr", "--all-features", "--test", "managers"]
variants = {
    "exact": base + ["legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth", "--", "--exact", "--skip", "export_bindings"],
    "suite": base + ["--", "--skip", "export_bindings"],
    "legacy": base + ["legacy_registry::", "--", "--skip", "export_bindings"],
}
command = variants[variant]
receipt = RESEARCH / f"manager-lock-{variant}.json"
if receipt.exists():
    raise SystemExit("Refusing to replace an existing diagnostic receipt")
fixture = RESEARCH / f"manager-lock-fixture-{variant}"
fixture.mkdir(exist_ok=False)
(fixture / "locks").mkdir()
(fixture / "ccr-root").mkdir()
environment = os.environ.copy()
overrides = {
    "CCR_ROOT": str(fixture / "ccr-root"),
    "CCR_LOCK_DIR": str(fixture / "locks"),
    "RUSTUP_AUTO_INSTALL": "0",
}
environment.update(overrides)
before = hashes()
save_json(RESEARCH / f"manager-lock-{variant}-hashes-before.json", before)
stdout_path = RESEARCH / f"manager-lock-{variant}.stdout.log"
stderr_path = RESEARCH / f"manager-lock-{variant}.stderr.log"
started = dt.datetime.now(dt.timezone.utc).isoformat()
started_perf = time.monotonic()
with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
    result = subprocess.run(command, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr, check=False)
after = hashes()
save_json(RESEARCH / f"manager-lock-{variant}-hashes-after.json", after)
data = {
    "command": command,
    "working_directory": str(ROOT),
    "started_at_utc": started,
    "duration_seconds": round(time.monotonic() - started_perf, 3),
    "exit_code": result.returncode,
    "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "os": "Windows local",
    "child_process_environment_overrides": overrides,
    "global_environment_changed": False,
    "parallelism": "Cargo and Rust test harness defaults; no thread override",
    "retries": 0,
    "source_hash_count": len(before),
    "source_hashes_equal": before == after,
    "changed_source_paths": [name for name in before if before[name] != after[name]],
    "stdout": stdout_path.name,
    "stdout_sha256": digest(stdout_path),
    "stderr": stderr_path.name,
    "stderr_sha256": digest(stderr_path),
    "fixture_files": sorted(str(path.relative_to(fixture)) for path in fixture.rglob("*") if path.is_file()),
    "boundary": "One narrow local diagnostic execution. No source edits. No claim of full CI, hosted, or other OS acceptance.",
}
save_json(receipt, data)
print(json.dumps(data, ensure_ascii=False, indent=2))
print(stdout_path.read_text(encoding="utf-8", errors="replace"))
print(stderr_path.read_text(encoding="utf-8", errors="replace"))
raise SystemExit(result.returncode)
