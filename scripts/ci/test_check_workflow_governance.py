from __future__ import annotations

import json
import shutil
import subprocess
import unittest

from scripts.common import REPO_ROOT
from scripts.ci.ci_surface_policy import SURFACE_PATHS, is_relevant, path_matches
from scripts.ci.check_workflow_governance import (
    DEVELOPMENT_RUST_TOOLCHAIN,
    EXPECTED_BUN_WORKFLOWS,
    EXPECTED_NODE_WORKFLOW_INPUTS,
    MSRV_RUST_TOOLCHAIN,
    NODE_TOOLCHAIN,
    bun_version_inputs,
    canonical_bun_version,
    duplicate_mapping_keys,
    node_pin_failures,
    node_version_inputs,
    rust_toolchain_inputs,
    setup_node_version_inputs,
    workflow_event_values,
    workflow_job_block,
)


def _workflow_step_fields(text: str, needle: str) -> dict[str, str]:
    """Return scalar keys from the YAML sequence item that contains *needle*."""
    lines = text.splitlines()
    match_index = next(
        (index for index, line in enumerate(lines) if needle in line),
        None,
    )
    if match_index is None:
        return {}

    start: int | None = None
    start_indent = 0
    for index in range(match_index, -1, -1):
        line = lines[index]
        if not line.strip():
            continue
        indent = len(line) - len(line.lstrip(" "))
        if line.lstrip().startswith("- "):
            start = index
            start_indent = indent
            break
    if start is None:
        return {}

    end = len(lines)
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = len(line) - len(line.lstrip(" "))
        if indent < start_indent:
            end = index
            break
        if indent == start_indent and line.lstrip().startswith("- "):
            end = index
            break

    fields: dict[str, str] = {}
    for line in lines[start:end]:
        stripped = line.strip()
        if stripped.startswith("- "):
            stripped = stripped[2:].strip()
        if ":" not in stripped:
            continue
        key, value = stripped.split(":", 1)
        fields[key.strip()] = value.strip()
    return fields


