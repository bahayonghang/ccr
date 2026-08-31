#!/usr/bin/env python3
"""Fail closed on mutable GitHub Actions and hosted/local CI drift."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

try:
    from scripts.common import REPO_ROOT
except ModuleNotFoundError:
    # pywin32 在 site-packages 注册了同名 namespace package `scripts`
    _scripts_dir = Path(__file__).resolve().parent
    while _scripts_dir.name != "scripts":
        _scripts_dir = _scripts_dir.parent
    sys.path.insert(0, str(_scripts_dir.parent))
    sys.modules.pop("scripts", None)
    from scripts.common import REPO_ROOT

try:
    from scripts.ci.ci_surface_policy import SURFACE_PATHS
except ModuleNotFoundError:
    from ci_surface_policy import SURFACE_PATHS


WORKFLOW_DIR = REPO_ROOT / ".github" / "workflows"
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
USES_RE = re.compile(r"^\s*-?\s*uses:\s*([^\s#]+)", re.MULTILINE)
RUST_TOOLCHAIN_RE = re.compile(r"^\s*toolchain:\s*([^\s#]+)", re.MULTILINE)
BUN_VERSION_RE = re.compile(r"^\s*bun-version:\s*([^\s#]+)", re.MULTILINE)
NODE_VERSION_RE = re.compile(r"^\s*node-version:\s*([^\s#]+)", re.MULTILINE)
SETUP_NODE_USES_RE = re.compile(
    r"^(?P<indent> *)(?P<sequence>-\s+)?uses:\s*actions/setup-node@[^\s#]+"
)
MAPPING_KEY_RE = re.compile(
    r"^(?P<indent> *)(?P<sequence>-\s+)?(?P<key>[A-Za-z_][A-Za-z0-9_-]*):(?P<value>.*)$"
)
SERIAL_TEST_RE = re.compile(r"#\[\s*(?:serial|serial_test::serial)(?:\([^]]*\))?\s*\]")
MAX_SERIAL_TESTS = 0
REQUIRED_BRANCHES = {"main", "develop", "dev"}
DEVELOPMENT_RUST_TOOLCHAIN = "1.98.0"
MSRV_RUST_TOOLCHAIN = "1.95.0"
EXPECTED_BUN_WORKFLOWS = {"frontend-ci.yml", "release.yml", "tauri-rust-ci.yml"}
NODE_TOOLCHAIN = "24.20.0"
EXPECTED_NODE_WORKFLOW_INPUTS = {"release.yml": 2, "vscode-ci.yml": 1}


def duplicate_mapping_keys(text: str) -> list[tuple[int, str]]:
    """Detect duplicate keys in the workflow YAML subset without external packages."""
    seen_by_indent: dict[int, set[str]] = {}
    block_scalar_indent: int | None = None
    duplicates: list[tuple[int, str]] = []
    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        if not raw_line.strip() or raw_line.lstrip().startswith("#"):
            continue
        indent = len(raw_line) - len(raw_line.lstrip(" "))
        if block_scalar_indent is not None:
            if indent > block_scalar_indent:
                continue
            block_scalar_indent = None
        match = MAPPING_KEY_RE.match(raw_line)
        if match is None:
            continue
        is_sequence_item = match.group("sequence") is not None
        effective_indent = indent + 2 if is_sequence_item else indent
        cutoff = effective_indent if is_sequence_item else effective_indent + 1
        for level in [level for level in seen_by_indent if level >= cutoff]:
            del seen_by_indent[level]
        key = match.group("key")
        seen = seen_by_indent.setdefault(effective_indent, set())
        if key in seen:
            duplicates.append((line_number, key))
        seen.add(key)
        if match.group("value").strip().startswith(("|", ">")):
            block_scalar_indent = effective_indent
    return duplicates


def workflow_event_values(text: str, event: str, key: str) -> set[str]:
    """Read a branches/paths sequence from the conventional workflow layout."""
    lines = text.splitlines()
    on_index = next((index for index, line in enumerate(lines) if line == "on:"), None)
    if on_index is None:
        return set()
    event_index: int | None = None
    for index in range(on_index + 1, len(lines)):
        line = lines[index]
        if line and not line.startswith(" "):
            break
        if line == f"  {event}:":
            event_index = index
            break
    if event_index is None:
        return set()
    key_index: int | None = None
    inline_value = ""
    for index in range(event_index + 1, len(lines)):
        line = lines[index]
        if line.startswith("  ") and not line.startswith("    ") and line.strip():
            break
        prefix = f"    {key}:"
        if line.startswith(prefix):
            key_index = index
            inline_value = line[len(prefix) :].strip()
            break
    if key_index is None:
        return set()
    if inline_value.startswith("[") and inline_value.endswith("]"):
        return {
            value.strip().strip("'\"")
            for value in inline_value[1:-1].split(",")
            if value.strip()
        }
    values: set[str] = set()
    for line in lines[key_index + 1 :]:
        if line.startswith("      - "):
            values.add(line.removeprefix("      - ").strip().strip("'\""))
            continue
        if line.strip() and not line.startswith("      "):
            break
    return values


def workflow_job_block(text: str, job_id: str) -> str:
    lines = text.splitlines()
    start = next(
        (index for index, line in enumerate(lines) if line == f"  {job_id}:"),
        None,
    )
    if start is None:
        return ""
    end = next(
        (
            index
            for index in range(start + 1, len(lines))
            if lines[index].startswith("  ")
            and not lines[index].startswith("    ")
            and lines[index].strip()
        ),
        len(lines),
    )
    return "\n".join(lines[start:end])


def rust_toolchain_inputs(text: str) -> list[str]:
    return RUST_TOOLCHAIN_RE.findall(text)


def bun_version_inputs(text: str) -> list[str]:
    return BUN_VERSION_RE.findall(text)


def node_version_inputs(text: str) -> list[str]:
    return NODE_VERSION_RE.findall(text)


def setup_node_version_inputs(text: str) -> list[list[str]]:
    """Return node-version inputs grouped by each actions/setup-node step."""
    lines = text.splitlines()
    inputs_by_step: list[list[str]] = []
    for index, line in enumerate(lines):
        match = SETUP_NODE_USES_RE.match(line)
        if match is None:
            continue
        uses_indent = len(match.group("indent"))
        step_indent = uses_indent if match.group("sequence") else max(uses_indent - 2, 0)
        end = len(lines)
        for candidate in range(index + 1, len(lines)):
            candidate_line = lines[candidate]
            if not candidate_line.strip() or candidate_line.lstrip().startswith("#"):
                continue
            candidate_indent = len(candidate_line) - len(candidate_line.lstrip(" "))
            if candidate_indent <= step_indent:
                end = candidate
                break
        inputs_by_step.append(node_version_inputs("\n".join(lines[index:end])))
    return inputs_by_step


def node_pin_failures(workflows: dict[str, str]) -> list[str]:
    failures: list[str] = []
    for name, expected_count in sorted(EXPECTED_NODE_WORKFLOW_INPUTS.items()):
        inputs = setup_node_version_inputs(workflows.get(name, ""))
        expected = [[NODE_TOOLCHAIN]] * expected_count
        if inputs != expected:
            failures.append(
                f"{name}: expected {expected_count} Node {NODE_TOOLCHAIN} setup "
                f"input(s); found {inputs or 'none'}"
            )
    for name, workflow in sorted(workflows.items()):
        if name in EXPECTED_NODE_WORKFLOW_INPUTS:
            continue
        inputs = setup_node_version_inputs(workflow)
        if inputs:
            failures.append(
                f"{name}: unexpected Node setup input outside governed workflows: "
                f"{inputs}"
            )
    return failures


def canonical_bun_version(root: Path = REPO_ROOT) -> str:
    package_path = root / "ccr-ui" / "package.json"
    package = json.loads(package_path.read_text(encoding="utf-8"))
    package_manager = str(package.get("packageManager") or "").strip()
    match = re.fullmatch(r"bun@(\d+\.\d+\.\d+)", package_manager)
    if match is None:
        raise ValueError(
            "ccr-ui/package.json#packageManager must be an exact bun@x.y.z pin"
        )
    return match.group(1)


def main() -> int:
    failures: list[str] = []
    workflow_paths = sorted(
        path for path in WORKFLOW_DIR.iterdir() if path.suffix in {".yml", ".yaml"}
    )
    workflows = {
        path.name: path.read_text(encoding="utf-8") for path in workflow_paths
    }
    required = {"ci.yml", "frontend-ci.yml", "release.yml", "tauri-rust-ci.yml", "vscode-ci.yml"}
    missing = sorted(required - workflows.keys())
    if missing:
        failures.append(f"missing workflows: {', '.join(missing)}")

    bun_version: str | None = None
    try:
        bun_version = canonical_bun_version()
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as error:
        failures.append(f"canonical Bun pin is invalid: {error}")

    for name in sorted(required & workflows.keys()):
        workflow = workflows[name]
        ordinary_workflow = workflow
        if name == "ci.yml":
            msrv_job = workflow_job_block(workflow, "workspace-msrv")
            if not msrv_job:
                failures.append("ci.yml: missing explicit workspace-msrv job")
            else:
                if rust_toolchain_inputs(msrv_job) != [MSRV_RUST_TOOLCHAIN]:
                    failures.append(
                        "ci.yml: workspace-msrv must use exactly Rust 1.95.0"
                    )
                if "cargo check --workspace --all-targets --all-features" not in msrv_job:
                    failures.append(
                        "ci.yml: workspace-msrv must check all workspace targets and features"
                    )
                ordinary_workflow = workflow.replace(msrv_job, "", 1)
        ordinary_toolchains = rust_toolchain_inputs(ordinary_workflow)
        if not ordinary_toolchains:
            failures.append(f"{name}: missing Rust {DEVELOPMENT_RUST_TOOLCHAIN} toolchain input")
        unexpected_toolchains = sorted(
            {
                toolchain
                for toolchain in ordinary_toolchains
                if toolchain != DEVELOPMENT_RUST_TOOLCHAIN
            }
        )
        if unexpected_toolchains:
            failures.append(
                f"{name}: ordinary Rust jobs must use {DEVELOPMENT_RUST_TOOLCHAIN}; "
                f"found {', '.join(unexpected_toolchains)}"
            )

    action_count = 0
    for name, text in sorted(workflows.items()):
        for line_number, key in duplicate_mapping_keys(text):
            failures.append(f"{name}:{line_number}: duplicate YAML mapping key '{key}'")
        for reference in USES_RE.findall(text):
            if reference.startswith("./"):
                continue
            if "@" not in reference:
                failures.append(f"{name}: non-local action ref is missing an immutable SHA: {reference}")
                continue
            action, ref = reference.rsplit("@", maxsplit=1)
            action_count += 1
            if not SHA_RE.fullmatch(ref):
                failures.append(f"{name}: mutable action ref {action}@{ref}")

    governed_surfaces = {
        "ci.yml": "root",
        "frontend-ci.yml": "frontend",
        "tauri-rust-ci.yml": "tauri",
        "vscode-ci.yml": "vscode",
    }
    required_gates = {
        "ci.yml": ("root-required", "Root Workspace Required"),
        "frontend-ci.yml": ("frontend-required-gate", "Vue and Docs Required"),
        "tauri-rust-ci.yml": ("tauri-required", "Tauri Linux Required"),
        "vscode-ci.yml": ("vscode-required-gate", "VS Code Required"),
    }
    for name, surface in governed_surfaces.items():
        text = workflows.get(name, "")
        branches = workflow_event_values(text, "pull_request", "branches")
        if branches != REQUIRED_BRANCHES:
            failures.append(f"{name}: pull_request branches must cover main/develop/dev")
        paths = workflow_event_values(text, "pull_request", "paths")
        if paths:
            failures.append(
                f"{name}: pull_request paths must be delegated to the required-check relevance job"
            )
        if not SURFACE_PATHS.get(surface):
            failures.append(f"{name}: missing path policy for CI surface {surface}")
        if f"--surface {surface}" not in text:
            failures.append(f"{name}: missing relevance detection for CI surface {surface}")
        push_branches = workflow_event_values(text, "push", "branches")
        if push_branches:
            failures.append(f"{name}: quality workflows must be pull_request-only")
        gate_id, context_name = required_gates[name]
        gate = workflow_job_block(text, gate_id)
        if not gate:
            failures.append(f"{name}: missing stable required gate {gate_id}")
        else:
            if f"name: {context_name}" not in gate:
                failures.append(f"{name}: required context must be named {context_name}")
            if "if: ${{ always() }}" not in gate:
                failures.append(f"{name}: required context must run with always()")
            if "needs:" not in gate or "changes" not in gate:
                failures.append(f"{name}: required context must depend on change detection")

    root_workflow = workflows.get("ci.yml", "")
    for runner in ("ubuntu-24.04", "windows-2025", "macos-15"):
        if runner not in root_workflow:
            failures.append(f"ci.yml: root workspace runner coverage missing {runner}")
    root_required = workflow_job_block(root_workflow, "root-required")
    if "workspace-msrv" not in root_required or "MSRV:" not in root_required:
        failures.append("ci.yml: root-required must fail closed on workspace-msrv")

    tauri_linux = workflow_job_block(
        workflows.get("tauri-rust-ci.yml", ""), "tauri-linux-required"
    )
    if (
        "oven-sh/setup-bun@" not in tauri_linux
        or bun_version is None
        or bun_version_inputs(tauri_linux) != [bun_version]
    ):
        failures.append(
            "tauri-rust-ci.yml: Linux validation must install the canonical "
            "Bun pin for bindings"
        )

    if bun_version is not None:
        for name in sorted(EXPECTED_BUN_WORKFLOWS):
            inputs = bun_version_inputs(workflows.get(name, ""))
            if inputs != [bun_version]:
                failures.append(
                    f"{name}: expected exactly one Bun {bun_version} setup input; "
                    f"found {inputs or 'none'}"
                )
        for name, workflow in sorted(workflows.items()):
            if name in EXPECTED_BUN_WORKFLOWS:
                continue
            inputs = bun_version_inputs(workflow)
            if inputs:
                failures.append(
                    f"{name}: unexpected Bun setup input outside governed workflows: "
                    f"{inputs}"
                )

    failures.extend(node_pin_failures(workflows))

    all_workflows = "\n".join(workflows.values())

    justfile = (REPO_ROOT / "justfile").read_text(encoding="utf-8")
    forbidden = "--test-threads=1"
    if forbidden in justfile or forbidden in all_workflows:
        failures.append("global --test-threads=1 remains in justfile or hosted workflows")
    for recipe in (
        "ci-governance-check:",
        "workflow-governance-check:",
        "dependency-governance-check:",
        "coverage-rust:",
        "coverage-tauri:",
        "frontend-audit:",
        "tauri-ci:",
        "vscode-ci:",
    ):
        if recipe not in justfile:
            failures.append(f"missing local recipe: {recipe[:-1]}")
    if "--overall 70 --gateway 85" not in justfile:
        failures.append("70% overall / 85% security-gateway coverage policy is missing")
    smoke_config = (REPO_ROOT / "ccr-ui" / "vitest.smoke.config.ts").read_text(
        encoding="utf-8"
    )
    if "thresholds" not in smoke_config or "lines: 70" not in smoke_config:
        failures.append("React 70% line-coverage policy is missing")
    if "--test-coverage-lines=70" not in (REPO_ROOT / "ccr-vscode" / "justfile").read_text(
        encoding="utf-8"
    ):
        failures.append("VS Code 70% line-coverage policy is missing")

    serial_tests = 0
    for source_root in (REPO_ROOT / "crates", REPO_ROOT / "ccr-ui" / "src-tauri" / "src"):
        for source in source_root.rglob("*.rs"):
            serial_tests += len(SERIAL_TEST_RE.findall(source.read_text(encoding="utf-8")))
    if serial_tests > MAX_SERIAL_TESTS:
        failures.append(
            f"serial-only test annotations {serial_tests} exceed target {MAX_SERIAL_TESTS}"
        )
    if not (REPO_ROOT / ".github" / "dependabot.yml").is_file():
        failures.append(".github/dependabot.yml is missing")

    if failures:
        print("CI governance check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print(f"CI governance check passed ({action_count} immutable action references)")
    print(f"Serial-only test annotations: {serial_tests} (target: {MAX_SERIAL_TESTS})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
