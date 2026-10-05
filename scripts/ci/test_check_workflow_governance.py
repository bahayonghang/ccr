from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

from scripts.common import REPO_ROOT
from scripts.ci.ci_surface_policy import SURFACE_PATHS, is_relevant, path_matches
from scripts.ci.check_workflow_governance import (
    DEVELOPMENT_RUST_TOOLCHAIN,
    EXPECTED_CI_STEPS,
    EXPECTED_BUN_WORKFLOWS,
    EXPECTED_NODE_WORKFLOW_INPUTS,
    MSRV_RUST_TOOLCHAIN,
    NODE_TOOLCHAIN,
    bun_version_inputs,
    canonical_bun_version,
    check_relevance_failures,
    dependabot_failures,
    dependabot_update_fields,
    duplicate_mapping_keys,
    hosted_gate_failures,
    local_aggregate_failures,
    local_gate_failures,
    node_pin_failures,
    node_version_inputs,
    recipe_block,
    rust_toolchain_inputs,
    setup_node_version_inputs,
    workflow_event_values,
    workflow_job_block,
)


def _workflow_step_fields(text: str, needle: str) -> dict[str, str]:
    """Return scalar keys from the YAML sequence item that contains *needle*."""
    lines = text.splitlines()
    match_index = next(
        (index for index, line in enumerate(lines) if needle in line),
        None,
    )
    if match_index is None:
        return {}

    start: int | None = None
    start_indent = 0
    for index in range(match_index, -1, -1):
        line = lines[index]
        if not line.strip():
            continue
        indent = len(line) - len(line.lstrip(" "))
        if line.lstrip().startswith("- "):
            start = index
            start_indent = indent
            break
    if start is None:
        return {}

    end = len(lines)
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = len(line) - len(line.lstrip(" "))
        if indent < start_indent:
            end = index
            break
        if indent == start_indent and line.lstrip().startswith("- "):
            end = index
            break

    fields: dict[str, str] = {}
    for line in lines[start:end]:
        stripped = line.strip()
        if stripped.startswith("- "):
            stripped = stripped[2:].strip()
        if ":" not in stripped:
            continue
        key, value = stripped.split(":", 1)
        fields[key.strip()] = value.strip()
    return fields