class WorkflowGovernanceParserTests(unittest.TestCase):
    ROOT = REPO_ROOT

    def test_duplicate_mapping_key_is_rejected(self) -> None:
        workflow = """jobs:
  build:
    steps:
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@0123456789012345678901234567890123456789
        with:
          toolchain: 1.95.0
        with:
          components: clippy
"""

        self.assertEqual(duplicate_mapping_keys(workflow), [(8, "with")])

    def test_sequence_items_and_block_scalars_do_not_look_duplicate(self) -> None:
        workflow = """jobs:
  build:
    steps:
      - name: First
        run: |
          echo "name: inside script"
      - name: Second
        run: echo done
"""

        self.assertEqual(duplicate_mapping_keys(workflow), [])

    def test_trigger_values_are_read_from_the_requested_event(self) -> None:
        workflow = """on:
  push:
    branches: [main, develop, dev]
    paths:
      - 'ccr-ui/**'
  pull_request:
    branches: [main, develop, dev]
    paths:
      - 'ccr-ui/**'
      - 'justfile'
"""

        self.assertEqual(
            workflow_event_values(workflow, "pull_request", "branches"),
            {"main", "develop", "dev"},
        )
        self.assertEqual(
            workflow_event_values(workflow, "pull_request", "paths"),
            {"ccr-ui/**", "justfile"},
        )
        self.assertEqual(
            workflow_event_values(workflow, "push", "paths"), {"ccr-ui/**"}
        )

    def test_missing_event_is_reported_as_empty(self) -> None:
        workflow = """on:
  pull_request:
    branches: [main, develop, dev]
"""

        self.assertEqual(workflow_event_values(workflow, "push", "branches"), set())

    def test_surface_paths_match_nested_and_exact_files(self) -> None:
        self.assertTrue(path_matches("crates/ccr/src/main.rs", SURFACE_PATHS["root"]))
        self.assertTrue(path_matches("justfile", SURFACE_PATHS["vscode"]))
        self.assertTrue(
            path_matches(
                "ccr-ui/src-tauri/src/main.rs", SURFACE_PATHS["tauri"]
            )
        )
        self.assertFalse(path_matches("README.md", SURFACE_PATHS["frontend"]))

    def test_relevance_is_true_when_any_changed_path_matches(self) -> None:
        self.assertTrue(
            is_relevant("vscode", ["README.md", "ccr-vscode/src/extension.ts"])
        )
        self.assertFalse(is_relevant("vscode", ["README.md", "docs/index.md"]))

    def test_policy_script_changes_validate_every_surface(self) -> None:
        for surface in SURFACE_PATHS:
            with self.subTest(surface=surface):
                self.assertTrue(
                    is_relevant(surface, ["scripts/ci/ci_surface_policy.py"])
                )

    def test_cargo_config_inputs_trigger_only_consumer_surfaces(self) -> None:
        expected = {
            ".cargo/tauri-ci.toml": {"tauri"},
            ".cargo/config.toml": {"root", "tauri"},
            ".cargo/audit.toml": {"root"},
        }
        for path, surfaces in expected.items():
            for surface in SURFACE_PATHS:
                with self.subTest(path=path, surface=surface):
                    self.assertEqual(
                        is_relevant(surface, [path]),
                        surface in surfaces,
                    )
        for patterns in SURFACE_PATHS.values():
            self.assertNotIn(".cargo/**", patterns)

    def test_vscode_coverage_step_uses_bash_pipefail(self) -> None:
        workflow = (
            self.ROOT / ".github" / "workflows" / "vscode-ci.yml"
        ).read_text(encoding="utf-8")
        step = _workflow_step_fields(workflow, "just vscode-coverage | tee")
        self.assertEqual(
            step.get("run"),
            "just vscode-coverage | tee vscode-coverage.txt",
        )
        self.assertEqual(step.get("shell"), "bash")

        bash = shutil.which("bash")
        if bash is None:
            self.skipTest(
                "bash is not available to probe GitHub pipefail semantics"
            )

        failed = subprocess.run(
            [
                bash,
                "--noprofile",
                "--norc",
                "-eo",
                "pipefail",
                "-c",
                "false | tee /dev/null",
            ],
            capture_output=True,
            check=False,
        )
        self.assertNotEqual(failed.returncode, 0)

        succeeded = subprocess.run(
            [
                bash,
                "--noprofile",
                "--norc",
                "-eo",
                "pipefail",
                "-c",
                "true | tee /dev/null",
            ],
            capture_output=True,
            check=False,
        )
        self.assertEqual(succeeded.returncode, 0)

    def test_job_block_stops_before_the_next_job(self) -> None:
        workflow = """jobs:
  validation:
    name: Validation
  required:
    name: Required
    if: ${{ always() }}
"""

        self.assertEqual(
            workflow_job_block(workflow, "required"),
            "  required:\n    name: Required\n    if: ${{ always() }}",
        )

    def test_rust_toolchain_inputs_are_extracted_from_action_inputs(self) -> None:
        workflow = """steps:
  - uses: dtolnay/rust-toolchain@0123456789012345678901234567890123456789
    with:
      toolchain: 1.98.0
"""

        self.assertEqual(rust_toolchain_inputs(workflow), ["1.98.0"])

    def test_bun_version_inputs_are_extracted_from_action_inputs(self) -> None:
        workflow = """steps:
  - uses: oven-sh/setup-bun@0123456789012345678901234567890123456789
    with:
      bun-version: 1.4.0
"""

        self.assertEqual(bun_version_inputs(workflow), ["1.4.0"])

    def test_node_version_inputs_are_extracted_from_action_inputs(self) -> None:
        workflow = """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
"""

        self.assertEqual(node_version_inputs(workflow), ["24.20.0"])
        self.assertEqual(setup_node_version_inputs(workflow), [["24.20.0"]])

    def test_node_toolchain_validator_fails_closed_on_drift_and_extra_inputs(
        self,
    ) -> None:
        workflows = {
            "release.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.19.0
""",
            "vscode-ci.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
""",
            "ci.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 26.8.1
""",
        }

        failures = node_pin_failures(workflows)

        self.assertTrue(
            any("release.yml: expected 2 Node 24.20.0" in item for item in failures)
        )
        self.assertTrue(
            any("ci.yml: unexpected Node setup input" in item for item in failures)
        )

    def test_unrelated_node_version_cannot_mask_an_unpinned_setup_step(self) -> None:
        workflows = {
            "release.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
  - uses: actions/setup-node@0123456789012345678901234567890123456789
  - uses: example/action@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
""",
            "vscode-ci.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
""",
        }

        failures = node_pin_failures(workflows)

        self.assertTrue(any("release.yml: expected 2" in item for item in failures))

    def test_bun_toolchain_pin_has_one_canonical_source(self) -> None:
        self.assertEqual(canonical_bun_version(self.ROOT), "1.4.0")
        docs_package = json.loads(
            (self.ROOT / "docs" / "package.json").read_text(encoding="utf-8")
        )
        self.assertEqual(docs_package.get("packageManager"), "bun@1.4.0")

        for name in EXPECTED_BUN_WORKFLOWS:
            workflow = (self.ROOT / ".github" / "workflows" / name).read_text(
                encoding="utf-8"
            )
            with self.subTest(workflow=name):
                self.assertEqual(bun_version_inputs(workflow), ["1.4.0"])

    def test_node_toolchain_pin_is_exact_and_scoped(self) -> None:
        workflow_dir = self.ROOT / ".github" / "workflows"
        workflows = {
            path.name: path.read_text(encoding="utf-8")
            for path in workflow_dir.iterdir()
            if path.suffix in {".yml", ".yaml"}
        }

        for name, expected_count in EXPECTED_NODE_WORKFLOW_INPUTS.items():
            with self.subTest(workflow=name):
                self.assertEqual(
                    node_version_inputs(workflows[name]),
                    [NODE_TOOLCHAIN] * expected_count,
                )
        for name, workflow in workflows.items():
            if name in EXPECTED_NODE_WORKFLOW_INPUTS:
                continue
            with self.subTest(workflow=name):
                self.assertEqual(node_version_inputs(workflow), [])

    def test_workflows_split_development_toolchain_from_msrv(self) -> None:
        workflows = {
            name: (self.ROOT / ".github" / "workflows" / name).read_text(
                encoding="utf-8"
            )
            for name in (
                "ci.yml",
                "frontend-ci.yml",
                "release.yml",
                "tauri-rust-ci.yml",
                "vscode-ci.yml",
            )
        }
        msrv_job = workflow_job_block(workflows["ci.yml"], "workspace-msrv")

        self.assertEqual(rust_toolchain_inputs(msrv_job), [MSRV_RUST_TOOLCHAIN])
        self.assertIn(
            "cargo check --workspace --all-targets --all-features", msrv_job
        )
        for name, workflow in workflows.items():
            ordinary_workflow = (
                workflow.replace(msrv_job, "", 1) if name == "ci.yml" else workflow
            )
            with self.subTest(workflow=name):
                self.assertTrue(rust_toolchain_inputs(ordinary_workflow))
                self.assertEqual(
                    set(rust_toolchain_inputs(ordinary_workflow)),
                    {DEVELOPMENT_RUST_TOOLCHAIN},
                )

        root_required = workflow_job_block(workflows["ci.yml"], "root-required")
        self.assertIn("workspace-msrv", root_required)
        self.assertIn("MSRV:", root_required)

    def test_tauri_rust_gates_use_a_tracked_frontend_fixture(self) -> None:
        cargo_config = (self.ROOT / ".cargo" / "tauri-ci.toml").read_text(
            encoding="utf-8"
        )
        root_justfile = (self.ROOT / "justfile").read_text(encoding="utf-8")
        ui_justfile = (self.ROOT / "ccr-ui" / "justfile").read_text(
            encoding="utf-8"
        )

        self.assertIn('frontendDist":"ci-dist"', cargo_config)
        self.assertTrue(
            (self.ROOT / "ccr-ui" / "src-tauri" / "ci-dist" / "index.html").is_file()
        )
        self.assertIn("cargo --config .cargo/tauri-ci.toml test", root_justfile)
        self.assertIn(
            "cargo --config .cargo/tauri-ci.toml llvm-cov", root_justfile
        )
        self.assertIn(
            "cargo --config ../.cargo/tauri-ci.toml test", ui_justfile
        )

    def test_root_fmt_repairs_json_before_fmt_check(self) -> None:
        root_justfile = (self.ROOT / "justfile").read_text(encoding="utf-8")

        self.assertIn("fmt: json-format", root_justfile)
        self.assertIn("fmt-check: json-format-check", root_justfile)

    def test_tauri_bindings_check_compares_against_the_worktree_baseline(self) -> None:
        ui_justfile = (self.ROOT / "ccr-ui" / "justfile").read_text(
            encoding="utf-8"
        )

        self.assertIn("scripts/check-generated-bindings.mjs", ui_justfile)
        self.assertIn("scripts/normalize-generated-bindings.mjs", ui_justfile)
        self.assertNotIn("git status --porcelain -- src/types/generated", ui_justfile)
        self.assertTrue(
            (self.ROOT / "ccr-ui" / "scripts" / "check-generated-bindings.mjs").is_file()
        )

    def test_quality_workflows_are_pull_request_only(self) -> None:
        for name in ("ci.yml", "frontend-ci.yml", "tauri-rust-ci.yml", "vscode-ci.yml"):
            workflow = (self.ROOT / ".github" / "workflows" / name).read_text(
                encoding="utf-8"
            )
            self.assertEqual(
                workflow_event_values(workflow, "push", "branches"),
                set(),
                msg=f"{name} must not trigger on branch push",
            )
            self.assertEqual(
                workflow_event_values(workflow, "pull_request", "branches"),
                {"main", "develop", "dev"},
            )

    def test_react_smoke_coverage_threshold_is_seventy_percent(self) -> None:
        smoke_config = (self.ROOT / "ccr-ui" / "vitest.smoke.config.ts").read_text(
            encoding="utf-8"
        )

        self.assertIn("thresholds", smoke_config)
        self.assertIn("lines: 70", smoke_config)

    def test_tauri_linux_gate_installs_pinned_bun_for_bindings(self) -> None:
        workflow = (
            self.ROOT / ".github" / "workflows" / "tauri-rust-ci.yml"
        ).read_text(encoding="utf-8")
        linux_job = workflow_job_block(workflow, "tauri-linux-required")

        self.assertIn(
            "oven-sh/setup-bun@0c5077e51419868618aeaa5fe8019c62421857d6",
            linux_job,
        )
        self.assertEqual(bun_version_inputs(linux_job), ["1.4.0"])
        self.assertIn("run: just tauri-ci", linux_job)


if __name__ == "__main__":
    unittest.main()
