from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts.quality.check_harness_contracts import REPO_ROOT, check_contracts


SHARED_FILES = (
    "AGENTS.md",
    "CLAUDE.md",
    "docs/agents/harnesses.md",
    "docs/en/agents/harnesses.md",
    ".codex/skills/ccr-gate-recovery/SKILL.md",
    ".trellis/workflow.md",
)
PAGES = SHARED_FILES[2:4]
SKILL = SHARED_FILES[4]
WORKSPACE_TEST = "cargo test --workspace --all-features -- --skip export_bindings"


class HarnessContractTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="ccr-harness-contract-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.originals = {}
        for path in SHARED_FILES:
            destination = self.root / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPO_ROOT / path, destination)
            self.originals[path] = destination.read_text(encoding="utf-8-sig")

    def replace(self, path: str, old: str, new: str) -> None:
        original = self.originals[path]
        self.assertIn(old, original)
        (self.root / path).write_text(original.replace(old, new), encoding="utf-8")

    def assert_failure(self, code: str, path: str) -> None:
        self.assertTrue(
            any(failure.startswith(f"{code} {path}:") for failure in check_contracts(self.root)),
            f"Expected {code} for {path}",
        )

    def row(self, path: str, label: str) -> str:
        return next(line for line in self.originals[path].splitlines() if line.startswith(f"| {label} |"))

    def test_current_docs_and_personal_free_fixture_pass(self) -> None:
        self.assertEqual(check_contracts(REPO_ROOT), [])
        self.assertEqual(check_contracts(self.root), [])
        self.assertFalse((self.root / ".git").exists())
        for directory in (".claude", ".grok", ".kimi-code", ".omp"):
            self.assertFalse((self.root / directory).exists())
        for path in (".codex/agents", ".codex/hooks.json", ".codex/config.toml"):
            self.assertFalse((self.root / path).exists())

    def test_each_missing_shared_file_fails(self) -> None:
        for path, original in self.originals.items():
            with self.subTest(path=path):
                (self.root / path).unlink()
                self.assert_failure("HC001", path)
                (self.root / path).write_text(original, encoding="utf-8")

    def test_empty_or_invalid_utf8_shared_file_fails(self) -> None:
        for content in (b"", b"\xff"):
            with self.subTest(content=content):
                (self.root / "AGENTS.md").write_bytes(content)
                self.assert_failure("HC001", "AGENTS.md")

    def test_import_must_be_real_and_target_agents(self) -> None:
        invalid = (
            "\x60@AGENTS.md\x60",
            "\x60\x60\x60\n@AGENTS.md\n\x60\x60\x60",
            "~~~markdown\n@AGENTS.md\n~~~",
            "<!--\n@AGENTS.md\n-->",
            "    @AGENTS.md",
            "> @AGENTS.md",
            "@MISSING.md",
        )
        for replacement in invalid:
            with self.subTest(replacement=replacement):
                self.replace("CLAUDE.md", "\n@AGENTS.md\n", f"\n{replacement}\n")
                self.assert_failure("HC002", "CLAUDE.md")

    def test_nested_fence_markers_do_not_expose_a_fake_import(self) -> None:
        replacement = "\x60\x60\x60\x60markdown\n\x60\x60\x60\n@AGENTS.md\n\x60\x60\x60\n\x60\x60\x60\x60"
        self.replace("CLAUDE.md", "\n@AGENTS.md\n", f"\n{replacement}\n")
        self.assert_failure("HC002", "CLAUDE.md")

    def test_bom_crlf_and_unrelated_prose_changes_are_supported(self) -> None:
        for path, original in self.originals.items():
            text = original + "\nAdditional audit context does not change the contract.\n"
            (self.root / path).write_bytes(b"\xef\xbb\xbf" + text.replace("\n", "\r\n").encode("utf-8"))
        self.assertEqual(check_contracts(self.root), [])

    def test_each_tool_requires_one_row_in_each_translation(self) -> None:
        for path in PAGES:
            for tool in ("Claude Code", "Codex", "Grok Build", "Kimi Code", "OMP"):
                for replacement in ("", self.row(path, tool) + "\n" + self.row(path, tool)):
                    with self.subTest(path=path, tool=tool, duplicate=bool(replacement)):
                        self.replace(path, self.row(path, tool), replacement)
                        self.assert_failure("HC003", path)
            (self.root / path).write_text(self.originals[path], encoding="utf-8")

    def test_root_and_skill_name_all_five_harnesses(self) -> None:
        for path in ("AGENTS.md", SKILL):
            with self.subTest(path=path):
                self.replace(path, "Grok Build", "Grok")
                self.assert_failure("HC003", path)

    def test_kimi_requires_project_agents_and_pull_integration(self) -> None:
        for path in PAGES:
            row = self.row(path, "Kimi Code")
            for old, new in (
                (".kimi-code/agents/", ".kimi-code/unknown/"),
                ("trellis-implement", "built-in coder"),
                ("pull", "automatic"),
            ):
                with self.subTest(path=path, old=old):
                    self.replace(path, row, row.replace(old, new))
                    self.assert_failure("HC004", path)
            (self.root / path).write_text(self.originals[path], encoding="utf-8")

    def test_kimi_cannot_deny_agents_while_retaining_their_names(self) -> None:
        path = "docs/en/agents/harnesses.md"
        row = self.row(path, "Kimi Code")
        self.replace(path, row, row.replace("Project hooks are absent.", "No project agents. Project hooks are absent."))
        self.assert_failure("HC004", path)

    def test_kimi_fallback_requires_execution_tools(self) -> None:
        for path, old in ((PAGES[0], "执行工具"), (PAGES[1], "execution tools")):
            with self.subTest(path=path):
                row = self.row(path, "Kimi Code")
                self.replace(path, row, row.replace(old, "read-only tools"))
                self.assert_failure("HC004", path)

    def test_direct_rust_commands_reject_parallelism_overrides(self) -> None:
        for path in ("CLAUDE.md", SKILL):
            for command in (
                WORKSPACE_TEST + " --test-threads=1",
                WORKSPACE_TEST + " --test-threads 1",
                WORKSPACE_TEST + " --test-threads=8",
                "RUST_TEST_THREADS=1 " + WORKSPACE_TEST,
            ):
                with self.subTest(path=path, command=command):
                    self.replace(path, WORKSPACE_TEST, command)
                    self.assert_failure("HC005", path)

    def test_independent_thread_environment_assignments_are_rejected(self) -> None:
        for path in ("CLAUDE.md", SKILL):
            for assignment in (
                "export RUST_TEST_THREADS=1",
                "$env:RUST_TEST_THREADS = '1'",
                "$env:rust_test_threads = '1'",
                "set RUST_TEST_THREADS=1",
            ):
                with self.subTest(path=path, assignment=assignment):
                    example = "\n\x60\x60\x60powershell\n" + assignment + "\n" + WORKSPACE_TEST + "\n\x60\x60\x60\n"
                    (self.root / path).write_text(self.originals[path] + example, encoding="utf-8")
                    self.assert_failure("HC005", path)
            (self.root / path).write_text(self.originals[path], encoding="utf-8")

    def test_workspace_and_package_commands_must_skip_binding_exports(self) -> None:
        for path, command in (
            ("CLAUDE.md", WORKSPACE_TEST),
            (SKILL, WORKSPACE_TEST),
            (SKILL, "cargo test -p <crate-name> -- --skip export_bindings"),
        ):
            with self.subTest(path=path, command=command):
                self.replace(path, command, command.replace(" -- --skip export_bindings", ""))
                self.assert_failure("HC005", path)

    def test_rust_fenced_command_continuations_are_supported(self) -> None:
        for continuation in ("\\", "\x60"):
            with self.subTest(continuation=continuation):
                command = "cargo test --workspace " + continuation + "\n  --all-features -- --skip export_bindings"
                self.replace(SKILL, WORKSPACE_TEST, command)
                self.assertEqual(check_contracts(self.root), [])

    def test_default_parallelism_and_binding_owner_remain_explicit(self) -> None:
        for path, old, new in (
            ("AGENTS.md", "default parallelism", "serialized execution"),
            (PAGES[0], "默认并行", "默认串行"),
            (PAGES[1], "separate generation gate", "ordinary test gate"),
        ):
            with self.subTest(path=path, old=old):
                self.replace(path, old, new)
                self.assert_failure("HC005", path)

    def test_writable_check_cannot_move_before_approval(self) -> None:
        for path, old, new in (
            ("AGENTS.md", "after implementation approval", "before implementation approval"),
            (PAGES[0], "批准实施之后", "审批前"),
            (PAGES[1], "After approval, as an executing role", "Before approval, as an executing role"),
            (SKILL, "After approval, implement", "Before approval, implement"),
            (".trellis/workflow.md", "Task creation approval is not implementation approval", "Task creation approval grants implementation approval"),
        ):
            with self.subTest(path=path):
                self.replace(path, old, new)
                self.assert_failure("HC006", path)

    def test_reviewer_and_writable_check_permissions_cannot_be_swapped(self) -> None:
        for path, old, new in (
            (PAGES[0], "不改产品代码", "可以修改产品代码"),
            (PAGES[1], "Do not change product code", "May change product code"),
            (PAGES[0], "**可写、可自修**", "只读"),
            (PAGES[1], "**May write and self-fix**", "Read only"),
            ("AGENTS.md", "may write and self-fix", "is read-only"),
        ):
            with self.subTest(path=path, old=old):
                self.replace(path, old, new)
                self.assert_failure("HC006", path)

    def test_fresh_checkout_instructions_and_native_evidence_boundary_are_required(self) -> None:
        for path in PAGES:
            for old in ("trellis init --help", "--skip-existing", ".omp/extensions/trellis/index.ts"):
                with self.subTest(path=path, old=old):
                    self.replace(path, old, "removed-contract")
                    self.assert_failure("HC007", path)
        for path, old, new in (
            (PAGES[0], "不能证明原生", "已经证明原生"),
            (PAGES[1], "does not prove native", "proves native"),
            ("AGENTS.md", "fresh checkout", "existing workspace"),
            (SKILL, "hosted CI", "a different check"),
        ):
            with self.subTest(path=path, old=old):
                self.replace(path, old, new)
                self.assert_failure("HC007", path)

    def test_repair_commands_cannot_be_relabelled_read_only(self) -> None:
        for path, old, new in (
            ("AGENTS.md", "Keep repair commands", "Keep harmless commands"),
            ("CLAUDE.md", "repair-oriented steps that may modify files", "read-only steps"),
            (PAGES[0], "**会改文件**", "**只读检查**"),
            (PAGES[1], "**May rewrite files**", "**Read-only checks**"),
        ):
            with self.subTest(path=path):
                self.replace(path, old, new)
                self.assert_failure("HC008", path)

    def test_cli_succeeds_without_local_integrations_and_fails_on_drift(self) -> None:
        script = self.root / "scripts/quality/check_harness_contracts.py"
        script.parent.mkdir(parents=True)
        shutil.copyfile(REPO_ROOT / "scripts/quality/check_harness_contracts.py", script)
        result = subprocess.run([sys.executable, str(script)], capture_output=True, text=True, encoding="utf-8", check=False)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("native loading, trust, and account settings remain unverified", result.stdout)
        self.replace("CLAUDE.md", "\n@AGENTS.md\n", "\n\x60@AGENTS.md\x60\n")
        result = subprocess.run([sys.executable, str(script), "--root", str(self.root)], capture_output=True, text=True, encoding="utf-8", check=False)
        self.assertEqual(result.returncode, 1)
        self.assertIn("HC002 CLAUDE.md", result.stdout)


if __name__ == "__main__":
    unittest.main()
