#!/usr/bin/env python3
"""Validate root/Tauri dependency drift, exception metadata, and MSRV pinning."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import sys
import tomllib
from pathlib import Path
from typing import Any

try:
    from scripts.common import REPO_ROOT
except ModuleNotFoundError:
    # pywin32 在 site-packages 注册了同名 namespace package `scripts`
    _scripts_dir = Path(__file__).resolve().parent
    while _scripts_dir.name != "scripts":
        _scripts_dir = _scripts_dir.parent
    sys.path.insert(0, str(_scripts_dir.parent))
    sys.modules.pop("scripts", None)
    from scripts.common import REPO_ROOT


ROOT_CARGO = REPO_ROOT / "Cargo.toml"
TAURI_CARGO = REPO_ROOT / "ccr-ui" / "src-tauri" / "Cargo.toml"
ALLOWLIST = REPO_ROOT / "scripts" / "drift" / "dependency-drift-allowlist.json"
RUST_TOOLCHAIN = REPO_ROOT / "rust-toolchain.toml"
EXPECTED_MSRV = "1.95"
EXPECTED_TOOLCHAIN = "1.95.0"
INTERNAL_UMBRELLA_ALLOWLIST: frozenset[str] = frozenset()


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def dependency_versions(section: dict[str, Any]) -> dict[str, str]:
    versions: dict[str, str] = {}
    for name, value in section.items():
        if isinstance(value, str):
            versions[name] = value
        elif isinstance(value, dict) and isinstance(value.get("version"), str):
            versions[name] = value["version"]
    return versions


def validate_exceptions(
    root_deps: dict[str, str], tauri_deps: dict[str, str]
) -> tuple[dict[str, dict[str, str]], list[str]]:
    payload = json.loads(ALLOWLIST.read_text(encoding="utf-8"))
    max_active = payload.get("max_active_exceptions")
    entries = payload.get("exceptions")
    failures: list[str] = []
    if not isinstance(max_active, int) or max_active < 0:
        failures.append("max_active_exceptions must be a non-negative integer")
        max_active = 0
    if not isinstance(entries, list):
        return {}, failures + ["exceptions must be an array"]
    if len(entries) > max_active:
        failures.append(f"active exceptions {len(entries)} exceed target {max_active}")

    today = dt.datetime.now(dt.timezone.utc).date()
    result: dict[str, dict[str, str]] = {}
    for index, raw in enumerate(entries):
        if not isinstance(raw, dict):
            failures.append(f"exception[{index}] must be an object")
            continue
        values = {key: raw.get(key) for key in ("dependency", "owner", "rationale", "expires")}
        for key, value in values.items():
            if not isinstance(value, str) or not value.strip():
                failures.append(f"exception[{index}].{key} must be a non-empty string")
        name = values["dependency"]
        if not isinstance(name, str) or not name:
            continue
        if name in result:
            failures.append(f"duplicate exception: {name}")
            continue
        try:
            expiry = dt.date.fromisoformat(str(values["expires"]))
            if expiry < today:
                failures.append(f"exception '{name}' expired on {expiry.isoformat()}")
        except ValueError:
            failures.append(f"exception '{name}' has invalid ISO expiry")
        if name not in root_deps or name not in tauri_deps:
            failures.append(f"exception '{name}' no longer maps to a repeated dependency")
        elif root_deps[name] == tauri_deps[name]:
            failures.append(f"exception '{name}' is stale because versions now match")
        result[name] = {key: str(value) for key, value in values.items()}
    return result, failures


def validate_msrv() -> list[str]:
    failures: list[str] = []
    manifests = sorted((REPO_ROOT / "crates").glob("*/Cargo.toml")) + [TAURI_CARGO]
    for manifest in manifests:
        package = load_toml(manifest).get("package", {})
        actual = package.get("rust-version")
        if actual != EXPECTED_MSRV:
            failures.append(
                f"{manifest.relative_to(REPO_ROOT)} rust-version={actual!r}, expected {EXPECTED_MSRV!r}"
            )
    channel = load_toml(RUST_TOOLCHAIN).get("toolchain", {}).get("channel")
    if channel != EXPECTED_TOOLCHAIN:
        failures.append(
            f"rust-toolchain.toml channel={channel!r}, expected {EXPECTED_TOOLCHAIN!r}"
        )
    return failures


def declares_dependency(payload: Any, dependency: str) -> bool:
    if not isinstance(payload, dict):
        return False
    for key, value in payload.items():
        if key in {"dependencies", "dev-dependencies", "build-dependencies"}:
            if isinstance(value, dict) and dependency in value:
                return True
        if declares_dependency(value, dependency):
            return True
    return False


def internal_umbrella_dependents(root: Path = REPO_ROOT) -> list[str]:
    dependents: list[str] = []
    for manifest in sorted((root / "crates").glob("*/Cargo.toml")):
        relative = manifest.relative_to(root).as_posix()
        if relative == "crates/ccr/Cargo.toml" or relative in INTERNAL_UMBRELLA_ALLOWLIST:
            continue
        if declares_dependency(load_toml(manifest), "ccr"):
            dependents.append(relative)
    return dependents


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verbose", "-v", action="store_true")
    args = parser.parse_args()

    root_deps = dependency_versions(load_toml(ROOT_CARGO)["workspace"]["dependencies"])
    tauri_deps = dependency_versions(load_toml(TAURI_CARGO)["dependencies"])
    exceptions, failures = validate_exceptions(root_deps, tauri_deps)
    repeated = sorted(root_deps.keys() & tauri_deps.keys())
    drifts: list[str] = []
    for name in repeated:
        if root_deps[name] == tauri_deps[name]:
            continue
        detail = f"{name} root={root_deps[name]} tauri={tauri_deps[name]}"
        if name not in exceptions:
            failures.append(detail)
        else:
            drifts.append(detail)
    failures.extend(validate_msrv())
    for manifest in internal_umbrella_dependents():
        failures.append(
            f"internal crate depends on umbrella ccr facade: {manifest}; use the owning domain crate"
        )

    if failures:
        print("Root/Tauri dependency governance check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    if args.verbose:
        print(f"Repeated dependencies checked: {len(repeated)}")
        print(f"Active exceptions: {len(exceptions)}")
        for drift in drifts:
            print(f"  - {drift}")
    print("root/Tauri dependency and MSRV governance check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
