#!/usr/bin/env python3
"""Validate ccr-ui README, Bun lock, and Tauri manifest facts."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

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


REQUIRED_FILES = (
    "ccr-ui/README.md",
    "ccr-ui/package.json",
    "ccr-ui/bun.lock",
    "ccr-ui/src-tauri/Cargo.toml",
)
NPM_LOCK = "ccr-ui/package-lock.json"
PACKAGE_MANAGER_RE = re.compile(r"^bun@[0-9]")
RUST_VERSION_RE = re.compile(r'(?m)^\s*rust-version\s*=\s*"([^"]+)"')
EDITION_RE = re.compile(r'(?m)^\s*edition\s*=\s*"([^"]+)"')
STALE_PATTERNS = (
    "version-2.5.0",
    "TypeScript-5.7",
    "Rust >= 1.70",
    "Edition 2021",
    "Tokio 1.48",
    "Axios",
    "HTTP API",
    "13 个命令",
    "Web 模式: 浏览器访问，通过 HTTP API",
    "自动检测环境，透明切换后端",
)


def required_readme_needles(
    frontend_version: str,
    package_manager: str,
    rust_version: str,
    edition: str,
) -> tuple[str, ...]:
    return (
        f"version-{frontend_version}",
        "Bun is the only maintained frontend package manager",
        "bun.lock is the dependency source of truth",
        f"Bun | `{package_manager}`",
        f"Rust | `>= {rust_version}`",
        f"Rust edition | Edition {edition}",
        "Tauri invoke APIs",
        "Web runtime",
        "bun run lint:fix",
    )


def check_doc_drift(root: Path = REPO_ROOT) -> list[str]:
    failures: list[str] = []
    for relative in REQUIRED_FILES:
        if not (root / relative).is_file():
            failures.append(f"文件不存在: {relative}")
    if (root / NPM_LOCK).is_file():
        failures.append("ccr-ui/package-lock.json 存在；ccr-ui 只维护 Bun/bun.lock")
    if failures:
        return failures

    package_path = root / "ccr-ui" / "package.json"
    try:
        package = json.loads(package_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        return [f"ccr-ui/package.json: {error}"]

    frontend_version = str(package.get("version") or "").strip()
    package_manager = str(package.get("packageManager") or "").strip()
    if not frontend_version:
        failures.append("ccr-ui/package.json 缺少 version 字段")
    if not PACKAGE_MANAGER_RE.match(package_manager):
        failures.append(
            f"ccr-ui/package.json#packageManager 必须声明 bun@x.y.z，当前: {package_manager}"
        )

    tauri_cargo = (root / "ccr-ui" / "src-tauri" / "Cargo.toml").read_text(
        encoding="utf-8"
    )
    rust_match = RUST_VERSION_RE.search(tauri_cargo)
    edition_match = EDITION_RE.search(tauri_cargo)
    if rust_match is None:
        failures.append("ccr-ui/src-tauri/Cargo.toml 缺少 rust-version")
    if edition_match is None:
        failures.append("ccr-ui/src-tauri/Cargo.toml 缺少 edition")
    if failures:
        return failures

    rust_version = rust_match.group(1)
    edition = edition_match.group(1)
    readme = (root / "ccr-ui" / "README.md").read_text(encoding="utf-8")
    for needle in required_readme_needles(
        frontend_version, package_manager, rust_version, edition
    ):
        if needle not in readme:
            failures.append(f"ccr-ui/README.md 缺少当前事实: {needle}")
    for pattern in STALE_PATTERNS:
        if pattern in readme:
            failures.append(f"ccr-ui/README.md 仍包含过期描述: {pattern}")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verbose", "-v", action="store_true")
    args = parser.parse_args()

    failures = check_doc_drift()
    if failures:
        print("文档/锁文件 drift 检查失败:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    if args.verbose:
        package = json.loads(
            (REPO_ROOT / "ccr-ui" / "package.json").read_text(encoding="utf-8")
        )
        tauri_cargo = (REPO_ROOT / "ccr-ui" / "src-tauri" / "Cargo.toml").read_text(
            encoding="utf-8"
        )
        rust_match = RUST_VERSION_RE.search(tauri_cargo)
        edition_match = EDITION_RE.search(tauri_cargo)
        print(f"📄 ccr-ui/README.md version: {package.get('version')}")
        print(f"📦 package manager: {package.get('packageManager')}")
        print(
            f"🦀 rust-version: {rust_match.group(1) if rust_match else '?'}, "
            f"edition: {edition_match.group(1) if edition_match else '?'}"
        )
        print("🔒 JS lock strategy: bun.lock only")
    print("✅ 文档/锁文件 drift 检查通过")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
