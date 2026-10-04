"""Capture fresh advisory evidence without editing manifests or lockfiles."""
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
SOURCES = ("ccr-ui/bun.lock", "ccr-ui/package.json", "docs/bun.lock", "docs/package.json", "ccr-vscode/package-lock.json", "ccr-vscode/package.json")


def snapshot():
    return {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in SOURCES}


def save(name, data):
    (OUT / ("new-advisories-" + name + ".json")).write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def fetch(name, url):
    started = datetime.now(timezone.utc).isoformat()
    try:
        request = urllib.request.Request(url, headers={"User-Agent": "CCR-approved-harness-audit-diagnostics", "Accept": "application/json"})
        with urllib.request.urlopen(request, timeout=45) as response:
            raw = response.read()
            record = {"name": name, "source": url, "requested_utc": started, "fetched_utc": datetime.now(timezone.utc).isoformat(), "http_status": response.status, "date": response.headers.get("Date"), "last_modified": response.headers.get("Last-Modified"), "etag": response.headers.get("ETag"), "sha256": hashlib.sha256(raw).hexdigest()}
        json.loads(raw)
        (OUT / ("new-advisories-" + name + ".json")).write_bytes(raw)
        return record
    except Exception as error:
        return {"name": name, "source": url, "requested_utc": started, "error": repr(error)}


def audit(surface, command):
    started = time.monotonic()
    executable = shutil.which(command[0])
    result = subprocess.run([executable, *command[1:]], cwd=ROOT / surface, capture_output=True, timeout=90)
    (OUT / f"new-advisories-{surface}-audit.stdout.json").write_bytes(result.stdout)
    (OUT / f"new-advisories-{surface}-audit.stderr.log").write_bytes(result.stderr)
    return {"name": surface + "-audit", "command": command, "executable": executable, "cwd": surface, "exit_code": result.returncode, "seconds": round(time.monotonic() - started, 3), "stdout_sha256": hashlib.sha256(result.stdout).hexdigest()}


def main():
    report = {"started_utc": datetime.now(timezone.utc).isoformat(), "sources_before": snapshot(), "results": []}
    with ThreadPoolExecutor(max_workers=6) as pool:
        futures = [pool.submit(fetch, ghsa, "https://api.github.com/advisories/" + ghsa) for ghsa in ("GHSA-qhr7-859c-m2p7", "GHSA-6j4f-fj2g-mc7p", "GHSA-q2hr-2g5m-vwhr", "GHSA-hrr3-gc8f-f4qj")]
        futures += [pool.submit(fetch, package + "-registry", "https://registry.npmjs.org/" + package) for package in ("brace-expansion", "fast-uri")]
        futures += [pool.submit(audit, surface, command) for surface, command in (("ccr-ui", ["bun", "audit", "--json"]), ("docs", ["bun", "audit", "--json"]), ("ccr-vscode", ["npm", "audit", "--json"]))]
        for future in as_completed(futures):
            try:
                result = future.result()
            except Exception as error:
                result = {"error": repr(error)}
            report["results"].append(result)
            print(json.dumps(result, ensure_ascii=False), flush=True)
            save("capture", report)
    report["sources_after"] = snapshot()
    report["changed_source_paths"] = [p for p in SOURCES if report["sources_before"][p] != report["sources_after"][p]]
    report["finished_utc"] = datetime.now(timezone.utc).isoformat()
    save("capture", report)
    print(json.dumps({"changed_source_paths": report["changed_source_paths"]}), flush=True)


if __name__ == "__main__":
    main()
