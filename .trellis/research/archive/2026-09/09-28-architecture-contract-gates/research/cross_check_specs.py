"""Independent read-only verification of the T10 spec author's artifacts."""
from datetime import datetime
from pathlib import Path
import hashlib
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
errors = []
artifacts = {}
selectors = []


def load(relative):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))


def check(condition, message):
    if not condition:
        errors.append(message)


def audit_artifacts(value):
    if isinstance(value, list):
        for entry in value:
            audit_artifacts(entry)
    elif isinstance(value, dict):
        if isinstance(value.get("path"), str) and isinstance(value.get("sha256"), str):
            name = value["path"]
            path = ROOT / name
            check(path.is_file(), f"Missing artifact: {name}")
            if path.is_file():
                data = path.read_bytes()
                digest = hashlib.sha256(data).hexdigest()
                check(digest == value["sha256"], f"SHA mismatch: {name}")
                if "bytes" in value:
                    check(len(data) == value["bytes"], f"Size mismatch: {name}")
                artifacts[name] = {"sha256": digest, "bytes": len(data)}
        for entry in value.values():
            audit_artifacts(entry)


matrix_path = ".trellis/tasks/09-28-architecture-contract-gates/research/requirements-evidence.json"
ledger_path = ".trellis/tasks/09-28-cli-tauri-architecture/research/p1-evidence-ledger.json"
matrix = load(matrix_path)
ledger = load(ledger_path)
spec = load(".trellis/tasks/09-28-architecture-contract-gates/research/spec-validation.json")
requirements = matrix["requirements"]
ids = [row["id"] for row in requirements]
check(len(ids) == 39 and len(set(ids)) == 39, "Expected 39 unique ACs")
check(len(matrix["test_groups"]) == 33, "Expected 33 behavior evidence groups including Codex backend settings")
check(len(ledger["findings"]) == 9, "Expected 9 P1 mappings")
check(matrix["fix_commit"] is None, "Unexpected fix commit")
for requirement in requirements:
    for group in requirement["test_groups"]:
        check(group in matrix["test_groups"], f"Missing group: {group}")
    if requirement.get("report"):
        check((ROOT / requirement["report"]).is_file(), f"Missing report: {requirement['id']}")
for key, group in matrix["test_groups"].items():
    lines = (ROOT / group["test_source"]["path"]).read_text(encoding="utf-8").splitlines()
    for case in group["cases"]:
        matched = 0 < case["line"] <= len(lines) and case["selector"] in lines[case["line"] - 1]
        check(matched, f"Selector mismatch: {key}/{case['selector']}")
        selectors.append({"group": key, **case, "matched": matched})
for finding in ledger["findings"]:
    check(finding["fix_commit"] is None, f"Unexpected P1 commit: {finding['finding']}")
    check(bool(finding["final_behavior_test_mapping"]), f"Missing behavior mapping: {finding['finding']}")
    for requirement in finding["requirement_refs"]:
        check(requirement in ids, f"Unknown P1 requirement: {requirement}")
audit_artifacts(matrix)
audit_artifacts(ledger)
audit_artifacts(spec)

links = []
for entry in spec["owned_files"]:
    path = ROOT / entry["path"]
    data = path.read_bytes()
    if entry["path"].startswith(".trellis/spec/"):
        check(len(data) <= 32768, f"Context byte limit: {entry['path']}")
    for chunk in data.decode("utf-8").split("](")[1:]:
        target = chunk.split(")", 1)[0].split("#", 1)[0]
        if not target or "://" in target or not target.startswith("."):
            continue
        check((path.parent / target).exists(), f"Broken link: {entry['path']} -> {target}")
        links.append({"source": entry["path"], "target": target})
manifest = load("ccr-ui/src/api/generated/command-manifest.json")
counts = {key: manifest[key] for key in ("base_command_count", "windows_command_count", "typed_command_count", "exact_wire_type_count")}
check(tuple(counts.values()) == (340, 348, 278, 278), "Manifest counts differ")
registry = (ROOT / "ccr-ui/src-tauri/src/commands/handler_registry.rs").read_text(encoding="utf-8")
check("assert_eq!(COMMAND_MODULES.len(), 38);" in registry, "Registry module count differs")
context_results = []
for task in ("09-28-usage-job-lifecycle", "09-28-architecture-contract-gates"):
    command = [sys.executable, ".trellis/scripts/task.py", "validate", ".trellis/tasks/" + task]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace")
    output = result.stdout + result.stderr
    log = "cross-spec-context-" + task + ".log"
    (OUT / log).write_text(output, encoding="utf-8")
    check(result.returncode == 0 and "exceeds" not in output.lower(), f"Context validation failed: {task}")
    context_results.append({"task": task, "exit_code": result.returncode, "log": log})

