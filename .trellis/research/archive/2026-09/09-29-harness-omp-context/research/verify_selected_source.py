"""Record T01 local checks without using ignored files in the source fixture."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
from tempfile import TemporaryDirectory
from time import monotonic
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent / "evidence"
SOURCES = [
    ".gitignore",
    ".omp/extensions/trellis/index.ts",
    "scripts/trellis/omp-context.test.ts",
]
EVIDENCE.mkdir(parents=True, exist_ok=True)
BUN = shutil.which("bun")
if BUN is None:
    raise SystemExit("Bun is required")
checks = []


def run(name: str, args: list[str], cwd: Path = ROOT, input_text: str | None = None):
    started = monotonic()
    result = subprocess.run(
        args, cwd=cwd, input=input_text, text=True, encoding="utf-8",
        errors="replace", stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        timeout=60, check=False,
    )
    output = result.stdout + result.stderr
    log = EVIDENCE / f"{name}.log"
    log.write_text(output, encoding="utf-8", newline="\n")
    check = {
        "name": name,
        "command": args,
        "cwd": str(cwd),
        "exit_code": result.returncode,
        "seconds": round(monotonic() - started, 3),
        "log": log.name,
        "log_sha256": hashlib.sha256(log.read_bytes()).hexdigest(),
        "status": "PASS" if result.returncode == 0 else "FAIL",
    }
    checks.append(check)
    return result, check


selection, selection_check = run(
    "selected-sources",
    ["git", "ls-files", "--cached", "--others", "--exclude-standard", "--", *SOURCES],
)
selected = selection.stdout.splitlines()
if set(selected) != set(SOURCES):
    selection_check["status"] = "FAIL"
    raise SystemExit(f"Expected explicit sources {SOURCES}, received {selected}")

worktree, worktree_check = run("worktree-bun", [BUN, "test", SOURCES[2]])
source_text = (ROOT / SOURCES[1]).read_text(encoding="utf-8")
imports = re.findall(r'^import (type )?.*? from "([^"]+)";', source_text, re.MULTILINE)
runtime_imports = [module for type_only, module in imports if not type_only]
type_imports = [module for type_only, module in imports if type_only]
if not runtime_imports or any(not module.startswith("node:") for module in runtime_imports):
    raise SystemExit("Unexpected runtime import dependency")

probe_paths = [
    SOURCES[1],
    ".omp/settings.local.json",
    ".omp/agents/trellis-implement.md",
    ".omp/extensions/other/index.ts",
    ".omp/extensions/trellis/local-config.json",
    ".omp/extensions/trellis/cache/data.json",
    "nested/.omp/private.json",
]
ignored, ignore_check = run(
    "ignore-boundary", ["git", "check-ignore", "--no-index", "--stdin", "-z"],
    input_text="\0".join(probe_paths) + "\0",
)
ignored_paths = [path for path in ignored.stdout.split("\0") if path]
ignore_check["status"] = "PASS" if set(ignored_paths) == set(probe_paths[1:]) else "FAIL"

with TemporaryDirectory(prefix="ccr-omp-selected-source-") as temp:
    fixture = Path(temp).resolve()
    for source in selected:
        target = fixture / source
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / source, target)
    fixture_result, fixture_check = run(
        "selected-source-bun", [BUN, "test", SOURCES[2]], fixture,
    )
    fixture_check["copied_files"] = sorted(selected)
    fixture_check["node_modules_present"] = (fixture / "node_modules").exists()
    fixture_check["scope"] = "Uncommitted working-tree bytes selected by Git; not a committed clone or native OMP session."
    bundled = fixture / "extension.js"
    build, build_check = run(
        "runtime-build", [BUN, "build", SOURCES[1], "--target=bun", "--outfile", str(bundled)], fixture,
    )
    if build.returncode == 0:
        build_check["type_only_import_erased"] = "@oh-my-pi/pi-coding-agent" not in bundled.read_text(encoding="utf-8")
        if not build_check["type_only_import_erased"]:
            build_check["status"] = "FAIL"

run("tracked-diff-check", ["git", "diff", "--check", "--", SOURCES[0], SOURCES[2]])
new_source_diff, new_source_check = run("new-source-diff-check", [
    "git", "-c", "core.autocrlf=false",
    "-c", "core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol",
    "diff", "--no-index", "--check", "--", os.devnull, SOURCES[1],
])
new_source_check["status"] = "PASS" if (
    new_source_diff.returncode in (0, 1)
    and not new_source_diff.stdout.strip()
    and not new_source_diff.stderr.strip()
) else "FAIL"
new_source_check["exit_code_meaning"] = "No-index diff returns 1 for a new file; PASS also requires no whitespace diagnostics."
run("task-context-validate", [
    "python", ".trellis/scripts/task.py", "validate", ".trellis/tasks/09-29-harness-omp-context",
])

report = {
    "recorded_at_utc": datetime.now(timezone.utc).isoformat(),
    "os": platform.platform(),
    "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "bun_version": subprocess.check_output([BUN, "--version"], cwd=ROOT, text=True).strip(),
    "source_sha256": {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in SOURCES},
    "runtime_imports": runtime_imports,
    "type_only_imports": type_imports,
    "ignored_probe_paths": ignored_paths,
    "checks": checks,
    "native_omp_runtime": "UNVERIFIED",
}
(EVIDENCE / "verification.json").write_text(
    json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n",
)
print(json.dumps({
    "checks": [{"name": check["name"], "status": check["status"], "exit_code": check["exit_code"]} for check in checks],
    "runtime_imports": runtime_imports,
    "sources": selected,
}, ensure_ascii=False, indent=2))
raise SystemExit(0 if all(check["status"] == "PASS" for check in checks) else 1)
