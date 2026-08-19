from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.common import REPO_ROOT
from scripts.quality.check_json_format import canonical_json, process_json_configs


class JsonFormatTests(unittest.TestCase):
    def test_shared_repo_root_is_importable(self) -> None:
        self.assertTrue((REPO_ROOT / "scripts" / "common.py").is_file())

    def test_noncanonical_json_fails_without_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "config.json"
            path.write_text('{"value":1}', encoding="utf-8")
            self.assertEqual(
                process_json_configs(root, ["config.json"]),
                ["noncanonical JSON formatting: config.json"],
            )
            self.assertEqual(path.read_text(encoding="utf-8"), '{"value":1}')

    def test_write_mode_is_semantic_and_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "config.json"
            original = {"version": "7.0.0", "nested": {"enabled": True}}
            path.write_text(json.dumps(original), encoding="utf-8")
            self.assertEqual(process_json_configs(root, ["config.json"], write=True), [])
            self.assertEqual(path.read_text(encoding="utf-8"), canonical_json(original))
            self.assertEqual(process_json_configs(root, ["config.json"]), [])

    def test_malformed_and_missing_files_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "bad.json").write_text("{", encoding="utf-8")
            failures = process_json_configs(root, ["bad.json", "missing.json"])
            self.assertEqual(len(failures), 2)
            self.assertTrue(any("bad.json" in failure for failure in failures))
            self.assertTrue(any("missing.json" in failure for failure in failures))

    def test_unlisted_json_is_not_rewritten(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            excluded = root / "fixture.json"
            excluded.write_text('{"whitespace":"semantic"}', encoding="utf-8")
            self.assertEqual(process_json_configs(root, [], write=True), [])
            self.assertEqual(
                excluded.read_text(encoding="utf-8"), '{"whitespace":"semantic"}'
            )


if __name__ == "__main__":
    unittest.main()