class DependabotGovernanceTests(unittest.TestCase):
    CONFIG = """version: 2
updates:
  - package-ecosystem: github-actions
    directory: /
  - package-ecosystem: cargo
    directory: /
  - package-ecosystem: cargo
    directory: /ccr-ui/src-tauri
  - package-ecosystem: bun
    directory: /ccr-ui
  - package-ecosystem: bun
    directory: /docs
  - package-ecosystem: npm
    directory: /ccr-vscode
"""

    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="ccr-dependabot-test-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.config = self.root / ".github" / "dependabot.yml"
        self.config.parent.mkdir()
        self.config.write_text(self.CONFIG, encoding="utf-8")
        for directory, lockfile, manifest in (
            ("ccr-ui", "bun.lock", {"packageManager": "bun@1.4.0"}),
            ("docs", "bun.lock", {"packageManager": "bun@1.4.0"}),
            ("ccr-vscode", "package-lock.json", {}),
        ):
            package = self.root / directory
            package.mkdir()
            (package / lockfile).write_text("{}\n", encoding="utf-8")
            (package / "package.json").write_text(json.dumps(manifest), encoding="utf-8")
        (self.root / "Cargo.lock").write_text("# fixture\n", encoding="utf-8")
        tauri = self.root / "ccr-ui" / "src-tauri"
        tauri.mkdir()
        (tauri / "Cargo.lock").write_text("# fixture\n", encoding="utf-8")

    def test_valid_mapping_allows_actions_and_cargo_at_root(self) -> None:
        self.assertEqual(dependabot_failures(self.root), [])
        self.assertEqual(dependabot_failures(REPO_ROOT), [])

    def test_quotes_comments_field_order_and_nested_schedule_are_supported(self) -> None:
        config = self.CONFIG.replace(
            "  - package-ecosystem: bun\n    directory: /docs\n",
            '  - directory: "/docs" # documentation\n'
            "    package-ecosystem: 'bun'\n"
            "    schedule:\n      interval: weekly\n    labels: [dependencies, docs]\n",
        )
        self.config.write_text(config, encoding="utf-8")
        self.assertIn(
            {"directory": "/docs", "package-ecosystem": "bun"},
            dependabot_update_fields(config),
        )
        self.assertEqual(dependabot_failures(self.root), [])

    def test_wrong_or_missing_javascript_mapping_fails(self) -> None:
        for directory, ecosystem, wrong in (
            ("/ccr-ui", "bun", "npm"),
            ("/docs", "bun", "npm"),
            ("/ccr-vscode", "npm", "bun"),
        ):
            entry = f"  - package-ecosystem: {ecosystem}\n    directory: {directory}\n"
            for replacement in ("", entry.replace(ecosystem, wrong, 1)):
                with self.subTest(directory=directory, replacement=replacement):
                    self.config.write_text(self.CONFIG.replace(entry, replacement), encoding="utf-8")
                    self.assertIn(
                        f"Dependabot {directory}: expected exactly one {ecosystem} mapping",
                        dependabot_failures(self.root),
                    )

    def test_duplicate_directory_fails_for_same_and_different_ecosystems(self) -> None:
        for ecosystem in ("bun", "npm"):
            with self.subTest(ecosystem=ecosystem):
                self.config.write_text(
                    self.CONFIG + f"  - package-ecosystem: {ecosystem}\n    directory: /docs\n",
                    encoding="utf-8",
                )
                failures = dependabot_failures(self.root)
                self.assertIn("Dependabot /docs: expected exactly one bun mapping", failures)
                if ecosystem == "bun":
                    self.assertIn("duplicate Dependabot mapping: bun /docs", failures)

    def test_missing_file_and_update_fields_fail(self) -> None:
        entry = "  - package-ecosystem: bun\n    directory: /docs\n"
        for incomplete in ("  - package-ecosystem: bun\n", "  - directory: /docs\n"):
            self.config.write_text(self.CONFIG.replace(entry, incomplete), encoding="utf-8")
            self.assertTrue(any(
                "missing ecosystem or directory" in failure
                for failure in dependabot_failures(self.root)
            ))
        self.config.unlink()
        self.assertTrue(any("dependabot.yml" in failure for failure in dependabot_failures(self.root)))

    def test_missing_actions_and_cargo_mappings_fail(self) -> None:
        for ecosystem, directory in (
            ("github-actions", "/"),
            ("cargo", "/"),
            ("cargo", "/ccr-ui/src-tauri"),
        ):
            with self.subTest(ecosystem=ecosystem, directory=directory):
                entry = f"  - package-ecosystem: {ecosystem}\n    directory: {directory}\n"
                self.config.write_text(self.CONFIG.replace(entry, ""), encoding="utf-8")
                self.assertIn(
                    f"missing Dependabot mapping: {ecosystem} {directory}",
                    dependabot_failures(self.root),
                )

    def test_duplicate_yaml_field_fails(self) -> None:
        self.config.write_text(
            self.CONFIG.replace("    directory: /docs\n", "    directory: /docs\n    directory: /docs\n"),
            encoding="utf-8",
        )
        self.assertTrue(any("duplicate YAML mapping key 'directory'" in failure for failure in dependabot_failures(self.root)))

    def test_missing_authoritative_locks_fail(self) -> None:
        for relative in ("ccr-ui/bun.lock", "docs/bun.lock", "ccr-vscode/package-lock.json", "Cargo.lock", "ccr-ui/src-tauri/Cargo.lock"):
            with self.subTest(lockfile=relative):
                lockfile = self.root / relative
                original = lockfile.read_bytes()
                lockfile.unlink()
                self.assertTrue(any(f"{lockfile.name} is missing" in failure for failure in dependabot_failures(self.root)))
                lockfile.write_bytes(original)

    def test_competing_javascript_locks_fail(self) -> None:
        for relative in ("ccr-ui/package-lock.json", "docs/package-lock.json", "ccr-vscode/bun.lock"):
            with self.subTest(lockfile=relative):
                lockfile = self.root / relative
                lockfile.write_text("{}", encoding="utf-8")
                self.assertTrue(any("conflicting lockfile" in failure for failure in dependabot_failures(self.root)))
                lockfile.unlink()

    def test_package_manager_mismatch_and_pin_drift_fail(self) -> None:
        for directory, manager in (("ccr-ui", "npm@11.0.0"), ("docs", "npm@11.0.0"), ("docs", "bun@1.3.0"), ("ccr-vscode", "bun@1.4.0")):
            with self.subTest(directory=directory, manager=manager):
                manifest = self.root / directory / "package.json"
                original = manifest.read_bytes()
                manifest.write_text(json.dumps({"packageManager": manager}), encoding="utf-8")
                self.assertTrue(any("packageManager" in failure for failure in dependabot_failures(self.root)))
                manifest.write_bytes(original)

    def test_invalid_manifests_fail_without_crashing(self) -> None:
        for directory in ("ccr-ui", "docs", "ccr-vscode"):
            for invalid in ("[", "[]"):
                with self.subTest(directory=directory, invalid=invalid):
                    manifest = self.root / directory / "package.json"
                    original = manifest.read_bytes()
                    manifest.write_text(invalid, encoding="utf-8")
                    self.assertTrue(dependabot_failures(self.root))
                    manifest.write_bytes(original)


class ReadonlyAggregateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = (REPO_ROOT / "justfile").read_text(encoding="utf-8")
        cls.extension = (REPO_ROOT / "ccr-vscode/justfile").read_text(encoding="utf-8")
        cls.scripts = json.loads((REPO_ROOT / "ccr-vscode/package.json").read_text(encoding="utf-8"))["scripts"]
        cls.workflows = {
            name: (REPO_ROOT / ".github/workflows" / name).read_text(encoding="utf-8")
            for name in ("ci.yml", "frontend-ci.yml", "vscode-ci.yml")
        }

    def run_body(self, platform: str, recipe: str, stub: str) -> subprocess.CompletedProcess[str]:
        shell = shutil.which("pwsh" if platform == "windows" else "bash")
        if shell is None:
            self.skipTest(f"{platform} recipe probe requires its shell, not the product toolchain")
        block = recipe_block(self.source, recipe)
        self.assertTrue(block, recipe)
        body = textwrap.dedent("\n".join(block.splitlines()[2:]))
        body = body.replace("{{CARGO_AUDIT_VERSION}}", "0.22.2").replace("{{RUST_TOOLCHAIN}}", DEVELOPMENT_RUST_TOOLCHAIN)
        script = stub + "\n" + body
        # stdin avoids a second expansion through Windows' legacy WSL bash.exe -c.
        args = ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script] if platform == "windows" else ["--noprofile", "--norc", "-s"]
        result = subprocess.run(
            [shell, *args], input=None if platform == "windows" else script.encode("utf-8"), cwd=REPO_ROOT,
            capture_output=True, timeout=30,
        )
        return subprocess.CompletedProcess(
            result.args, result.returncode,
            result.stdout.decode("utf-8", errors="replace"),
            result.stderr.decode("utf-8", errors="replace"),
        )

    def check_platform_execution(self, platform: str) -> None:
        # Execute each real timing body. Only its leaf command is a stub.
        for failed in (None, EXPECTED_CI_STEPS[0], "harness-check", "audit", "tauri-ci", "frontend-audit", EXPECTED_CI_STEPS[-1]):
            with self.subTest(platform=platform, failure=failed):
                if platform == "windows":
                    stub = f"""
function chcp {{}}
function just {{
    param([string]$Name)
    Write-Output "__CCR_STEP__$Name"
    $global:LASTEXITCODE = if ($Name -eq '{failed or ''}') {{ 37 }} else {{ 0 }}
}}
"""
                else:
                    stub = f"""
just() {{
    printf '__CCR_STEP__%s\\n' "$1"
    if [ "$1" = '{failed or ''}' ]; then return 37; fi
    return 0
}}
"""
                result = self.run_body(platform, f"_ci-timed-{platform}", stub)
                self.assertEqual(result.returncode, 37 if failed else 0, result.stdout + result.stderr)
                calls = re.findall(r"__CCR_STEP__([a-z-]+)", result.stdout)
                count = EXPECTED_CI_STEPS.index(failed) + 1 if failed else len(EXPECTED_CI_STEPS)
                self.assertEqual(calls, list(EXPECTED_CI_STEPS[:count]))
                self.assertFalse({"version-sync", "fmt"}.intersection(calls))

    def test_windows_aggregate_order_and_first_failure(self) -> None:
        self.check_platform_execution("windows")

    def test_linux_aggregate_order_and_first_failure(self) -> None:
        self.check_platform_execution("linux")

    def test_macos_aggregate_order_and_first_failure(self) -> None:
        self.check_platform_execution("macos")

    def test_audit_prerequisite_never_installs_tools(self) -> None:
        for platform in ("windows", "linux", "macos"):
            for installed in (False, True):
                with self.subTest(platform=platform, installed=installed):
                    if platform == "windows":
                        stub = (
                            "function Get-Command { " + ("return 'fixture'" if installed else "return $null") + " }\n"
                            "function cargo { Write-Output 'UNEXPECTED_INSTALL'; exit 79 }"
                        )
                    else:
                        stub = (
                            f"command() {{ return {0 if installed else 1}; }}\n"
                            "cargo() { printf 'UNEXPECTED_INSTALL'; exit 79; }"
                        )
                    result = self.run_body(platform, f"_ensure-cargo-audit-{platform}", stub)
                    self.assertEqual(result.returncode, 0 if installed else 1, result.stdout + result.stderr)
                    self.assertNotIn("UNEXPECTED_INSTALL", result.stdout + result.stderr)
                    if not installed:
                        self.assertIn("cargo-audit", result.stdout + result.stderr)
                        self.assertIn("0.22.2", result.stdout + result.stderr)
                        self.assertIn("--locked", result.stdout + result.stderr)

    def test_every_platform_rejects_omission_reorder_and_repair(self) -> None:
        self.assertEqual(local_aggregate_failures(self.source), [])
        for platform in ("windows", "linux", "macos"):
            block = recipe_block(self.source, f"_ci-timed-{platform}")
            for step in EXPECTED_CI_STEPS:
                with self.subTest(platform=platform, missing=step):
                    broken = self.source.replace(block, block.replace(f'"{step}"', '"omitted"', 1))
                    self.assertEqual(len(local_aggregate_failures(broken)), 1)
            for check, repair in (("fmt-check", "fmt"), ("version-check", "version-sync")):
                broken = self.source.replace(block, block.replace(f'"{check}"', f'"{repair}"', 1))
                self.assertTrue(local_aggregate_failures(broken))
            swapped = block.replace('"omp-check"', '"SWAP"').replace('"harness-check"', '"omp-check"').replace('"SWAP"', '"harness-check"')
            self.assertTrue(local_aggregate_failures(self.source.replace(block, swapped)))

    def test_leaf_check_commands_cannot_be_omitted(self) -> None:
        self.assertEqual(local_gate_failures(self.source, self.extension, self.scripts), [])
        commands = (
            "bun test scripts/trellis/omp-context.test.ts",
            "-m unittest scripts.quality.test_check_harness_contracts",
            "python scripts/quality/check_harness_contracts.py",
            "python3 scripts/quality/check_harness_contracts.py",
            "node --test scripts/quality/check-copilot-assets.test.mjs",
            "node scripts/quality/check-copilot-assets.mjs",
            "cargo audit --file Cargo.lock",
            "cargo audit --file ccr-ui/src-tauri/Cargo.lock",
            "just _ensure-cargo-audit-{{os()}}",
            "bun run audit:dependencies",
            "[env('CCR_SKIP_ICON_GENERATION', '1')]",
            "just _vscode-run ci",
        )
        for command in commands:
            with self.subTest(omitted=command):
                self.assertIn(command, self.source)
                broken = self.source.replace(command, "OMITTED")
                self.assertTrue(local_gate_failures(broken, self.extension, self.scripts))
        for platform in ("windows", "linux", "macos"):
            block = recipe_block(self.source, f"_ensure-cargo-audit-{platform}")
            broken = self.source.replace(block, block.replace("exit 1", "cargo install cargo-audit\n        exit 1"))
            self.assertTrue(local_gate_failures(broken, self.extension, self.scripts))

    def test_final_vsix_chain_cannot_be_shortened(self) -> None:
        for original, replacement in (("ci: install lint test build", "ci: install lint test"), ("npm run package", "npm run build")):
            self.assertTrue(local_gate_failures(self.source, self.extension.replace(original, replacement), self.scripts))
        for key in ("package", "check:vsix"):
            scripts = {**self.scripts, key: "echo omitted"}
            self.assertTrue(local_gate_failures(self.source, self.extension, scripts))
        scripts = {**self.scripts, "package": "npm run check:vsix && npm run build:package && vsce package --no-dependencies -o ccr-vscode.vsix"}
        self.assertTrue(local_gate_failures(self.source, self.extension, scripts))

    def test_hosted_checks_and_runtime_prerequisites_cannot_be_omitted(self) -> None:
        self.assertEqual(hosted_gate_failures(self.workflows, "1.4.0"), [])
        required = {
            "ci.yml": ("bun-version: 1.4.0", "node-version: 24.20.0", "run: just omp-check", "run: just harness-check", "run: just copilot-check", "cargo audit --file Cargo.lock", "cargo audit --file ccr-ui/src-tauri/Cargo.lock"),
            "frontend-ci.yml": ("run: just frontend-check", "run: just frontend-coverage", "run: just frontend-audit", "CCR_SKIP_ICON_GENERATION: '1'"),
            "vscode-ci.yml": ("run: just vscode-ci",),
        }
        for name, commands in required.items():
            for command in commands:
                with self.subTest(workflow=name, omitted=command):
                    self.assertIn(command, self.workflows[name])
                    broken = {**self.workflows, name: self.workflows[name].replace(command, "OMITTED")}
                    self.assertTrue(hosted_gate_failures(broken, "1.4.0"))

    def test_new_checker_inputs_are_in_the_relevance_policy(self) -> None:
        self.assertEqual(check_relevance_failures(SURFACE_PATHS), [])
        for surface in SURFACE_PATHS:
            with self.subTest(surface=surface):
                self.assertTrue(check_relevance_failures({**SURFACE_PATHS, surface: ()}))

    def test_frontend_build_sets_icon_skip_for_both_bun_commands(self) -> None:
        just = shutil.which("just")
        self.assertIsNotNone(just, "just is required to verify recipe environment propagation")
        with tempfile.TemporaryDirectory(prefix="ccr-icon-env-") as temporary:
            root = Path(temporary)
            (root / "ccr-ui").mkdir()
            (root / "fixture.py").write_text(
                "import os, sys\nassert os.environ.get('CCR_SKIP_ICON_GENERATION') == '1'\nprint('__BUN__' + ' '.join(sys.argv[1:]))\n",
                encoding="utf-8",
            )
            if os.name == "nt":
                (root / "bun.cmd").write_text(f'@"{sys.executable}" "%~dp0fixture.py" %*\n', encoding="utf-8")
            else:
                shim = root / "bun"
                shim.write_text(f'#!/bin/sh\nexec "{sys.executable}" "{root / "fixture.py"}" "$@"\n', encoding="utf-8")
                shim.chmod(0o755)
            settings = re.findall(r"^set (?:windows-shell|shell) := .*", self.source, re.MULTILINE)
            block = recipe_block(self.source, "frontend-build")
            attributes = re.search(r"^(\[env[^\n]+\]\n)frontend-build:", self.source, re.MULTILINE)
            self.assertIsNotNone(attributes)
            (root / "justfile").write_text(
                "\n".join(settings) + "\n" + attributes.group(1) + block + "\nheader message:\n    @echo header\nsuccess message:\n    @echo success\n",
                encoding="utf-8",
            )
            env = {**os.environ, "PATH": str(root) + os.pathsep + os.environ["PATH"], "CCR_SKIP_ICON_GENERATION": "0"}
            result = subprocess.run([just, "frontend-build"], cwd=root, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=30)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("__BUN__install --frozen-lockfile", result.stdout)
            self.assertIn("__BUN__run build", result.stdout)


