from __future__ import annotations

import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest

from scripts.common import REPO_ROOT
from scripts.ci.check_workflow_governance import local_aggregate_failures, recipe_block


class ArchitectureAggregateTests(unittest.TestCase):
    def test_all_local_platforms_reuse_complete_desktop_gate(self) -> None:
        source = (REPO_ROOT / "justfile").read_text(encoding="utf-8")
        self.assertEqual(local_aggregate_failures(source), [])
        for platform in ("windows", "linux", "macos"):
            block = recipe_block(source, f"_ci-timed-{platform}")
            broken = source.replace(block, block.replace('"tauri-ci"', '"tauri-bindings-check"'))
            self.assertEqual(len(local_aggregate_failures(broken)), 1, platform)
        desktop = recipe_block(source, "tauri-ci")
        broken = source.replace(desktop, desktop.replace("--all-features", "command_inventory"))
        self.assertTrue(local_aggregate_failures(broken))

    def test_real_aggregate_propagates_desktop_test_failure(self) -> None:
        just = shutil.which("just")
        self.assertIsNotNone(just, "just is required to verify aggregate execution")
        source = (REPO_ROOT / "justfile").read_text(encoding="utf-8")
        blocks = [recipe_block(source, name) for name in (
            "ci", "_ci-timed-windows", "_ci-timed-linux", "_ci-timed-macos", "tauri-ci",
        )]
        self.assertTrue(all(blocks))
        settings = re.findall(r"^set (?:windows-shell|shell) := .*", source, re.MULTILINE)
        leaves = {"dependency-governance-check", "tauri-bindings-check", "tauri-command-inventory-check"}
        leaves.update(re.findall(r'Name\s*=\s*"([^"]+)"', blocks[1]))
        leaves.discard("tauri-ci")
        with tempfile.TemporaryDirectory(prefix="ccr-aggregate-") as temporary:
            root = Path(temporary)
            runner = root / "fixture.py"
            runner.write_text(
                "import json, os, pathlib, sys\n"
                "args = sys.argv[1:]\n"
                "with pathlib.Path('calls.jsonl').open('a', encoding='utf-8') as out:\n"
                "    out.write(json.dumps(args) + chr(10))\n"
                "if args[0] == 'cargo' and 'test' in args and '--all-features' in args:\n"
                "    assert os.environ.get('CCR_FIXTURE_DESKTOP_FAIL') != '1', 'injected desktop behavior failure'\n",
                encoding="utf-8",
            )
            python = shutil.which("python") or shutil.which("python3")
            self.assertIsNotNone(python)
            if os.name == "nt":
                (root / "cargo.cmd").write_text(f'@"{python}" "%~dp0fixture.py" cargo %*\n', encoding="utf-8")
            else:
                shim = root / "cargo"
                shim.write_text(f'#!/bin/sh\nexec "{python}" "{runner}" cargo "$@"\n', encoding="utf-8")
                shim.chmod(0o755)
            python_command = "python" if os.name == "nt" else "python3"
            stubs = [f"{name}:\n    @{python_command} fixture.py {name}\n" for name in sorted(leaves)]
            stubs.append(f'success message:\n    @{python_command} fixture.py success\n')
            (root / "justfile").write_text("\n".join(settings + blocks + stubs), encoding="utf-8")
            env = {**os.environ, "PATH": str(root) + os.pathsep + os.environ["PATH"]}
            for fail in (True, False):
                with self.subTest(desktop_failure=fail):
                    trace = root / "calls.jsonl"
                    trace.unlink(missing_ok=True)
                    env["CCR_FIXTURE_DESKTOP_FAIL"] = "1" if fail else "0"
                    result = subprocess.run([just, "ci"], cwd=root, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=90)
                    self.assertEqual(result.returncode != 0, fail, result.stdout + result.stderr)
                    calls = [json.loads(line) for line in trace.read_text(encoding="utf-8").splitlines()]
                    self.assertTrue(any(call[0] == "cargo" and "clippy" in call and "-D" in call for call in calls))
                    self.assertTrue(any(call[0] == "cargo" and "test" in call and "--all-features" in call for call in calls))
                    desktop_tests = next(call for call in calls if call[0] == "cargo" and "test" in call)
                    self.assertEqual(desktop_tests[-3:], ["--", "--skip", "export_bindings"])
                    self.assertEqual(["frontend-check"] in calls, not fail)
                    self.assertEqual(["tauri-bindings-check"] in calls, not fail)
                    if fail:
                        self.assertIn("injected desktop behavior failure", result.stderr)

    def test_ui_behavior_entries_do_not_write_bindings(self) -> None:
        ui = REPO_ROOT / "ccr-ui"
        source = (ui / "justfile").read_text(encoding="utf-8")
        commands = re.findall(r"cargo test[^\n]*", source)
        self.assertEqual(len(commands), 3)
        self.assertTrue(all(command.endswith("-- --skip export_bindings") for command in commands))
        package = json.loads((ui / "package.json").read_text(encoding="utf-8"))
        self.assertEqual(package["scripts"]["tauri:test"], "cd src-tauri && cargo test -- --skip export_bindings")


if __name__ == "__main__":
    unittest.main()
