from __future__ import annotations

import io
import unittest
import zipfile

from scripts.ci.install_prebuilt_tool import asset_for, extract_binary, verify_sha256


class InstallPrebuiltToolTests(unittest.TestCase):
    def test_pinned_just_matches_the_linux_asset_used_by_coverage(self) -> None:
        asset = asset_for("just", "linux", "x86_64")
        self.assertEqual(
            asset.sha256,
            "4a5cc2f53e6f0f8c59092a6cc38291eb729d46a7dd95d3ae582008881b84931d",
        )
        self.assertTrue(asset.url.endswith("just-1.58.0-x86_64-unknown-linux-musl.tar.gz"))

    def test_hosted_runners_have_assets(self) -> None:
        for tool, system, arch in (
            ("just", "linux", "x86_64"),
            ("just", "windows", "x86_64"),
            ("just", "darwin", "aarch64"),
            ("cargo-audit", "linux", "x86_64"),
            ("cargo-llvm-cov", "linux", "x86_64"),
        ):
            with self.subTest(tool=tool, system=system, arch=arch):
                asset = asset_for(tool, system, arch)
                self.assertEqual(len(asset.sha256), 64)
                self.assertEqual(asset.binary, tool)

    def test_unknown_platform_fails_closed(self) -> None:
        with self.assertRaises(SystemExit):
            asset_for("cargo-llvm-cov", "windows", "x86_64")

    def test_checksum_mismatch_fails_closed(self) -> None:
        with self.assertRaises(SystemExit):
            verify_sha256(b"not-the-archive", "0" * 64)

    def test_nested_zip_extracts_only_the_binary(self) -> None:
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("cargo-audit-win/README.md", "docs")
            archive.writestr("cargo-audit-win/cargo-audit.exe", b"binary")
        name, data = extract_binary(buffer.getvalue(), "https://example.test/tool.zip", "cargo-audit")
        self.assertEqual(name, "cargo-audit.exe")
        self.assertEqual(data, b"binary")


if __name__ == "__main__":
    unittest.main()
