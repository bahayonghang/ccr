from __future__ import annotations

import unittest
from pathlib import Path

from scripts.ci_surface_policy import SURFACE_PATHS, is_relevant, path_matches
from scripts.check_workflow_governance import (
    duplicate_mapping_keys,
    workflow_event_values,
    workflow_job_block,
)


class WorkflowGovernanceParserTests(unittest.TestCase):
    ROOT = Path(__file__).resolve().parents[1]

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
                    is_relevant(surface, ["scripts/ci_surface_policy.py"])
                )

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

    def test_tauri_linux_gate_installs_pinned_bun_for_bindings(self) -> None:
        workflow = (
            self.ROOT / ".github" / "workflows" / "tauri-rust-ci.yml"
        ).read_text(encoding="utf-8")
        linux_job = workflow_job_block(workflow, "tauri-linux-required")

        self.assertIn(
            "oven-sh/setup-bun@0c5077e51419868618aeaa5fe8019c62421857d6",
            linux_job,
        )
        self.assertIn("bun-version: 1.3.10", linux_job)
        self.assertIn("run: just tauri-ci", linux_job)


if __name__ == "__main__":
    unittest.main()
