from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.common import REPO_ROOT
from scripts.drift.check_doc_drift import check_doc_drift


class DocDriftTests(unittest.TestCase):
    def test_shared_repo_root_is_importable(self) -> None:
        self.assertTrue((REPO_ROOT / "scripts" / "common.py").is_file())

    def _write_happy_tree(self, root: Path) -> None:
        ui = root / "ccr-ui"
        tauri = ui / "src-tauri"
        tauri.mkdir(parents=True)
        (ui / "package.json").write_text(
            json.dumps(
                {"version": "1.2.3", "packageManager": "bun@1.3.10"},
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        (ui / "bun.lock").write_text("{}\n", encoding="utf-8")
        (tauri / "Cargo.toml").write_text(
            '[package]\nname = "ccr-ui"\nrust-version = "1.95"\nedition = "2024"\n',
            encoding="utf-8",
        )
        (ui / "README.md").write_text(
            "\n".join(
                [
                    "version-1.2.3",
                    "Bun is the only maintained frontend package manager",
                    "bun.lock is the dependency source of truth",
                    "Bun | `bun@1.3.10`",
                    "Rust | `>= 1.95`",
                    "Rust edition | Edition 2024",
                    "Tauri invoke APIs",
                    "Web runtime",
                    "bun run lint:fix",
                    "",
                ]
            ),
            encoding="utf-8",
        )

    def test_missing_file_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            (root / "ccr-ui" / "bun.lock").unlink()
            failures = check_doc_drift(root)
            self.assertTrue(any("bun.lock" in failure for failure in failures))

    def test_package_lock_present_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            (root / "ccr-ui" / "package-lock.json").write_text("{}\n", encoding="utf-8")
            failures = check_doc_drift(root)
            self.assertTrue(any("package-lock.json" in failure for failure in failures))

    def test_missing_readme_fact_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            readme = root / "ccr-ui" / "README.md"
            readme.write_text(
                readme.read_text(encoding="utf-8").replace("Tauri invoke APIs", ""),
                encoding="utf-8",
            )
            failures = check_doc_drift(root)
            self.assertTrue(any("Tauri invoke APIs" in failure for failure in failures))

    def test_stale_readme_text_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            readme = root / "ccr-ui" / "README.md"
            readme.write_text(
                readme.read_text(encoding="utf-8") + "Axios\n",
                encoding="utf-8",
            )
            failures = check_doc_drift(root)
            self.assertTrue(any("Axios" in failure for failure in failures))

    def test_happy_path_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            self.assertEqual(check_doc_drift(root), [])


if __name__ == "__main__":
    unittest.main()