class WorkflowGovernanceParserTests(unittest.TestCase):
    ROOT = REPO_ROOT

    def test_duplicate_mapping_key_is_rejected(self) -> None:
        workflow = """jobs:
  build:
    steps:
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@0123456789012345678901234567890123456789
        with:
          toolchain: 1.95.0
        with:
          components: clippy
"""

        self.assertEqual(duplicate_mapping_keys(workflow), [(8, "with")])

    def test_sequence_items_and_block_scalars_do_not_look_duplicate(self) -> None:
        workflow = """jobs:
  build:
    steps:
      - name: First
        run: |
          echo "name: inside script"
      - name: Second
        run: echo done
"""

        self.assertEqual(duplicate_mapping_keys(workflow), [])

    def test_trigger_values_are_read_from_the_requested_event(self) -> None:
        workflow = """on:
  push:
    branches: [main, develop, dev]
    paths:
      - 'ccr-ui/**'
  pull_request:
    branches: [main, develop, dev]
    paths:
      - 'ccr-ui/**'
      - 'justfile'
"""

        self.assertEqual(
            workflow_event_values(workflow, "pull_request", "branches"),
            {"main", "develop", "dev"},
        )
        self.assertEqual(
            workflow_event_values(workflow, "pull_request", "paths"),
            {"ccr-ui/**", "justfile"},
        )
        self.assertEqual(
            workflow_event_values(workflow, "push", "paths"), {"ccr-ui/**"}
        )

    def test_missing_event_is_reported_as_empty(self) -> None:
        workflow = """on:
  pull_request:
    branches: [main, develop, dev]
"""

        self.assertEqual(workflow_event_values(workflow, "push", "branches"), set())

    def test_surface_paths_match_nested_and_exact_files(self) -> None:
        self.assertTrue(path_matches("crates/ccr/src/main.rs", SURFACE_PATHS["root"]))
        self.assertTrue(path_matches("justfile", SURFACE_PATHS["vscode"]))
        self.assertTrue(
            path_matches(
                "ccr-ui/src-tauri/src/main.rs", SURFACE_PATHS["tauri"]
            )
        )
        self.assertFalse(path_matches("README.md", SURFACE_PATHS["frontend"]))

    def test_relevance_is_true_when_any_changed_path_matches(self) -> None:
        self.assertTrue(
            is_relevant("vscode", ["README.md", "ccr-vscode/src/extension.ts"])
        )
        self.assertFalse(is_relevant("vscode", ["README.md", "docs/index.md"]))

    def test_policy_script_changes_validate_every_surface(self) -> None:
        for surface in SURFACE_PATHS:
            with self.subTest(surface=surface):
                self.assertTrue(
                    is_relevant(surface, ["scripts/ci/ci_surface_policy.py"])
                )

    def test_cargo_config_inputs_trigger_only_consumer_surfaces(self) -> None:
        expected = {
            ".cargo/tauri-ci.toml": {"tauri"},
            ".cargo/config.toml": {"root", "tauri"},
            ".cargo/audit.toml": {"root"},
        }
        for path, surfaces in expected.items():
            for surface in SURFACE_PATHS:
                with self.subTest(path=path, surface=surface):
                    self.assertEqual(
                        is_relevant(surface, [path]),
                        surface in surfaces,
                    )
        for patterns in SURFACE_PATHS.values():
            self.assertNotIn(".cargo/**", patterns)

    def test_vscode_coverage_step_uses_bash_pipefail(self) -> None:
        workflow = (
            self.ROOT / ".github" / "workflows" / "vscode-ci.yml"
        ).read_text(encoding="utf-8")
        step = _workflow_step_fields(workflow, "just vscode-coverage | tee")
        self.assertEqual(
            step.get("run"),
            "just vscode-coverage | tee vscode-coverage.txt",
        )
        self.assertEqual(step.get("shell"), "bash")

        bash = shutil.which("bash")
        if bash is None:
            self.skipTest(
                "bash is not available to probe GitHub pipefail semantics"
            )

        failed = subprocess.run(
            [
                bash,
                "--noprofile",
                "--norc",
                "-eo",
                "pipefail",
                "-c",
                "false | tee /dev/null",
            ],
            capture_output=True,
            check=False,
        )
        self.assertNotEqual(failed.returncode, 0)

        succeeded = subprocess.run(
            [
                bash,
                "--noprofile",
                "--norc",
                "-eo",
                "pipefail",
                "-c",
                "true | tee /dev/null",
            ],
            capture_output=True,
            check=False,
        )
        self.assertEqual(succeeded.returncode, 0)

    def test_job_block_stops_before_the_next_job(self) -> None:
        workflow = """jobs:
  validation:
    name: Validation
  required:
    name: Required
    if: ${{ always() }}
"""

        self.assertEqual(
            workflow_job_block(workflow, "required"),
            "  required:\n    name: Required\n    if: ${{ always() }}",
        )

    def test_rust_toolchain_inputs_are_extracted_from_action_inputs(self) -> None:
        workflow = """steps:
  - uses: dtolnay/rust-toolchain@0123456789012345678901234567890123456789
    with:
      toolchain: 1.98.0
"""

        self.assertEqual(rust_toolchain_inputs(workflow), ["1.98.0"])

    def test_bun_version_inputs_are_extracted_from_action_inputs(self) -> None:
        workflow = """steps:
  - uses: oven-sh/setup-bun@0123456789012345678901234567890123456789
    with:
      bun-version: 1.4.0
"""

        self.assertEqual(bun_version_inputs(workflow), ["1.4.0"])

    def test_node_version_inputs_are_extracted_from_action_inputs(self) -> None:
        workflow = """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
"""

        self.assertEqual(node_version_inputs(workflow), ["24.20.0"])
        self.assertEqual(setup_node_version_inputs(workflow), [["24.20.0"]])

    def test_node_toolchain_validator_fails_closed_on_drift_and_extra_inputs(
        self,
    ) -> None:
        workflows = {
            "release.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.19.0
""",
            "vscode-ci.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
""",
            "unexpected.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 26.8.1
""",
        }

        failures = node_pin_failures(workflows)

        self.assertTrue(
            any("release.yml: expected 2 Node 24.20.0" in item for item in failures)
        )
        self.assertTrue(
            any("unexpected.yml: unexpected Node setup input" in item for item in failures)
        )

    def test_unrelated_node_version_cannot_mask_an_unpinned_setup_step(self) -> None:
        workflows = {
            "release.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
  - uses: actions/setup-node@0123456789012345678901234567890123456789
  - uses: example/action@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
""",
            "vscode-ci.yml": """steps:
  - uses: actions/setup-node@0123456789012345678901234567890123456789
    with:
      node-version: 24.20.0
""",
        }

        failures = node_pin_failures(workflows)

        self.assertTrue(any("release.yml: expected 2" in item for item in failures))

    def test_bun_toolchain_pin_has_one_canonical_source(self) -> None:
        self.assertEqual(canonical_bun_version(self.ROOT), "1.4.0")
        docs_package = json.loads(
            (self.ROOT / "docs" / "package.json").read_text(encoding="utf-8")
        )
        self.assertEqual(docs_package.get("packageManager"), "bun@1.4.0")

        for name in EXPECTED_BUN_WORKFLOWS:
            workflow = (self.ROOT / ".github" / "workflows" / name).read_text(
                encoding="utf-8"
            )
            with self.subTest(workflow=name):
                self.assertEqual(bun_version_inputs(workflow), ["1.4.0"])

    def test_node_toolchain_pin_is_exact_and_scoped(self) -> None:
        workflow_dir = self.ROOT / ".github" / "workflows"
        workflows = {
            path.name: path.read_text(encoding="utf-8")
            for path in workflow_dir.iterdir()
            if path.suffix in {".yml", ".yaml"}
        }

        for name, expected_count in EXPECTED_NODE_WORKFLOW_INPUTS.items():
            with self.subTest(workflow=name):
                self.assertEqual(
                    node_version_inputs(workflows[name]),
                    [NODE_TOOLCHAIN] * expected_count,
                )
        for name, workflow in workflows.items():
            if name in EXPECTED_NODE_WORKFLOW_INPUTS:
                continue
            with self.subTest(workflow=name):
                self.assertEqual(node_version_inputs(workflow), [])

    def test_workflows_split_development_toolchain_from_msrv(self) -> None:
        workflows = {
            name: (self.ROOT / ".github" / "workflows" / name).read_text(
                encoding="utf-8"
            )
            for name in (
                "ci.yml",
                "frontend-ci.yml",
                "release.yml",
                "tauri-rust-ci.yml",
                "vscode-ci.yml",
            )
        }
        msrv_job = workflow_job_block(workflows["ci.yml"], "workspace-msrv")

        self.assertEqual(rust_toolchain_inputs(msrv_job), [MSRV_RUST_TOOLCHAIN])
        self.assertIn(
            "cargo check --workspace --all-targets --all-features", msrv_job
        )
        for name, workflow in workflows.items():
            ordinary_workflow = (
                workflow.replace(msrv_job, "", 1) if name == "ci.yml" else workflow
            )
            with self.subTest(workflow=name):
                self.assertTrue(rust_toolchain_inputs(ordinary_workflow))
                self.assertEqual(
                    set(rust_toolchain_inputs(ordinary_workflow)),
                    {DEVELOPMENT_RUST_TOOLCHAIN},
                )

        root_required = workflow_job_block(workflows["ci.yml"], "root-required")
        self.assertIn("workspace-msrv", root_required)
        self.assertIn("MSRV:", root_required)

    def test_tauri_rust_gates_use_a_tracked_frontend_fixture(self) -> None:
        cargo_config = (self.ROOT / ".cargo" / "tauri-ci.toml").read_text(
            encoding="utf-8"
        )
        root_justfile = (self.ROOT / "justfile").read_text(encoding="utf-8")
        ui_justfile = (self.ROOT / "ccr-ui" / "justfile").read_text(
            encoding="utf-8"
        )

        self.assertIn('frontendDist":"ci-dist"', cargo_config)
        self.assertTrue(
            (self.ROOT / "ccr-ui" / "src-tauri" / "ci-dist" / "index.html").is_file()
        )
        self.assertIn("cargo --config .cargo/tauri-ci.toml test", root_justfile)
        self.assertIn(
            "cargo --config .cargo/tauri-ci.toml llvm-cov", root_justfile
        )
        self.assertIn("scripts/generate-bindings.mjs", ui_justfile)
        generator = (self.ROOT / "ccr-ui" / "scripts" / "generate-bindings.mjs").read_text(encoding="utf-8")
        self.assertIn("'--config', '../.cargo/tauri-ci.toml', 'test'", generator)

    def test_root_fmt_repairs_json_before_fmt_check(self) -> None:
        root_justfile = (self.ROOT / "justfile").read_text(encoding="utf-8")

        self.assertIn("fmt: json-format", root_justfile)
        self.assertIn("fmt-check: json-format-check", root_justfile)

    def test_tauri_bindings_check_compares_against_the_worktree_baseline(self) -> None:
        ui_justfile = (self.ROOT / "ccr-ui" / "justfile").read_text(
            encoding="utf-8"
        )

        self.assertIn("scripts/check-generated-bindings.mjs", ui_justfile)
        self.assertIn("scripts/generate-bindings.mjs", ui_justfile)
        generator = (self.ROOT / "ccr-ui" / "scripts" / "generate-bindings.mjs").read_text(encoding="utf-8")
        self.assertIn("./scripts/normalize-generated-bindings.mjs", generator)
        self.assertIn("withGeneratedDirectory", generator)
        self.assertNotIn("git status --porcelain -- src/types/generated", ui_justfile)
        self.assertTrue(
            (self.ROOT / "ccr-ui" / "scripts" / "check-generated-bindings.mjs").is_file()
        )

    def test_quality_workflows_are_pull_request_only(self) -> None:
        for name in ("ci.yml", "frontend-ci.yml", "tauri-rust-ci.yml", "vscode-ci.yml"):
            workflow = (self.ROOT / ".github" / "workflows" / name).read_text(
                encoding="utf-8"
            )
            self.assertEqual(
                workflow_event_values(workflow, "push", "branches"),
                set(),
                msg=f"{name} must not trigger on branch push",
            )
            self.assertEqual(
                workflow_event_values(workflow, "pull_request", "branches"),
                {"main", "develop", "dev"},
            )

    def test_react_smoke_coverage_threshold_is_seventy_percent(self) -> None:
        smoke_config = (self.ROOT / "ccr-ui" / "vitest.smoke.config.ts").read_text(
            encoding="utf-8"
        )

        self.assertIn("thresholds", smoke_config)
        self.assertIn("lines: 70", smoke_config)

    def test_tauri_linux_gate_installs_pinned_bun_for_bindings(self) -> None:
        workflow = (
            self.ROOT / ".github" / "workflows" / "tauri-rust-ci.yml"
        ).read_text(encoding="utf-8")
        linux_job = workflow_job_block(workflow, "tauri-linux-required")

        self.assertIn(
            "oven-sh/setup-bun@0c5077e51419868618aeaa5fe8019c62421857d6",
            linux_job,
        )
        self.assertEqual(bun_version_inputs(linux_job), ["1.4.0"])
        self.assertIn("run: just tauri-ci", linux_job)


if __name__ == "__main__":
    unittest.main()
