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
        docs = root / "docs"
        tauri.mkdir(parents=True)
        docs.mkdir(parents=True)
        (ui / "package.json").write_text(
            json.dumps(
                {"version": "1.2.3", "packageManager": "bun@1.4.0"},
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
                    "Bun | `bun@1.4.0`",
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
        (docs / "package.json").write_text(
            json.dumps(
                {"name": "ccr-docs", "packageManager": "bun@1.4.0"}, indent=2
            )
            + "\n",
            encoding="utf-8",
        )
        (docs / "bun.lock").write_text("{}\n", encoding="utf-8")
        (docs / "README.md").write_text(
            "\n".join(
                [
                    "`docs/bun.lock` is the only maintained docs dependency lockfile",
                    "`docs/package.json#packageManager` must mirror the canonical "
                    "`ccr-ui/package.json#packageManager` Bun pin",
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

    def test_docs_bun_lock_missing_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            (root / "docs" / "bun.lock").unlink()
            failures = check_doc_drift(root)
            self.assertTrue(any("docs/bun.lock" in failure for failure in failures))

    def test_ui_package_lock_present_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            (root / "ccr-ui" / "package-lock.json").write_text("{}\n", encoding="utf-8")
            failures = check_doc_drift(root)
            self.assertTrue(
                any("ccr-ui/package-lock.json" in failure for failure in failures)
            )

    def test_docs_package_lock_present_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            (root / "docs" / "package-lock.json").write_text(
                "{}\n", encoding="utf-8"
            )
            failures = check_doc_drift(root)
            self.assertTrue(
                any("docs/package-lock.json" in failure for failure in failures)
            )

    def test_docs_lock_authority_missing_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            (root / "docs" / "README.md").write_text(
                "Documentation site\n", encoding="utf-8"
            )
            failures = check_doc_drift(root)
            self.assertTrue(any("锁文件权威声明" in failure for failure in failures))

    def test_docs_package_manager_must_match_canonical_ui_pin(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            docs_package = root / "docs" / "package.json"
            docs_package.write_text(
                json.dumps(
                    {"name": "ccr-docs", "packageManager": "bun@1.3.10"},
                    indent=2,
                )
                + "\n",
                encoding="utf-8",
            )
            failures = check_doc_drift(root)
            self.assertTrue(any("必须与" in failure for failure in failures))

    def test_package_manager_requires_an_exact_semver_pin(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_happy_tree(root)
            ui_package = root / "ccr-ui" / "package.json"
            ui_package.write_text(
                json.dumps(
                    {"version": "1.2.3", "packageManager": "bun@1.4"},
                    indent=2,
                )
                + "\n",
                encoding="utf-8",
            )
            failures = check_doc_drift(root)
            self.assertTrue(any("bun@x.y.z" in failure for failure in failures))

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
