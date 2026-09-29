"""One exclusive formal bindings check with exact before/after evidence."""
import base64
import datetime
import difflib
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

sys.stdout.reconfigure(encoding="utf-8")
ROOT = Path(__file__).resolve().parents[3]
TASK = Path(__file__).resolve().parent
GENERATED = ROOT / "ccr-ui/src/types/generated"


def snapshot():
    return {
        path.relative_to(GENERATED).as_posix(): path.read_bytes()
        for path in sorted(GENERATED.rglob("*"))
        if path.is_file()
    }


def save_snapshot(name, files):
    data = {
        path: {"sha256": hashlib.sha256(raw).hexdigest(), "base64": base64.b64encode(raw).decode()}
        for path, raw in files.items()
    }
    (TASK / name).write_text(json.dumps(data, indent=2), encoding="utf-8")


before = snapshot()
save_snapshot("check-bindings-before.json", before)
env = os.environ.copy()
env.update(CCR_SKIP_ICON_GENERATION="1", CARGO_NET_OFFLINE="true")
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
with (TASK / "check-bindings.log").open("wb") as log:
    result = subprocess.run(["just.exe", "tauri-bindings-check"], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
after = snapshot()
save_snapshot("check-bindings-after.json", after)
changed = [path for path in sorted(before.keys() | after.keys()) if before.get(path) != after.get(path)]
diff = []
for path in changed:
    diff.extend(difflib.unified_diff(
        before.get(path, b"").decode("utf-8", errors="replace").splitlines(keepends=True),
        after.get(path, b"").decode("utf-8", errors="replace").splitlines(keepends=True),
        fromfile="before/" + path,
        tofile="after/" + path,
    ))
(TASK / "check-bindings.diff").write_text("".join(diff), encoding="utf-8")
summary = {"started_utc": started, "finished_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "command": "just tauri-bindings-check", "exit_code": result.returncode, "files_before": len(before), "files_after": len(after), "byte_changed_paths": changed}
(TASK / "check-bindings-result.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
print(json.dumps(summary))
print("".join((TASK / "check-bindings.log").read_text(encoding="utf-8", errors="replace").splitlines(keepends=True)[-18:]))
sys.exit(result.returncode)
