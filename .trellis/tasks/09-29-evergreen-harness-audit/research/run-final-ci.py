"""Record the real aggregate result and source-byte preservation."""

from __future__ import annotations

import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time


def source_snapshot(root: Path) -> dict[str, str]:
    output = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root,
    )
    paths = sorted(set(output.decode("utf-8").rstrip("\0").split("\0")))
    result = {}
    for name in paths:
        if name.startswith((".trellis/tasks/", ".trellis/workspace/")):
            continue
        path = root / name
        result[name] = (
            hashlib.sha256(path.read_bytes()).hexdigest()
            if path.is_file()
            else "MISSING"
        )
    return result


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    root = Path(__file__).resolve().parents[4]
    evidence = Path(__file__).resolve().parent
    prefix = sys.argv[1] if len(sys.argv) > 1 else "final-ci"
    before = source_snapshot(root)
    (evidence / f"{prefix}-source-before.json").write_text(
        json.dumps(before, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    timer = time.monotonic()
    env = os.environ.copy()
    env["RUSTUP_AUTO_INSTALL"] = "0"
    log = evidence / f"{prefix}.log"
    print(json.dumps({"started": started, "command": ["just", "ci"], "source_files": len(before)}), flush=True)
    with log.open("wb") as stream:
        process = subprocess.run(
            ["just", "ci"], cwd=root, env=env, stdout=stream, stderr=subprocess.STDOUT
        )
    after = source_snapshot(root)
    (evidence / f"{prefix}-source-after.json").write_text(
        json.dumps(after, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    changed = [name for name in sorted(before.keys() | after.keys()) if before.get(name) != after.get(name)]
    receipt = {
        "command": ["just", "ci"],
        "started_at_utc": started,
        "duration_seconds": round(time.monotonic() - timer, 2),
        "exit_code": process.returncode,
        "os": "Windows local",
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "source_files_before": len(before),
        "source_files_after": len(after),
        "changed_source_paths": changed,
        "source_bytes_preserved": not changed,
        "snapshot_scope": "Tracked and unignored deliverable files; task evidence and developer journals excluded. Ignored build outputs are outside the source contract.",
        "environment_overrides": {"RUSTUP_AUTO_INSTALL": "0"},
        "log": log.name,
        "log_sha256": hashlib.sha256(log.read_bytes()).hexdigest(),
        "boundary": "Actual local aggregate only. Separate coverage, MSRV, native client and hosted results retain their own receipts.",
    }
    (evidence / f"{prefix}.json").write_text(
        json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(receipt, ensure_ascii=False, indent=2), flush=True)
    print(log.read_text(encoding="utf-8", errors="replace")[-5000:], flush=True)
    return process.returncode or (1 if changed else 0)


if __name__ == "__main__":
    raise SystemExit(main())
