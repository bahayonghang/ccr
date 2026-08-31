from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.common import REPO_ROOT
from scripts.drift.check_dependency_drift import (
    declares_dependency,
    internal_umbrella_dependents,
    validate_msrv,
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


class RustVersionGovernanceTests(unittest.TestCase):
    def write_fixture(
        self, root: Path, *, toolchain: str, crate_msrv: str, tauri_msrv: str
    ) -> None:
        (root / "crates" / "sample").mkdir(parents=True)
        (root / "crates" / "sample" / "Cargo.toml").write_text(
            f'[package]\nname = "sample"\nversion = "1.0.0"\nrust-version = "{crate_msrv}"\n',
            encoding="utf-8",
        )
        (root / "ccr-ui" / "src-tauri").mkdir(parents=True)
        (root / "ccr-ui" / "src-tauri" / "Cargo.toml").write_text(
            f'[package]\nname = "desktop"\nversion = "1.0.0"\nrust-version = "{tauri_msrv}"\n',
            encoding="utf-8",
        )
        (root / "rust-toolchain.toml").write_text(
            f'[toolchain]\nchannel = "{toolchain}"\n', encoding="utf-8"
        )

    def test_development_toolchain_and_msrv_are_independent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_fixture(
                root, toolchain="1.98.0", crate_msrv="1.95", tauri_msrv="1.95"
            )

            self.assertEqual(validate_msrv(root), [])

    def test_stale_development_pin_and_msrv_drift_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_fixture(
                root, toolchain="1.95.0", crate_msrv="1.96", tauri_msrv="1.95"
            )

            self.assertEqual(
                validate_msrv(root),
                [
                    "crates/sample/Cargo.toml rust-version='1.96', expected '1.95'",
                    "rust-toolchain.toml channel='1.95.0', expected '1.98.0'",
                ],
            )


if __name__ == "__main__":
    unittest.main()
