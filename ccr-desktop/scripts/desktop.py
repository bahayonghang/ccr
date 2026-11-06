"""Read-only prerequisites and boundary-checked cleaning for this workspace."""
from __future__ import annotations

import json
import hashlib
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def resolve_cargo() -> str:
    """Bypass only an officially identified mbx shim; preserve ordinary Cargo."""
    selected = shutil.which("cargo")
    if not selected:
        raise ValueError("Cargo is missing from PATH")
    mbx = shutil.which("mbx")
    if not mbx:
        return selected
    status = subprocess.run([mbx, "setup", "--status"], cwd=ROOT,
                            capture_output=True, text=True)
    first = status.stdout.splitlines()[0] if status.stdout else ""
    prefix = "mbx setup is installed and current: "
    if status.returncode or not first.startswith(prefix):
        # A failed mbx inspection must not block a positively identified plain
        # rustup proxy or the active toolchain's actual Cargo executable.
        rustup = shutil.which("rustup")
        if rustup and Path(rustup).is_file():
            if Path(selected).samefile(rustup):
                return selected
            resolved = subprocess.run([rustup, "which", "cargo"], cwd=ROOT,
                                      capture_output=True, text=True)
            actual = Path(resolved.stdout.strip())
            if not resolved.returncode and actual.is_file() and Path(selected).samefile(actual):
                return selected
        raise ValueError("Cannot verify Cargo shim identity from mbx setup --status")
    installed = Path(first[len(prefix):].strip())
    if not installed.is_file():
        raise ValueError("mbx reports a missing Cargo shim; refusing an ambiguous build")
    if not Path(selected).samefile(installed):
        return selected
    rustup = shutil.which("rustup")
    if not rustup:
        raise ValueError("The mbx Cargo shim requires rustup to preserve the desktop target")
    result = subprocess.run([rustup, "which", "cargo"], cwd=ROOT,
                            capture_output=True, text=True, check=True)
    actual = Path(result.stdout.strip())
    if not actual.is_file() or actual.samefile(installed):
        raise ValueError("rustup did not resolve an independent Cargo toolchain")
    return str(actual)


def cargo_environment(cargo: str) -> dict[str, str] | None:
    selected = shutil.which("cargo")
    if selected and Path(cargo).is_file() and not Path(cargo).samefile(selected):
        # Direct Cargo must use its matching compiler even when a dependency's
        # cwd is outside this project and a rustup proxy would choose a default.
        env = os.environ.copy()
        env["PATH"] = str(Path(cargo).parent) + os.pathsep + env.get("PATH", "")
        return env
    return None


def isolated_target(metadata: dict, root: Path = ROOT) -> Path:
    root = root.resolve()
    if Path(metadata["workspace_root"]).resolve() != root:
        raise ValueError("Cargo workspace_root is not ccr-desktop")
    target = Path(metadata["target_directory"])
    expected = root / "target"
    # Refuse inherited target overrides and junctions/symlinks outside this checkout.
    if target.absolute() != expected or target.resolve() != expected.absolute():
        raise ValueError(f"Refusing non-isolated Cargo target: {target}")
    build = Path(metadata.get("build_directory", target))
    if build.absolute() != expected or build.resolve() != expected.absolute():
        raise ValueError(f"Refusing non-isolated Cargo build directory: {build}")
    return target


def metadata() -> dict:
    cargo = resolve_cargo()
    result = subprocess.run(
        [cargo, "metadata", "--locked", "--no-deps", "--format-version", "1"],
        cwd=ROOT, env=cargo_environment(cargo), capture_output=True, text=True, check=True,
    )
    return json.loads(result.stdout)