baseline = matrix["baseline_commit"]
probe = OUT / "baseline-repro"
mapper_name = "ccr-ui/src/configs/settings-codex-map.ts"
helper_name = "ccr-ui/src/configs/settings-helpers.ts"
mapper = subprocess.check_output(["git", "show", baseline + ":" + mapper_name], cwd=ROOT)
helper = subprocess.check_output(["git", "show", baseline + ":" + helper_name], cwd=ROOT)
normalized_mapper = (probe / "settings-codex-map-baseline.ts").read_bytes().replace(bytes([13, 10]), bytes([10]))
normalized_helper = (probe / "settings-helpers-baseline.ts").read_bytes().replace(bytes([13, 10]), bytes([10]))
expected_mapper = mapper.replace(b"@/configs/settings-helpers", b"./settings-helpers-baseline")
check(normalized_mapper == expected_mapper, "A14 probe changed baseline mapper logic")
check(normalized_helper == helper, "A14 probe changed baseline helper logic")
a14 = json.loads((probe / "a14-baseline.json").read_text(encoding="utf-8"))
check(a14["exit_code"] == 1, "A14 no longer records a red execution")
check(a14["original_mapper_sha256"] == hashlib.sha256(mapper).hexdigest(), "A14 mapper baseline SHA mismatch")
check(a14["original_helper_sha256"] == hashlib.sha256(helper).hexdigest(), "A14 helper baseline SHA mismatch")
a14_log = (probe / a14["log"]).read_text(encoding="utf-8")
check("0 pass" in a14_log and "1 fail" in a14_log, "A14 raw failure log missing")
gateway = subprocess.check_output(["git", "show", baseline + ":ccr-ui/src-tauri/src/process/gateway.rs"], cwd=ROOT)
gateway_sha_lf = hashlib.sha256(gateway).hexdigest()
gateway_sha_crlf = hashlib.sha256(gateway.replace(bytes([10]), bytes([13, 10]))).hexdigest()
a09 = json.loads((probe / "a09-instrumentation.json").read_text(encoding="utf-8"))
check(a09["gateway_original_sha256"] in (gateway_sha_lf, gateway_sha_crlf), "A09 gateway original SHA mismatch")

output = {
    "timestamp": datetime.now().astimezone().isoformat(),
    "reviewer": "/root/implement_t10_gates",
    "reviewed_author": "/root/implement_t10_specs",
    "status": "passed_scoped" if not errors else "needs_correction",
    "requirements": len(ids), "test_groups": len(matrix["test_groups"]),
    "p1_groups": len(ledger["findings"]),
    "artifacts_checked": artifacts, "selectors_checked": selectors,
    "relative_links_checked": links, "manifest_counts": counts,
    "context_results": context_results,
    "baseline_review": {
        "a14_mapper_matches_blob_except_runtime_import": normalized_mapper == expected_mapper,
        "a14_helper_matches_blob": normalized_helper == helper,
        "a14_log_passed": 0, "a14_log_failed": 1,
        "a09_gateway_blob_lf_sha256": gateway_sha_lf,
        "a09_gateway_checkout_crlf_sha256": gateway_sha_crlf,
        "limits": "A14 checks mapper behavior with original helper, without native transport. A09 compares baseline identity only; separate execution evidence belongs to the root.",
    },
    "p1_red_statuses": {finding["finding"]: finding["red_execution_status"] for finding in ledger["findings"]},
    "matrix_sha256": hashlib.sha256((ROOT / matrix_path).read_bytes()).hexdigest(),
    "ledger_sha256": hashlib.sha256((ROOT / ledger_path).read_bytes()).hexdigest(),
    "errors": sorted(set(errors)),
    "limits": "Verifies captured paths, hashes, source selectors, scope and context loadability. Does not execute product, Cargo, exports, native UI, or replace final root gates. Root-owned clean regression assertions require the root focused rerun.",
}
(OUT / "cross-check-specs.json").write_text(json.dumps(output, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")
print(json.dumps({"status": output["status"], "artifacts": len(artifacts), "selectors": len(selectors), "links": len(links), "errors": output["errors"]}, ensure_ascii=False))
sys.exit(0 if not errors else 1)
