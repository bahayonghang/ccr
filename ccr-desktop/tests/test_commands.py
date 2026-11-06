"""Guard destructive boundaries and the public command graph, without compiling GPUI."""
import importlib.util
import io
import json
import os
from contextlib import redirect_stdout
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("desktop", ROOT / "scripts/desktop.py")
desktop = importlib.util.module_from_spec(spec)
spec.loader.exec_module(desktop)


class CommandContract(unittest.TestCase):
    def test_package_rejects_redirected_build_paths_before_building(self):
        with tempfile.TemporaryDirectory() as fixture:
            target = Path(fixture) / "target"
            info = {"packages": [{"name": "ccr-gpui", "version": "1.2.3"}]}
            resolve = Path.resolve
            for redirected in (target / "sidecar", target / "sidecar/release", target / "release"):
                with (
                    self.subTest(redirected=redirected),
                    mock.patch.object(desktop, "metadata", return_value=info),
                    mock.patch.object(desktop, "isolated_target", return_value=target),
                    mock.patch.object(desktop, "resolve_cargo", return_value="fixture-cargo"),
                    mock.patch.object(Path, "resolve", autospec=True,
                                      side_effect=lambda path, *args, **kwargs:
                                      Path(fixture) / "external" if path == redirected
                                      else resolve(path, *args, **kwargs)),
                    mock.patch.object(desktop.subprocess, "run") as run,
                ):
                    with self.assertRaisesRegex(ValueError, "non-isolated package build"):
                        desktop.package()
                    run.assert_not_called()

    def test_package_contains_only_the_built_binary_license_and_manifest(self):
        with tempfile.TemporaryDirectory() as fixture:
            root = Path(fixture) / "desktop"
            target = root / "target"
            binary = target / "release" / ("ccr-gpui.exe" if os.name == "nt" else "ccr-gpui")
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"fixture binary")
            sidecar = target / "sidecar" / "release" / ("ccr.exe" if os.name == "nt" else "ccr")
            sidecar.parent.mkdir(parents=True)
            sidecar.write_bytes(b"fixture sidecar")
            (root.parent / "LICENSE").write_text("fixture license")
            (binary.parent / "private.txt").write_text("must not ship")
            info = {"workspace_root": str(ROOT), "target_directory": str(ROOT / "target"),
                    "packages": [{"name": "ccr-gpui", "version": "1.2.3"}]}
            event = json.dumps({"reason": "compiler-artifact", "target": {"name": "ccr-gpui"},
                                "executable": str(binary)})
            sidecar_event = json.dumps({"reason": "compiler-artifact", "target": {"name": "ccr"},
                                       "executable": str(sidecar)})
            def build_result(command, **kwargs):
                if command[-1] == "--version":
                    return subprocess.CompletedProcess(command, 0, "ccr 1.2.3\n")
                return subprocess.CompletedProcess(command, 0, sidecar_event if "--package" in command else event)
            with (
                mock.patch.object(desktop, "ROOT", root),
                mock.patch.object(desktop, "metadata", return_value=info),
                mock.patch.object(desktop, "isolated_target", return_value=target),
                mock.patch.object(desktop, "resolve_cargo", return_value="fixture-cargo"),
                mock.patch.object(desktop.platform, "system", return_value="Windows"),
                mock.patch.object(desktop.platform, "machine", return_value="AMD64"),
                mock.patch.object(desktop.subprocess, "run", side_effect=build_result) as run,
                mock.patch.dict(os.environ, {"CCR_SIDECAR_SHA256": "untrusted-parent-value"}),
                redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(desktop.package(), 0)
                self.assertIn("--locked", run.call_args.args[0])
                self.assertIn("--target-dir", run.call_args_list[0].args[0])
                self.assertEqual(run.call_args.kwargs["env"]["CCR_SIDECAR_SHA256"],
                                 desktop.hashlib.sha256(b"fixture sidecar").hexdigest())
                self.assertEqual(os.environ["CCR_SIDECAR_SHA256"], "untrusted-parent-value")
                archive = next((target / "package").glob("*.zip"))
                original = archive.read_bytes()
                with zipfile.ZipFile(archive) as output:
                    self.assertEqual(set(output.namelist()), {binary.name, sidecar.name, "LICENSE", "manifest.json"})
                    self.assertEqual(output.read(binary.name), b"fixture binary")
                    self.assertEqual(output.read(sidecar.name), b"fixture sidecar")
                    manifest = json.loads(output.read("manifest.json"))
                    self.assertEqual(manifest["sha256"], desktop.hashlib.sha256(b"fixture binary").hexdigest())
                    self.assertFalse(manifest["signed"])
                    self.assertEqual(manifest["sidecar"]["sha256"],
                                     desktop.hashlib.sha256(b"fixture sidecar").hexdigest())
                run.side_effect = None
                run.return_value = subprocess.CompletedProcess([], 23, "")
                self.assertEqual(desktop.package(), 23)
                self.assertEqual(archive.read_bytes(), original)
                foreign = {**json.loads(event), "executable": str(root / "foreign.exe")}
                run.side_effect = [subprocess.CompletedProcess([], 0, sidecar_event),
                                   subprocess.CompletedProcess([], 0, "ccr 1.2.3\n"),
                                   subprocess.CompletedProcess([], 0, json.dumps(foreign))]
                with self.assertRaisesRegex(ValueError, "No native release"):
                    desktop.package()
                run.side_effect = [subprocess.CompletedProcess([], 0, sidecar_event),
                                   subprocess.CompletedProcess([], 0, "ccr 1.2.30\n")]
                with self.assertRaisesRegex(ValueError, "version does not match"):
                    desktop.package()

    def test_dev_preserves_argument_boundaries_through_both_justfiles(self):
        with tempfile.TemporaryDirectory() as fixture:
            folder = Path(fixture)
            captured = folder / "arguments.json"
            helper = folder / "cargo.py"
            data = {"workspace_root": str(ROOT), "target_directory": str(ROOT / "target")}
            helper.write_text(
                "import json, pathlib, sys\n"
                f"metadata = {data!r}\n"
                "if sys.argv[1] == 'metadata':\n"
                "    print(json.dumps(metadata))\n"
                "else:\n"
                f"    pathlib.Path({str(captured)!r}).write_text(json.dumps(sys.argv[1:]))\n"
                "    sys.exit(23)\n",
                encoding="utf-8",
            )
            shim = folder / "identified-shim"
            shim.touch()
            status = f"mbx setup is installed and current: {shim}"
            if os.name == "nt":
                (folder / "cargo.cmd").write_text(f'@echo off\n"{sys.executable}" "{helper}" %*\n')
                (folder / "mbx.cmd").write_text(f'@echo off\necho {status}\n')
            else:
                import shlex
                cargo = folder / "cargo"
                cargo.write_text(f'#!/bin/sh\nexec {shlex.quote(sys.executable)} {shlex.quote(str(helper))} "$@"\n')
                cargo.chmod(0o755)
                mbx = folder / "mbx"
                mbx.write_text(f"#!/bin/sh\nprintf '%s\\n' {shlex.quote(status)}\n")
                mbx.chmod(0o755)
            args = ["--fixture-profiles", str(folder / "profile data/profiles.toml"),
                    "--fixture-registry", str(folder / "registry data/config.toml")]
            for command in ("gpui-dev", "desktop-dev"):
                with self.subTest(command=command):
                    result = subprocess.run(
                        ["just", "--justfile", str(ROOT.parent / "justfile"), command, *args],
                        env={**os.environ, "PATH": str(folder) + os.pathsep + os.environ["PATH"]},
                        capture_output=True, text=True,
                    )
                    self.assertEqual(result.returncode, 23, result.stdout + result.stderr)
                    self.assertEqual(json.loads(captured.read_text()),
                                     ["run", "--locked", "--bin", "ccr-gpui", "--", *args])

    def test_cargo_without_mbx_is_preserved(self):
        for selected, mbx in (("ordinary-cargo", None), ("fixture-cargo.cmd", None)):
            with (
                self.subTest(selected=selected),
                mock.patch.object(desktop.shutil, "which", side_effect={"cargo": selected, "mbx": mbx}.get),
                mock.patch.object(desktop.subprocess, "run") as run,
            ):
                self.assertEqual(desktop.resolve_cargo(), selected)
                run.assert_not_called()

    def test_only_the_officially_identified_shim_uses_project_toolchain(self):
        with tempfile.TemporaryDirectory() as fixture:
            folder = Path(fixture)
            shim, actual, other = (folder / name for name in ("shim.exe", "actual.exe", "other.exe"))
            for path in (shim, actual, other):
                path.touch()
            status = subprocess.CompletedProcess([], 0, f"mbx setup is installed and current: {shim}\n")
            resolved = subprocess.CompletedProcess([], 0, str(actual) + "\n")
            for selected, expected in ((shim, actual), (other, other)):
                with (
                    self.subTest(selected=selected),
                    mock.patch.object(desktop.shutil, "which", side_effect={
                        "cargo": str(selected), "mbx": "mbx", "rustup": "rustup",
                    }.get),
                    mock.patch.object(desktop.subprocess, "run", side_effect=[status, resolved]) as run,
                    mock.patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": "fixture-toolchain"}),
                ):
                    self.assertEqual(desktop.resolve_cargo(), str(expected))
                    if selected == shim:
                        self.assertEqual(run.call_args.args[0], ["rustup", "which", "cargo"])
                        self.assertEqual(run.call_args.kwargs["cwd"], ROOT)
                        self.assertNotIn("env", run.call_args.kwargs)
                    else:
                        self.assertEqual(run.call_count, 1)

    def test_ambiguous_shim_or_missing_real_toolchain_fails_closed(self):
        with tempfile.TemporaryDirectory() as fixture:
            shim = Path(fixture) / "shim.exe"
            shim.touch()
            known = f"mbx setup is installed and current: {shim}\n"
            for status, rustup, actual in (("unknown status\n", "rustup", str(shim)),
                                           (known, None, str(shim)),
                                           (known, "rustup", str(shim))):
                with (
                    self.subTest(status=status, rustup=rustup),
                    mock.patch.object(desktop.shutil, "which", side_effect={
                        "cargo": str(shim), "mbx": "mbx", "rustup": rustup,
                    }.get),
                    mock.patch.object(desktop.subprocess, "run", side_effect=[
                        subprocess.CompletedProcess([], 0, status),
                        subprocess.CompletedProcess([], 0, actual),
                    ]),
                    self.assertRaises(ValueError),
                ):
                    desktop.resolve_cargo()

    def test_failed_mbx_status_preserves_verified_plain_cargo(self):
        with tempfile.TemporaryDirectory() as fixture:
            proxy, actual = (Path(fixture) / name for name in ("rustup.exe", "cargo.exe"))
            proxy.touch()
            actual.touch()
            for selected in (proxy, actual):
                with (
                    self.subTest(selected=selected),
                    mock.patch.object(desktop.shutil, "which", side_effect={
                        "cargo": str(selected), "mbx": "mbx", "rustup": str(proxy),
                    }.get),
                    mock.patch.object(desktop.subprocess, "run", side_effect=[
                        subprocess.CompletedProcess([], 1, "not installed\n"),
                        subprocess.CompletedProcess([], 0, str(actual)),
                    ]),
                ):
                    self.assertEqual(desktop.resolve_cargo(), str(selected))

    def test_bypassed_shim_uses_matching_compiler_without_global_path_change(self):
        with tempfile.TemporaryDirectory() as fixture:
            shim, actual = (Path(fixture) / name for name in ("shim.exe", "actual.exe"))
            shim.touch()
            actual.touch()
            original = os.environ["PATH"]
            with mock.patch.object(desktop.shutil, "which", return_value=str(shim)):
                env = desktop.cargo_environment(str(actual))
                self.assertEqual(env["PATH"], str(actual.parent) + os.pathsep + original)
                self.assertEqual(os.environ["PATH"], original)
                self.assertIsNone(desktop.cargo_environment(str(shim)))

    def test_metadata_and_execution_use_resolved_cargo_and_preserve_exit(self):
        good = {"workspace_root": str(ROOT), "target_directory": str(ROOT / "target")}
        with (
            mock.patch.object(desktop, "resolve_cargo", return_value="resolved-cargo"),
            mock.patch.object(desktop.subprocess, "run", return_value=
                              subprocess.CompletedProcess([], 0, json.dumps(good))) as run,
        ):
            self.assertEqual(desktop.metadata(), good)
            self.assertEqual(run.call_args.args[0][0], "resolved-cargo")
        with (
            mock.patch.object(desktop, "resolve_cargo", return_value="resolved-cargo"),
            mock.patch.object(desktop, "metadata", return_value=good),
            mock.patch.object(desktop.sys, "argv", ["desktop.py", "cargo", "build", "--locked"]),
            mock.patch.object(desktop.subprocess, "run", return_value=subprocess.CompletedProcess([], 23)) as run,
            redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(desktop.main(), 23)
            self.assertEqual(run.call_args.args[0], ["resolved-cargo", "build", "--locked"])

    def test_linux_doctor_rejects_missing_xkbcommon_x11_development_package(self):
        cargo_metadata = {"workspace_root": str(ROOT), "target_directory": str(ROOT / "target")}

        def tool_result(command, **kwargs):
            missing = command[0] == "pkg-config" and "xkbcommon-x11" in command
            return subprocess.CompletedProcess(command, int(missing))

        with (
            mock.patch.object(desktop.platform, "system", return_value="Linux"),
            mock.patch.object(desktop.platform, "platform", return_value="Linux fixture"),
            mock.patch.object(desktop.shutil, "which", return_value="fixture-tool"),
            mock.patch.object(desktop, "resolve_cargo", return_value="fixture-tool"),
            mock.patch.object(desktop, "metadata", return_value=cargo_metadata),
            mock.patch.object(desktop.subprocess, "run", side_effect=tool_result) as run,
            mock.patch.dict(os.environ, {"DISPLAY": ":fixture"}),
            redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(desktop.doctor(), 1)
        pkg_config = next(call.args[0] for call in run.call_args_list if call.args[0][0] == "pkg-config")
        self.assertIn("xkbcommon-x11", pkg_config)
        self.assertIn("--print-errors", pkg_config)

    def test_build_entrypoints_reject_foreign_target_overrides(self):
        for key in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "CARGO_BUILD_BUILD_DIR"):
            for command in ("dev", "build", "build-debug", "check", "lint", "test", "test-ui"):
                with self.subTest(key=key, command=command):
                    result = subprocess.run(
                        ["just", "--justfile", str(ROOT.parent / "justfile"), f"gpui-{command}"],
                        env={**os.environ, key: str(ROOT.parent / "target")},
                        capture_output=True, text=True,
                    )
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("Refusing non-isolated Cargo", result.stdout + result.stderr)

    def test_actual_cargo_failure_propagates_through_both_justfiles(self):
        with tempfile.TemporaryDirectory() as fixture:
            folder = Path(fixture)
            data = json.dumps({"workspace_root": str(ROOT), "target_directory": str(ROOT / "target")})
            identified_shim = folder / "identified-cargo-shim"
            identified_shim.touch()
            # Keep host mbx state out of this ordinary-Cargo exit-code fixture.
            status = f"mbx setup is installed and current: {identified_shim}"
            if os.name == "nt":
                stub = folder / "cargo.cmd"
                stub.write_text(f'@echo off\nif "%1"=="metadata" (\necho {data}\nexit /b 0\n)\nexit /b 23\n')
                (folder / "mbx.cmd").write_text(f'@echo off\necho {status}\n')
            else:
                stub = folder / "cargo"
                stub.write_text(f'#!/bin/sh\nif [ "$1" = metadata ]; then\nprintf \'%s\\n\' \'{data}\'\nexit 0\nfi\nexit 23\n')
                stub.chmod(0o755)
                mbx = folder / "mbx"
                mbx.write_text(f"#!/bin/sh\nprintf '%s\\n' '{status}'\n")
                mbx.chmod(0o755)
            for command in ("gpui-build", "desktop-build"):
                with self.subTest(command=command):
                    result = subprocess.run(
                        ["just", "--justfile", str(ROOT.parent / "justfile"), command],
                        env={**os.environ, "PATH": str(folder) + os.pathsep + os.environ["PATH"]},
                        capture_output=True, text=True,
                    )
                    self.assertEqual(result.returncode, 23, result.stdout + result.stderr)

    def test_clean_rejects_foreign_workspace_and_target(self):
        with tempfile.TemporaryDirectory() as fixture:
            root = Path(fixture).resolve()
            good = {"workspace_root": str(root), "target_directory": str(root / "target")}
            self.assertEqual(desktop.isolated_target(good, root), root / "target")
            for key in good:
                with self.subTest(key=key), self.assertRaises(ValueError):
                    desktop.isolated_target({**good, key: str(root.parent / "target")}, root)
            with self.assertRaises(ValueError):
                desktop.isolated_target({**good, "build_directory": str(root.parent / "target")}, root)

    def test_ci_is_ordered_and_read_only(self):
        result = subprocess.run(
            ["just", "--justfile", str(ROOT / "justfile"), "--dry-run", "desktop-ci"],
            capture_output=True, text=True, check=True,
        )
        lines = [line.removesuffix("; exit $LASTEXITCODE") for line in (result.stdout + result.stderr).strip().splitlines()]
        self.assertEqual(lines, [f"just desktop-{step}" for step in (
            "fmt-check", "check", "lint", "test", "build"
        )])

    def test_root_forwards_from_child_working_directory(self):
        commands = ("help", "doctor", "dev", "build", "package", "build-debug", "check", "fmt",
                    "fmt-check", "lint", "test", "test-ui", "ci", "audit", "clean")
        listed = subprocess.run(
            ["just", "--justfile", str(ROOT.parent / "justfile"), "--summary"],
            capture_output=True, text=True, check=True,
        ).stdout.split()
        for command in commands:
            with self.subTest(command=command):
                self.assertIn(f"gpui-{command}", listed)
                self.assertNotIn(f"desktop-{command}", listed)
                forwarded = []
                for prefix in ("gpui", "desktop"):
                    result = subprocess.run(
                        ["just", "--justfile", str(ROOT.parent / "justfile"), "--dry-run", f"{prefix}-{command}"],
                        capture_output=True, text=True, check=True,
                    )
                    expected = ('"--justfile", "ccr-desktop/justfile", "desktop-dev"' if command == "dev"
                                else f"--justfile ccr-desktop/justfile desktop-{command}")
                    self.assertIn(expected, result.stderr + result.stdout)
                    forwarded.append(result.stderr + result.stdout)
                self.assertEqual(*forwarded)

    def test_unix_help_expands_just_text_without_shell_variables(self):
        bash = r"C:\Program Files\Git\bin\bash.exe" if os.name == "nt" else "bash"
        env = {key: value for key, value in os.environ.items() if key != "HELP_FLAT"}
        outputs = []
        for platform in ("linux", "macos"):
            with self.subTest(platform=platform):
                rendered = subprocess.run(
                    ["just", "--justfile", str(ROOT.parent / "justfile"), "--dry-run", f"_help-{platform}"],
                    capture_output=True, text=True, encoding="utf-8", check=True,
                )
                command = rendered.stdout + rendered.stderr
                parsed = subprocess.run([bash, "-n"], input=command, capture_output=True,
                                        text=True, encoding="utf-8")
                self.assertEqual(parsed.returncode, 0, parsed.stderr)
                result = subprocess.run(
                    [bash, "--noprofile", "--norc", "-c", command], env=env,
                    capture_output=True, text=True, encoding="utf-8",
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("GPUI      gpui-dev  gpui-build  gpui-ci", result.stdout)
                self.assertIn("完整 recipe：just --list --unsorted", result.stdout)
                self.assertGreater(len(result.stdout.splitlines()), 10)
                outputs.append(result.stdout)
        if len(outputs) == 2:
            self.assertEqual(*outputs)


if __name__ == "__main__":
    unittest.main()
