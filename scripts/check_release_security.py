#!/usr/bin/env python3
"""Validate fail-closed release policy and support release-only helpers."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RELEASE_WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"

REQUIRED_ENV = {
    "macos": (
        "APPLE_CERTIFICATE",
        "APPLE_CERTIFICATE_PASSWORD",
        "APPLE_SIGNING_IDENTITY",
        "APPLE_ID",
        "APPLE_PASSWORD",
        "APPLE_TEAM_ID",
    ),
    "windows": (
        "WINDOWS_CERTIFICATE_BASE64",
        "WINDOWS_CERTIFICATE_PASSWORD",
        "WINDOWS_CERTIFICATE_THUMBPRINT",
        "WINDOWS_TIMESTAMP_URL",
    ),
    "vsix": ("VSCE_PAT", "VSIX_SIGN_TOOL_PATH"),
}

REQUIRED_WORKFLOW_TOKENS = (
    "environment: release",
    "signtool sign",
    "signtool verify /pa /all",
    "codesign --verify --deep --strict",
    "xcrun stapler validate",
    "vsce package --no-dependencies --sign-tool",
    "vsce verify-signature",
    "vsce publish",
    "actions/attest-build-provenance@",
    "actions/attest-sbom@",
    "anchore/sbom-action@",
    "attestations: write",
    "id-token: write",
    "merge-multiple: true",
)


def missing_environment(platform: str, environ: dict[str, str]) -> list[str]:
    return [name for name in REQUIRED_ENV[platform] if not environ.get(name, "").strip()]


def preflight(platform: str) -> int:
    missing = missing_environment(platform, dict(os.environ))
    if missing:
        print(
            f"{platform} release identity is incomplete: {', '.join(missing)}",
            file=sys.stderr,
        )
        return 1
    if platform == "vsix":
        tool = Path(os.environ["VSIX_SIGN_TOOL_PATH"])
        if not tool.is_file() or not os.access(tool, os.X_OK):
            print("VSIX_SIGN_TOOL_PATH must name an executable file", file=sys.stderr)
            return 1
    print(f"{platform} release identity preflight passed")
    return 0


def tauri_override(platform: str, environ: dict[str, str]) -> dict[str, object]:
    if platform == "linux":
        return {}
    missing = missing_environment(platform, environ)
    if missing:
        raise ValueError(f"missing release identity fields: {', '.join(missing)}")
    if platform == "macos":
        return {
            "bundle": {
                "macOS": {"signingIdentity": environ["APPLE_SIGNING_IDENTITY"]}
            }
        }
    thumbprint = re.sub(r"\s+", "", environ["WINDOWS_CERTIFICATE_THUMBPRINT"])
    return {
        "bundle": {
            "windows": {
                "certificateThumbprint": thumbprint,
                "digestAlgorithm": "sha256",
                "timestampUrl": environ["WINDOWS_TIMESTAMP_URL"],
            }
        }
    }


def write_tauri_override(platform: str, output: Path) -> int:
    try:
        config = tauri_override(platform, dict(os.environ))
    except ValueError as error:
        print(error, file=sys.stderr)
        return 1
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(config, separators=(",", ":")) + "\n", encoding="utf-8")
    print(f"wrote release-only Tauri override for {platform}")
    return 0


def updater_policy_failures(root: Path = ROOT) -> list[str]:
    failures: list[str] = []
    for relative in (
        "Cargo.toml",
        "ccr-ui/package.json",
        "ccr-ui/src-tauri/Cargo.toml",
        "ccr-ui/src-tauri/tauri.conf.json",
    ):
        path = root / relative
        if not path.is_file():
            failures.append(f"missing updater policy input: {relative}")
            continue
        text = path.read_text(encoding="utf-8")
        if "tauri-plugin-updater" in text or "@tauri-apps/plugin-updater" in text:
            failures.append(f"automatic updater dependency is enabled: {relative}")
        if relative.endswith("tauri.conf.json") and re.search(
            r'["\']updater["\']\s*:', text, re.IGNORECASE
        ):
            failures.append(f"automatic updater configuration is enabled: {relative}")
    return failures


def workflow_policy_failures(text: str) -> list[str]:
    failures = [
        f"release workflow missing required contract: {token}"
        for token in REQUIRED_WORKFLOW_TOKENS
        if token not in text
    ]
    if "continue-on-error" in text:
        failures.append("release workflow must not suppress signing or publication failures")
    if text.count("softprops/action-gh-release@") != 1:
        failures.append("exactly one centralized GitHub Release publication step is required")
    publish_index = text.find("  publish-release:")
    release_action_index = text.find("softprops/action-gh-release@")
    if publish_index < 0 or release_action_index < publish_index:
        failures.append("GitHub Release publication must be owned by publish-release")
    workflow_header = text.partition("\njobs:")[0]
    if not re.search(r"permissions:\s*\n\s+contents:\s+read", workflow_header):
        failures.append("workflow default permissions must be contents: read")
    for job in (
        "build-cli",
        "build-vscode",
        "build-tauri",
        "verify-and-attest",
        "publish-vscode",
        "publish-release",
    ):
        if f"  {job}:" not in text:
            failures.append(f"release workflow missing job: {job}")
    if "needs: [verify-and-attest, publish-vscode]" not in text:
        failures.append("GitHub Release must wait for attestation and Marketplace publication")
    if "tagName:" in text:
        failures.append("Tauri build action must not publish before centralized verification")
    return failures


def check_policy() -> int:
    failures = updater_policy_failures()
    if not RELEASE_WORKFLOW.is_file():
        failures.append("release workflow is missing")
    else:
        failures.extend(
            workflow_policy_failures(RELEASE_WORKFLOW.read_text(encoding="utf-8"))
        )
    if failures:
        print("Release security check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("Release security check passed (updater disabled; publication is fail closed)")
    return 0


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_checksums(root: Path, output: Path) -> int:
    root = root.resolve()
    output = output.resolve()
    files = sorted(
        path
        for path in root.rglob("*")
        if path.is_file() and path.resolve() != output and not path.is_symlink()
    )
    if not files:
        print("release asset directory is empty", file=sys.stderr)
        return 1
    lines = [f"{sha256(path)}  {path.relative_to(root).as_posix()}" for path in files]
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(files)} release checksums")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("check")

    preflight_parser = subparsers.add_parser("preflight")
    preflight_parser.add_argument("platform", choices=sorted(REQUIRED_ENV))

    tauri_parser = subparsers.add_parser("write-tauri-config")
    tauri_parser.add_argument("platform", choices=("linux", "macos", "windows"))
    tauri_parser.add_argument("output", type=Path)

    checksum_parser = subparsers.add_parser("checksums")
    checksum_parser.add_argument("root", type=Path)
    checksum_parser.add_argument("output", type=Path)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if args.command == "check":
        return check_policy()
    if args.command == "preflight":
        return preflight(args.platform)
    if args.command == "write-tauri-config":
        return write_tauri_override(args.platform, args.output)
    return write_checksums(args.root, args.output)


if __name__ == "__main__":
    raise SystemExit(main())
