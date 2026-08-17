from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.common import REPO_ROOT
from scripts.drift.check_dependency_drift import (
    declares_dependency,
    internal_umbrella_dependents,
)


class InternalUmbrellaDependencyTests(unittest.TestCase):
    def test_shared_repo_root_is_importable(self) -> None:
        self.assertTrue((REPO_ROOT / "scripts" / "common.py").is_file())

    def test_detects_direct_and_target_specific_dependencies(self) -> None:
        self.assertTrue(declares_dependency({"dependencies": {"ccr": "7"}}, "ccr"))
        self.assertTrue(
            declares_dependency(
                {"target": {"cfg(windows)": {"dev-dependencies": {"ccr": "7"}}}},
                "ccr",
            )
        )

    def test_ignores_unrelated_dependency_names(self) -> None:
        self.assertFalse(
            declares_dependency({"dependencies": {"ccr-core": "7"}}, "ccr")
        )

    def test_root_facade_is_not_reported_as_its_own_dependent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            facade = root / "crates/ccr/Cargo.toml"
            facade.parent.mkdir(parents=True)
            facade.write_text(
                '[package]\nname = "ccr"\nversion = "7.0.0"\n', encoding="utf-8"
            )
            consumer = root / "crates/consumer/Cargo.toml"
            consumer.parent.mkdir(parents=True)
            consumer.write_text(
                '[package]\nname = "consumer"\nversion = "1.0.0"\n'
                '[dependencies]\nccr = { path = "../ccr" }\n',
                encoding="utf-8",
            )
            self.assertEqual(
                internal_umbrella_dependents(root), ["crates/consumer/Cargo.toml"]
            )


if __name__ == "__main__":
    unittest.main()
