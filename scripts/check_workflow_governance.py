#!/usr/bin/env python3
"""Fail closed on mutable GitHub Actions and hosted/local CI drift."""

from __future__ import annotations

import re
import sys
from pathlib import Path

try:
    from scripts.ci_surface_policy import SURFACE_PATHS
except ModuleNotFoundError:
    from ci_surface_policy import SURFACE_PATHS


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_DIR = ROOT / ".github" / "workflows"
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
USES_RE = re.compile(r"^\s*-?\s*uses:\s*([^\s#]+)", re.MULTILINE)
MAPPING_KEY_RE = re.compile(
    r"^(?P<indent> *)(?P<sequence>-\s+)?(?P<key>[A-Za-z_][A-Za-z0-9_-]*):(?P<value>.*)$"
)
SERIAL_TEST_RE = re.compile(r"#\[\s*(?:serial|serial_test::serial)(?:\([^]]*\))?\s*\]")
MAX_SERIAL_TESTS = 0
REQUIRED_BRANCHES = {"main", "develop", "dev"}


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

    frontend_push_branches = workflow_event_values(
        workflows.get("frontend-ci.yml", ""), "push", "branches"
    )
    if frontend_push_branches != REQUIRED_BRANCHES:
        failures.append("frontend-ci.yml: push branches must cover main/develop/dev")

    root_workflow = workflows.get("ci.yml", "")
    for runner in ("ubuntu-24.04", "windows-2025", "macos-15"):
        if runner not in root_workflow:
            failures.append(f"ci.yml: root workspace runner coverage missing {runner}")

    tauri_linux = workflow_job_block(
        workflows.get("tauri-rust-ci.yml", ""), "tauri-linux-required"
    )
    if "oven-sh/setup-bun@" not in tauri_linux or "bun-version: 1.3.10" not in tauri_linux:
        failures.append(
            "tauri-rust-ci.yml: Linux validation must install pinned Bun 1.3.10 for bindings"
        )

    all_workflows = "\n".join(workflows.values())
    if "bun-version: 1.3.10" not in all_workflows:
        failures.append("Bun 1.3.10 pin is missing")
    if "node-version: 24.18.0" not in all_workflows:
        failures.append("Node 24.18.0 pin is missing")

    justfile = (ROOT / "justfile").read_text(encoding="utf-8")
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
        "release-security-check:",
        "tauri-ci:",
        "vscode-ci:",
    ):
        if recipe not in justfile:
            failures.append(f"missing local recipe: {recipe[:-1]}")
    if "--overall 70 --gateway 85" not in justfile:
        failures.append("70% overall / 85% security-gateway coverage policy is missing")
    if "--coverage.thresholds.lines=70" not in justfile:
        failures.append("Vue 70% line-coverage policy is missing")
    if "--test-coverage-lines=70" not in (ROOT / "ccr-vscode" / "justfile").read_text(
        encoding="utf-8"
    ):
        failures.append("VS Code 70% line-coverage policy is missing")

    serial_tests = 0
    for source_root in (ROOT / "crates", ROOT / "ccr-ui" / "src-tauri" / "src"):
        for source in source_root.rglob("*.rs"):
            serial_tests += len(SERIAL_TEST_RE.findall(source.read_text(encoding="utf-8")))
    if serial_tests > MAX_SERIAL_TESTS:
        failures.append(
            f"serial-only test annotations {serial_tests} exceed target {MAX_SERIAL_TESTS}"
        )
    if not (ROOT / ".github" / "dependabot.yml").is_file():
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
