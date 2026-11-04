#!/usr/bin/env python3
"""Install pinned prebuilt CI tools.

Hosted jobs must not compile ``just``, ``cargo-audit``, or ``cargo-llvm-cov``
with ``cargo install``. Checksums are the release assets verified for the
versions already used by this repository.
"""

from __future__ import annotations

import hashlib
import io
import os
import platform
import stat
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path

@dataclass(frozen=True)
class Asset:
    url: str
    sha256: str
    binary: str


ASSETS: dict[tuple[str, str, str], Asset] = {
    ("just", "linux", "x86_64"): Asset(
        "https://github.com/casey/just/releases/download/1.58.0/just-1.58.0-x86_64-unknown-linux-musl.tar.gz",
        "4a5cc2f53e6f0f8c59092a6cc38291eb729d46a7dd95d3ae582008881b84931d",
        "just",
    ),
    ("just", "linux", "aarch64"): Asset(
        "https://github.com/casey/just/releases/download/1.58.0/just-1.58.0-aarch64-unknown-linux-musl.tar.gz",
        "748237128c4c40cbdabc65e841d05ceba13cc23a91eaba395495894c1d9764df",
        "just",
    ),
    ("just", "darwin", "x86_64"): Asset(
        "https://github.com/casey/just/releases/download/1.58.0/just-1.58.0-x86_64-apple-darwin.tar.gz",
        "9a09cfef66aaa79da58203970103a0684307716caaabd3e9844cacc4dc0f4023",
        "just",
    ),
    ("just", "darwin", "aarch64"): Asset(
        "https://github.com/casey/just/releases/download/1.58.0/just-1.58.0-aarch64-apple-darwin.tar.gz",
        "50ae3e996c974a0bf32ea7d10f495070df33f1b43e0616b2769e3d4821ed8f48",
        "just",
    ),
    ("just", "windows", "x86_64"): Asset(
        "https://github.com/casey/just/releases/download/1.58.0/just-1.58.0-x86_64-pc-windows-msvc.zip",
        "759f16fb7aa17c5c8b9594b6d4a8c1a6630dfd042cf2b3ff84841454d3d188dc",
        "just",
    ),
    ("cargo-audit", "linux", "x86_64"): Asset(
        "https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.22.2/cargo-audit-x86_64-unknown-linux-gnu-v0.22.2.tgz",
        "ab28a1bdb54db4d5d8ad5981cf1f959410370b3d28250dbd35f6a44248620e39",
        "cargo-audit",
    ),
    ("cargo-audit", "darwin", "aarch64"): Asset(
        "https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.22.2/cargo-audit-aarch64-apple-darwin-v0.22.2.tgz",
        "ec7ca4263769593df4d909be85b94a6b79efa2897be5d2bb8ebd516e823175af",
        "cargo-audit",
    ),
    ("cargo-audit", "windows", "x86_64"): Asset(
        "https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.22.2/cargo-audit-x86_64-pc-windows-msvc-v0.22.2.zip",
        "0a7316540862c13d954f648917ceacca593747baed6eec180fafa590be2710ab",
        "cargo-audit",
    ),
    ("cargo-llvm-cov", "linux", "x86_64"): Asset(
        "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.9.0/cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz",
        "b068f7c98841aacb9c4f382b4a0c184ae82f49b56a32d442b429b2961c73be15",
        "cargo-llvm-cov",
    ),
}


def host_system() -> str:
    system = platform.system().lower()
    if system.startswith("win"):
        return "windows"
    if system == "darwin":
        return "darwin"
    if system == "linux":
        return "linux"
    raise SystemExit(f"unsupported operating system: {system}")


def host_arch() -> str:
    machine = platform.machine().lower()
    if machine in {"amd64", "x86_64"}:
        return "x86_64"
    if machine in {"arm64", "aarch64"}:
        return "aarch64"
    raise SystemExit(f"unsupported architecture: {machine}")


def asset_for(tool: str, system: str, arch: str) -> Asset:
    try:
        return ASSETS[(tool, system, arch)]
    except KeyError:
        known = ", ".join(
            f"{known_tool}/{known_system}/{known_arch}"
            for known_tool, known_system, known_arch in sorted(ASSETS)
        )
        raise SystemExit(
            f"no prebuilt {tool} asset for {system}/{arch}. Known assets: {known}"
        ) from None


def verify_sha256(blob: bytes, expected: str) -> None:
    actual = hashlib.sha256(blob).hexdigest()
    if actual != expected:
        raise SystemExit(f"checksum mismatch: expected {expected}, got {actual}")


def _member_name(name: str, binary: str) -> bool:
    basename = Path(name).name
    return basename in {binary, f"{binary}.exe"} and ".." not in Path(name).parts


def extract_binary(blob: bytes, url: str, binary: str) -> tuple[str, bytes]:
    if url.endswith(".zip"):
        with zipfile.ZipFile(io.BytesIO(blob)) as archive:
            matches = [name for name in archive.namelist() if _member_name(name, binary)]
            if len(matches) != 1:
                raise SystemExit(f"{url} does not contain exactly one {binary} binary: {matches}")
            return Path(matches[0]).name, archive.read(matches[0])
    with tarfile.open(fileobj=io.BytesIO(blob), mode="r:gz") as archive:
        matches = [name for name in archive.getnames() if _member_name(name, binary)]
        if len(matches) != 1:
            raise SystemExit(f"{url} does not contain exactly one {binary} binary: {matches}")
        extracted = archive.extractfile(matches[0])
        if extracted is None:
            raise SystemExit(f"{url} member {matches[0]} is not a file")
        return Path(matches[0]).name, extracted.read()


def download(url: str) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": "ccr-ci-prebuilt"})
    last_error: Exception | None = None
    for _ in range(3):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return response.read()
        except Exception as error:  # noqa: BLE001 - retry transient release-download failures
            last_error = error
    raise SystemExit(f"failed to download {url}: {last_error}")


def install_dir() -> Path:
    base = os.environ.get("RUNNER_TEMP") or tempfile.gettempdir()
    path = Path(base) / "ccr-prebuilt"
    path.mkdir(parents=True, exist_ok=True)
    return path


def publish(directory: Path) -> None:
    github_path = os.environ.get("GITHUB_PATH")
    if github_path:
        with open(github_path, "a", encoding="utf-8") as handle:
            handle.write(str(directory) + "\n")
        return
    print(f"prebuilt tools are in {directory}; add it to PATH", file=sys.stderr)


def install(tool: str, system: str | None = None, arch: str | None = None) -> Path:
    selected = asset_for(tool, system or host_system(), arch or host_arch())
    blob = download(selected.url)
    verify_sha256(blob, selected.sha256)
    filename, data = extract_binary(blob, selected.url, selected.binary)
    destination = install_dir() / filename
    destination.write_bytes(data)
    destination.chmod(destination.stat().st_mode | stat.S_IEXEC)
    publish(destination.parent)
    print(f"installed {destination}")
    return destination


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit(f"usage: {sys.argv[0]} <just|cargo-audit|cargo-llvm-cov>")
    install(sys.argv[1])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
