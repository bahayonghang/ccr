from __future__ import annotations

import unittest

from scripts.check_workflow_governance import (
    duplicate_mapping_keys,
    workflow_event_values,
)


class WorkflowGovernanceParserTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
