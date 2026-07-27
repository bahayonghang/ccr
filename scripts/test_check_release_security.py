from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.check_release_security import (
    RELEASE_WORKFLOW,
    missing_environment,
    tauri_override,
    updater_policy_failures,
    workflow_policy_failures,
    write_checksums,
)


class ReleaseSecurityTests(unittest.TestCase):
    def test_repository_release_workflow_satisfies_policy(self) -> None:
        self.assertEqual(
            workflow_policy_failures(RELEASE_WORKFLOW.read_text(encoding="utf-8")), []
        )

    def test_missing_verifier_and_early_release_are_rejected(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
        workflow = workflow.replace("signtool verify /pa /all", "Write-Host skipped")
        workflow = workflow.replace(
            "      - name: Publish GitHub Release",
            "      - uses: softprops/action-gh-release@0123456789012345678901234567890123456789\n"
            "      - name: Publish GitHub Release",
        )
        failures = workflow_policy_failures(workflow)
        self.assertTrue(any("signtool verify" in failure for failure in failures))
        self.assertTrue(any("exactly one" in failure for failure in failures))

    def test_preflight_reports_names_without_secret_values(self) -> None:
        environ = {"WINDOWS_CERTIFICATE_PASSWORD": "do-not-print"}
        missing = missing_environment("windows", environ)
        self.assertIn("WINDOWS_CERTIFICATE_BASE64", missing)
        self.assertNotIn("do-not-print", " ".join(missing))

    def test_tauri_overrides_contain_policy_not_credentials(self) -> None:
        windows = tauri_override(
            "windows",
            {
                "WINDOWS_CERTIFICATE_BASE64": "secret-pfx-payload",
                "WINDOWS_CERTIFICATE_PASSWORD": "secret-pfx-password",
                "WINDOWS_CERTIFICATE_THUMBPRINT": "AA BB",
                "WINDOWS_TIMESTAMP_URL": "https://timestamp.example.test",
            },
        )
        encoded = json.dumps(windows)
        self.assertIn("AABB", encoded)
        self.assertNotIn("secret-pfx-payload", encoded)
        self.assertNotIn("secret-pfx-password", encoded)

    def test_checksum_manifest_is_sorted_and_excludes_itself(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "z.zip").write_bytes(b"z")
            (root / "a.tar.gz").write_bytes(b"a")
            output = root / "SHA256SUMS"
            self.assertEqual(write_checksums(root, output), 0)
            names = [line.split("  ", 1)[1] for line in output.read_text().splitlines()]
            self.assertEqual(names, ["a.tar.gz", "z.zip"])

    def test_updater_dependency_breaks_freeze(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in (
                "Cargo.toml",
                "ccr-ui/package.json",
                "ccr-ui/src-tauri/Cargo.toml",
                "ccr-ui/src-tauri/tauri.conf.json",
            ):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("{}", encoding="utf-8")
            (root / "ccr-ui/src-tauri/Cargo.toml").write_text(
                'tauri-plugin-updater = "2"', encoding="utf-8"
            )
            self.assertTrue(updater_policy_failures(root))


if __name__ == "__main__":
    unittest.main()
