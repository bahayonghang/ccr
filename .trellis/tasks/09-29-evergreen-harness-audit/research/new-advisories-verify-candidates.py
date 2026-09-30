"""Verify proposed tarballs and save a review-only lock diff."""
import base64
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import difflib
import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile
import urllib.request

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
proposal = json.loads((OUT / "new-advisories-version-proposal.json").read_text(encoding="utf-8"))
capture = json.loads((OUT / "new-advisories-capture.json").read_text(encoding="utf-8"))


def fetch(url, data=None):
    headers = {"User-Agent": "CCR-approved-harness-audit-diagnostics"}
    if data is not None:
        headers["Content-Type"] = "application/json"
    with urllib.request.urlopen(urllib.request.Request(url, data=data, headers=headers), timeout=45) as response:
        return response.read()


def verify(change):
    raw = fetch(change["tarball"])
    integrity = "sha512-" + base64.b64encode(hashlib.sha512(raw).digest()).decode()
    assert integrity == change["new_integrity"]
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz") as archive:
        package = json.loads(archive.extractfile("package/package.json").read())
    assert package["name"] == change["package"]
    assert package["version"] == change["new_version"]
    assert package.get("dependencies", {}) == change["new_metadata"].get("dependencies", {})
    return {"package": change["package"], "version": change["new_version"],
            "source": change["tarball"], "fetched_utc": datetime.now(timezone.utc).isoformat(),
            "bytes": len(raw), "verified_sha512": integrity,
            "package_metadata": {key: package.get(key) for key in ("name", "version", "dependencies", "engines", "main", "exports", "type")}}


unique = {(change["package"], change["new_version"]): change for change in proposal["changes"]}
with ThreadPoolExecutor(max_workers=3) as pool:
    verified = list(pool.map(verify, unique.values()))
query = {}
for name, version in unique:
    query.setdefault(name, []).append(version)
bulk_url = "https://registry.npmjs.org/-/npm/v1/security/advisories/bulk"
bulk_raw = fetch(bulk_url, json.dumps(query).encode())
(OUT / "new-advisories-candidate-bulk-response.json").write_bytes(bulk_raw)
report = {"verified_tarballs": verified, "bulk_source": bulk_url, "bulk_query": query,
          "bulk_fetched_utc": datetime.now(timezone.utc).isoformat(), "bulk_response": json.loads(bulk_raw)}
assert report["bulk_response"] == {}

patches = []
for file in sorted({change["file"] for change in proposal["changes"]}):
    before = (ROOT / file).read_text(encoding="utf-8")
    changes = [change for change in proposal["changes"] if change["file"] == file]
    if file.endswith("bun.lock"):
        after = before
        for change in changes:
            prefix = '    ' + json.dumps(change["node"]) + ': ['
            line = next(line for line in after.splitlines() if line.startswith(prefix))
            replacement = line.replace(change["package"] + "@" + change["old_version"], change["package"] + "@" + change["new_version"]).replace(change["old_integrity"], change["new_integrity"])
            assert replacement != line
            after = after.replace(line, replacement, 1)
    else:
        lock = json.loads(before)
        for change in changes:
            node = lock["packages"][change["node"]]
            node.update(version=change["new_version"], resolved=change["tarball"], integrity=change["new_integrity"])
        after = json.dumps(lock, indent=2, ensure_ascii=False) + "\n"
    patches.extend(difflib.unified_diff(before.splitlines(keepends=True), after.splitlines(keepends=True), fromfile="a/" + file, tofile="b/" + file))
(OUT / "new-advisories-proposed-locks.patch").write_text("".join(patches), encoding="utf-8", newline="\n")
report["source_hashes_after"] = {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in capture["sources_before"]}
report["changed_source_paths"] = [p for p in capture["sources_before"] if capture["sources_before"][p] != report["source_hashes_after"][p]]
assert not report["changed_source_paths"]
(OUT / "new-advisories-candidate-verification.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report, ensure_ascii=False, indent=2))