def doctor() -> int:
    print(f"CCR GPUI doctor: {platform.platform()}")
    failed = False
    for tool in ("cargo", "rustc", "just"):
        found = shutil.which(tool)
        if tool == "cargo" and found:
            found = resolve_cargo()
        print(f"{tool}: {found or 'MISSING'}")
        if found:
            env = cargo_environment(found) if tool == "cargo" else None
            failed |= subprocess.run([found, "--version"], cwd=ROOT, env=env).returncode != 0
        else:
            failed = True
    try:
        print(f"isolated target: {isolated_target(metadata())}")
    except (ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        failed = True
    system = platform.system()
    if system == "Windows":
        # VS tools need not be on PATH: rustc discovers them through the VS registry.
        programs = Path(os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)"))
        vswhere = programs / "Microsoft Visual Studio/Installer/vswhere.exe"
        linker = ""
        if vswhere.is_file():
            result = subprocess.run([
                str(vswhere), "-latest", "-products", "*", "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-find",
                "VC/Tools/MSVC/**/bin/Hostx64/x64/link.exe",
            ], capture_output=True, text=True, check=True)
            linker = result.stdout.strip()
        print(f"MSVC linker: {linker or 'MISSING'}")
        failed |= not bool(linker)
        libraries = list((programs / "Windows Kits/10/Lib").glob("*/um/x64/kernel32.lib"))
        print(f"Windows SDK: {libraries[-1] if libraries else 'MISSING'}")
        failed |= not bool(libraries)
        graphics = subprocess.run([
            "pwsh.exe", "-NoLogo", "-NoProfile", "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion | ConvertTo-Json -Compress",
        ], capture_output=True, text=True, timeout=15)
        print(f"GPU / driver inventory: {graphics.stdout.strip() or graphics.stderr.strip() or 'MISSING'}")
        failed |= graphics.returncode != 0 or not bool(graphics.stdout.strip())
        print("DirectX rendering is verified only by native launch, not GPU inventory.")
    elif system == "Darwin":
        failed |= subprocess.run(["xcode-select", "-p"]).returncode != 0
        print("Required: Xcode tools, Metal GPU/driver and a logged-in graphical session.")
    elif system == "Linux":
        packages = ("fontconfig", "freetype2", "xkbcommon", "xkbcommon-x11", "wayland-client", "xcb")
        print(f"Linux development libraries (pkg-config): {', '.join(packages)}")
        if shutil.which("pkg-config"):
            failed |= subprocess.run(["pkg-config", "--exists", "--print-errors", *packages]).returncode != 0
        else:
            print("pkg-config: MISSING")
            failed = True
        print("Required: C/C++ linker, Vulkan GPU/driver, X11 or Wayland session.")
        if not (os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY")):
            print("Graphical session: MISSING")
            failed = True
    else:
        print(f"Unsupported desktop OS: {system}")
        failed = True
    print("Native GPU, CJK IME, accessibility and DPI remain manual checks; doctor is not native evidence.")
    return int(failed)


def built_executable(result: subprocess.CompletedProcess, name: str, target: Path) -> Path:
    executable = None
    for line in result.stdout.splitlines():
        event = json.loads(line)
        if (event.get("reason") == "compiler-artifact"
                and event.get("target", {}).get("name") == name
                and event.get("executable")):
            executable = Path(event["executable"])
    expected = target / "release" / (f"{name}.exe" if os.name == "nt" else name)
    if executable is None or executable.resolve() != expected.absolute() or not executable.is_file():
        raise ValueError("No native release executable produced; cross-target packaging is unsupported")
    return executable


def file_sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def package() -> int:
    """Build a matching CLI and bind its hash into the portable desktop."""
    info = metadata()
    target = isolated_target(info)
    cargo = resolve_cargo()
    version = next(item["version"] for item in info["packages"] if item["name"] == "ccr-gpui")
    sidecar_target = target / "sidecar"
    for build_path in (sidecar_target, sidecar_target / "release", target / "release"):
        if build_path.resolve() != build_path.absolute():
            raise ValueError("Refusing non-isolated package build directory")
    result = subprocess.run(
        [cargo, "build", "--manifest-path", str(ROOT.parent / "Cargo.toml"),
         "--locked", "--release", "--package", "ccr", "--bin", "ccr",
         "--target-dir", str(sidecar_target), "--message-format=json-render-diagnostics"],
        cwd=ROOT.parent, env=cargo_environment(cargo), stdout=subprocess.PIPE, text=True,
    )
    if result.returncode:
        return result.returncode
    sidecar = built_executable(result, "ccr", sidecar_target)
    # Clap exits on --version before logging, migration, or command dispatch.
    probe = subprocess.run([str(sidecar), "--version"], capture_output=True, text=True, timeout=5)
    if probe.returncode or probe.stdout.strip() != f"ccr {version}":
        raise ValueError("The built CCR sidecar version does not match the desktop")
    sidecar_hash = file_sha256(sidecar)
    build_env = cargo_environment(cargo)
    build_env = dict(os.environ if build_env is None else build_env)
    build_env["CCR_SIDECAR_SHA256"] = sidecar_hash
    result = subprocess.run(
        [cargo, "build", "--locked", "--release", "--bin", "ccr-gpui",
         "--message-format=json-render-diagnostics"],
        cwd=ROOT, env=build_env, stdout=subprocess.PIPE, text=True,
    )
    if result.returncode:
        return result.returncode
    executable = built_executable(result, "ccr-gpui", target)
    label = f"ccr-gpui-{version}-{platform.system().lower()}-{platform.machine().lower()}"
    if any(char not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.-_" for char in label):
        raise ValueError("Invalid package version or host label")
    destination = target / "package"
    if destination.resolve() != destination.absolute():
        raise ValueError("Refusing non-isolated package directory")
    destination.mkdir(exist_ok=True)
    digest = file_sha256(executable)
    manifest = {
        "version": version,
        "platform": platform.system(),
        "architecture": platform.machine(),
        "format": "portable",
        "signed": False,
        "executable": executable.name,
        "sha256": digest,
        "sidecar": {"executable": sidecar.name, "version": version, "sha256": sidecar_hash},
    }
    archive = destination / f"{label}.zip"
    with tempfile.NamedTemporaryFile(dir=destination, suffix=".zip", delete=False) as temporary:
        staging = Path(temporary.name)
    try:
        with zipfile.ZipFile(staging, "w", zipfile.ZIP_DEFLATED) as output:
            output.write(executable, executable.name)
            output.write(sidecar, sidecar.name)
            output.write(ROOT.parent / "LICENSE", "LICENSE")
            output.writestr("manifest.json", json.dumps(manifest, indent=2) + "\n")
        with zipfile.ZipFile(staging) as output:
            for name, expected_hash in ((executable.name, digest), (sidecar.name, sidecar_hash)):
                with output.open(name) as stream:
                    if hashlib.file_digest(stream, "sha256").hexdigest() != expected_hash:
                        raise ValueError("Build output changed while packaging; retry the package build")
        staging.replace(archive)
    finally:
        staging.unlink(missing_ok=True)
    print(f"Unsigned portable package: {archive}")
    return 0


def main() -> int:
    if sys.argv[1:] == ["verify"]:
        print(f"Verified workspace-local target: {isolated_target(metadata())}")
        return 0
    if sys.argv[1:] == ["doctor"]:
        return doctor()
    if sys.argv[1:] == ["package"]:
        return package()
    if sys.argv[1:] == ["clean"]:
        target = isolated_target(metadata())
        print(f"Verified workspace-local target: {target}")
        cargo = resolve_cargo()
        return subprocess.run([cargo, "clean", "--target-dir", str(target)],
                              cwd=ROOT, env=cargo_environment(cargo)).returncode
    if sys.argv[1:2] == ["cargo"] and len(sys.argv) > 2:
        target = isolated_target(metadata())
        print(f"Verified workspace-local target: {target}", flush=True)
        cargo = resolve_cargo()
        print(f"Desktop Cargo: {cargo}", flush=True)
        env = cargo_environment(cargo)
        if env is not None:
            print("Verified mbx shim bypass; matching toolchain preserves workspace-local target", flush=True)
        return subprocess.run([cargo, *sys.argv[2:]], cwd=ROOT,
                              env=env).returncode
    print("Usage: desktop.py verify|doctor|clean|package|cargo <args>", file=sys.stderr)
    return 2


if __name__ == "__main__":
    try:
        sys.exit(main())
    except subprocess.CalledProcessError as error:
        print(error.stderr or str(error), file=sys.stderr)
        sys.exit(error.returncode)
    except (ValueError, KeyError, OSError, subprocess.TimeoutExpired) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
