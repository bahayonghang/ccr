"""Test exact vendored UIA modules without upstream visible-window test fixtures."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile

from desktop import ROOT, cargo_environment, isolated_target, metadata, resolve_cargo


def source_hashes(root: Path) -> dict[str, str]:
    return {
        path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(root.rglob("*.rs"))
    }


def locked_packages(path: Path) -> set[tuple[str, str, str | None, str | None]]:
    import tomllib

    with path.open("rb") as stream:
        packages = tomllib.load(stream)["package"]
    return {
        (item["name"], item["version"], item.get("source"), item.get("checksum"))
        for item in packages
    }


def main() -> int:
    if platform.system() != "Windows":
        print("AccessKit Windows close regression: not_applicable (Windows only)")
        return 0

    import tomllib

    target = isolated_target(metadata())
    cargo = resolve_cargo()
    env = cargo_environment(cargo)
    print(f"AccessKit harness Cargo: {cargo}", flush=True)
    subprocess.run([cargo, "--version"], cwd=ROOT, env=env, check=True)
    vendor = ROOT / "vendor/accesskit_windows"
    before = source_hashes(vendor / "src")
    desktop_lock = locked_packages(ROOT / "Cargo.lock")
    manifest = (vendor / "Cargo.toml").read_text(encoding="utf-8")
    tables = tomllib.loads(manifest)
    expected = {"once_cell", "parking_lot", "scopeguard", "winit"}
    if set(tables.get("dev-dependencies", {})) != expected:
        raise ValueError("Upstream test dependencies changed; review the isolated harness")
    # These four tables are the manifest tail in the exact 0.34.0 source.
    start = manifest.index("[dev-dependencies.once_cell]")
    if any(line.startswith("[") and not line.startswith("[dev-dependencies.")
           for line in manifest[start:].splitlines()):
        raise ValueError("Unexpected manifest tail; refusing to remove production tables")
    manifest = manifest[:start] + '\n[workspace]\n\n[profile.dev]\ndebug = "line-tables-only"\n'

    target.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="accesskit-close-", dir=target) as directory:
        harness = Path(directory)
        shutil.copytree(vendor / "src", harness / "src")
        if source_hashes(harness / "src") != before:
            raise ValueError("Harness source is not byte-identical to the production patch")
        # Do not compile the unrelated upstream tests that create native windows.
        # All production modules and CCR private regressions remain byte-identical.
        library = harness / "src/lib.rs"
        body = library.read_bytes()
        entry = b"#[cfg(test)]\nmod tests;"
        if body.count(entry) != 1:
            raise ValueError("Upstream test module entry changed; review the harness")
        library.write_bytes(body.replace(entry, b"", 1))
        (harness / "Cargo.toml").write_text(manifest, encoding="utf-8")
        shutil.copyfile(ROOT / "Cargo.lock", harness / "Cargo.lock")

        print(json.dumps({"production_sources_sha256": before,
                          "transformations": ["omit upstream mod tests", "omit four upstream dev-dependencies"],
                          "harness": str(harness)}, sort_keys=True), flush=True)
        # Reconcile only the temporary root package/dev-dependency list offline.
        result = subprocess.run([
            cargo, "metadata", "--manifest-path", str(harness / "Cargo.toml"),
            "--offline", "--format-version", "1",
        ], cwd=ROOT, env=env, capture_output=True, text=True)
        if result.stderr:
            print(result.stderr, file=sys.stderr, end="")
        if result.returncode:
            return result.returncode
        selected = locked_packages(harness / "Cargo.lock")
        if selected - desktop_lock:
            raise ValueError(f"Harness dependency differs from production lock: {selected - desktop_lock}")
        print(f"Verified {len(selected)} harness packages match the desktop lock", flush=True)
        result = subprocess.run([
            cargo, "test", "--manifest-path", str(harness / "Cargo.toml"),
            "--target-dir", str(target), "--locked", "--offline", "--lib",
            "ccr_invalid_hwnd", "--", "--skip", "export_bindings",
        ], cwd=ROOT, env=env)
        copied = source_hashes(harness / "src")
        copied.pop("lib.rs")
        expected_sources = before.copy()
        expected_sources.pop("lib.rs")
        if copied != expected_sources or source_hashes(vendor / "src") != before:
            raise ValueError("Production modules changed during the regression run")
        return result.returncode


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
