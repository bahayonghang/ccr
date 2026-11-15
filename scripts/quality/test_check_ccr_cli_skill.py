#!/usr/bin/env python3
"""Unit tests for scripts/quality/check_ccr_cli_skill.py."""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

try:
    from scripts.common import REPO_ROOT
    from scripts.quality.check_ccr_cli_skill import (
        MAX_SKILL_UTF8_BYTES,
        SKILL_REL_DIR,
        check_repo,
        main,
        parse_simple_yaml_mapping,
    )
except ModuleNotFoundError:
    # pywin32 registers a namespace package named `scripts` in site-packages
    _scripts_dir = Path(__file__).resolve().parent
    while _scripts_dir.name != "scripts":
        _scripts_dir = _scripts_dir.parent
    sys.path.insert(0, str(_scripts_dir.parent))
    sys.modules.pop("scripts", None)
    from scripts.common import REPO_ROOT
    from scripts.quality.check_ccr_cli_skill import (
        MAX_SKILL_UTF8_BYTES,
        SKILL_REL_DIR,
        check_repo,
        main,
        parse_simple_yaml_mapping,
    )


def _copy_skill_fixture(temp_dir: str) -> Path:
    root = Path(temp_dir)
    shutil.copytree(REPO_ROOT / SKILL_REL_DIR, root / SKILL_REL_DIR)
    version_dir = root / "scripts" / "version"
    version_dir.mkdir(parents=True, exist_ok=True)
    for name in ("version-sync.ps1", "version-sync.sh"):
        src = REPO_ROOT / "scripts" / "version" / name
        if src.is_file():
            shutil.copy2(src, version_dir / name)
    return root


class CheckCcrCliSkillTests(unittest.TestCase):
    def test_live_repository_skill_package_passes(self) -> None:
        self.assertEqual(check_repo(REPO_ROOT), [])
        self.assertEqual(main(["--root", str(REPO_ROOT)]), 0)

    def test_nested_or_extra_skill_md_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-nested-") as tmp:
            root = _copy_skill_fixture(tmp)
            nested = root / SKILL_REL_DIR / "references" / "SKILL.md"
            nested.write_text("---\nname: nested\ndescription: bad\n---\n", encoding="utf-8")
            failures = check_repo(root)
            self.assertTrue(any("no nested or extra SKILL.md" in f for f in failures), failures)

    def test_skill_md_exceeding_14000_utf8_bytes_is_blocked_with_lf_normalization(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-budget-") as tmp:
            root = _copy_skill_fixture(tmp)
            skill_md = root / SKILL_REL_DIR / "SKILL.md"
            original = skill_md.read_text(encoding="utf-8")
            padding = "\n" + ("Paragraph filler line outside list items.\n" * 150)
            bloated = original + padding
            self.assertGreater(len(bloated.encode("utf-8")), MAX_SKILL_UTF8_BYTES)
            skill_md.write_text(bloated, encoding="utf-8")
            failures = check_repo(root)
            self.assertTrue(any("UTF-8 size" in f for f in failures), failures)

    def test_missing_or_fenced_h2_heading_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-h2-") as tmp:
            root = _copy_skill_fixture(tmp)
            skill_md = root / SKILL_REL_DIR / "SKILL.md"
            fenced = skill_md.read_text(encoding="utf-8").replace(
                "## Gate Ladder", "```markdown\n## Gate Ladder\n```\n## Safety Notes"
            )
            skill_md.write_text(fenced, encoding="utf-8")
            failures = check_repo(root)
            self.assertTrue(
                any("missing required H2 heading '## Gate Ladder'" in f for f in failures),
                failures,
            )

    def test_yaml_block_scalar_pronoun_duplicate_key_and_tabs_are_caught(self) -> None:
        doc = parse_simple_yaml_mapping(
            "\ufeffname: ccr-cli # inline comment\ndescription: >-\n  First line\n  second line.\n"
        )
        self.assertEqual(doc["name"], "ccr-cli")
        self.assertEqual(doc["description"], "First line second line.")

        with self.assertRaisesRegex(ValueError, "duplicate YAML mapping key"):
            parse_simple_yaml_mapping("name: ccr-cli\nname: duplicate\n")
        with self.assertRaisesRegex(ValueError, "tab indentation"):
            parse_simple_yaml_mapping("metadata:\n\towner: bahayonghang\n")

        with tempfile.TemporaryDirectory(prefix="ccr-skill-block-fm-") as tmp:
            root = _copy_skill_fixture(tmp)
            skill_md = root / SKILL_REL_DIR / "SKILL.md"
            text = skill_md.read_text(encoding="utf-8")
            _, rest = text.split("\n---\n", 1)
            block_fm = (
                "---\n"
                "name: ccr-cli\n"
                "description: |\n"
                "  Helps you manage your ccr profiles across claude, codex, and grok.\n"
                "license: MIT\n"
                "---\n" + rest
            )
            skill_md.write_text(block_fm, encoding="utf-8")
            failures = check_repo(root)
            self.assertTrue(any("pronouns" in f for f in failures), failures)

    def test_missing_readme_anchor_few_troubleshooting_items_or_duplicate_license_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-readme-") as tmp:
            root = _copy_skill_fixture(tmp)
            readme = root / SKILL_REL_DIR / "README.md"
            text = readme.read_text(encoding="utf-8").replace("你可以直接这样说", "示例提问")
            before_ts, _ = text.split("## Troubleshooting", 1)
            readme.write_text(
                before_ts + "## Troubleshooting\n\n1. Only one item.\n2. Only two items.\n",
                encoding="utf-8",
            )
            (root / SKILL_REL_DIR / "LICENSE").write_text("MIT\n", encoding="utf-8")
            failures = check_repo(root)
            self.assertTrue(any("你可以直接这样说" in f for f in failures), failures)
            self.assertTrue(any("Troubleshooting must contain at least 3 items" in f for f in failures), failures)
            self.assertTrue(any("duplicate LICENSE" in f for f in failures), failures)

    def test_backslash_in_skill_ir_resources_or_description_drift_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-ir-") as tmp:
            root = _copy_skill_fixture(tmp)
            ir_path = root / SKILL_REL_DIR / "reports" / "skill-ir.json"
            ir = json.loads(ir_path.read_text(encoding="utf-8"))
            ir["resources"]["references"] = ["references\\platforms-and-fields.md"]
            ir["intent"]["description"] = "Drifted description that no longer matches SKILL.md."
            ir_path.write_text(json.dumps(ir, indent=2) + "\n", encoding="utf-8")
            failures = check_repo(root)
            self.assertTrue(any("must use POSIX '/' paths" in f for f in failures), failures)
            self.assertTrue(any("intent.description is out of sync" in f for f in failures), failures)

    def test_backslash_in_markdown_link_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-mdlink-") as tmp:
            root = _copy_skill_fixture(tmp)
            readme = root / SKILL_REL_DIR / "README.md"
            readme.write_text(
                readme.read_text(encoding="utf-8")
                + "\nBroken slash: [diag](references\\diagnostics-and-recovery.md)\n",
                encoding="utf-8",
            )
            failures = check_repo(root)
            self.assertTrue(any("must use POSIX forward slashes" in f for f in failures), failures)

    def test_version_drift_across_manifest_ir_and_handoff_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-ver-") as tmp:
            root = _copy_skill_fixture(tmp)
            manifest_path = root / SKILL_REL_DIR / "manifest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["version"] = "1.2.0"
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
            failures = check_repo(root)
            self.assertTrue(
                any("reports/skill-ir.json" in f and "version mismatch" in f for f in failures),
                failures,
            )
            self.assertTrue(
                any("reports/creation-handoff.md" in f and "version mismatch" in f for f in failures),
                failures,
            )

    def test_version_sync_coupling_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-vsync-") as tmp:
            root = _copy_skill_fixture(tmp)
            sync_sh = root / "scripts" / "version" / "version-sync.sh"
            sync_sh.write_text(
                sync_sh.read_text(encoding="utf-8") + "\n# accidental skills/ccr-cli/manifest.json\n",
                encoding="utf-8",
            )
            failures = check_repo(root)
            self.assertTrue(any("decoupled from skills/ccr-cli" in f for f in failures), failures)

    def test_trigger_eval_failure_or_empty_bucket_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-trig-") as tmp:
            root = _copy_skill_fixture(tmp)
            cases_path = root / SKILL_REL_DIR / "evals" / "trigger_cases.json"
            cases = json.loads(cases_path.read_text(encoding="utf-8"))
            cases["near_neighbor"] = []
            cases_path.write_text(json.dumps(cases, indent=2) + "\n", encoding="utf-8")

            eval_path = root / SKILL_REL_DIR / "reports" / "trigger-eval.json"
            trig = json.loads(eval_path.read_text(encoding="utf-8"))
            trig["ok"] = False
            eval_path.write_text(json.dumps(trig, indent=2) + "\n", encoding="utf-8")

            failures = check_repo(root)
            self.assertTrue(any("bucket 'near_neighbor' must be non-empty" in f for f in failures), failures)
            self.assertTrue(any("trigger-eval.json: 'ok' must be true" in f for f in failures), failures)

    def test_fabricated_output_evidence_or_missing_evidence_omission_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-ev-") as tmp:
            root = _copy_skill_fixture(tmp)
            (root / SKILL_REL_DIR / "reports" / "output-evidence.json").write_text(
                '{"ok": true}\n', encoding="utf-8"
            )
            handoff = root / SKILL_REL_DIR / "reports" / "creation-handoff.md"
            handoff.write_text("# Creation Handoff 1.0.0\nAll online metrics verified.\n", encoding="utf-8")
            failures = check_repo(root)
            self.assertTrue(any("output-evidence.json must not exist" in f for f in failures), failures)
            self.assertTrue(any("missing evidence" in f for f in failures), failures)

    def test_secret_tokens_and_short_quoted_placeholders_are_blocked(self) -> None:
        samples = (
            "sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234",
            "xai-abcdefghijklmnopqrstuvwxyz123456",
            "ghp_abcdefghijklmnopqrstuvwxyz1234567890",
            "'sk-demo'",
            '"xai-demo"',
        )
        for token in samples:
            with self.subTest(token=token[:10]):
                with tempfile.TemporaryDirectory(prefix="ccr-skill-sec-") as tmp:
                    root = _copy_skill_fixture(tmp)
                    ref_doc = root / SKILL_REL_DIR / "references" / "platforms-and-fields.md"
                    ref_doc.write_text(
                        ref_doc.read_text(encoding="utf-8") + f"\nExample leaked value: {token}\n",
                        encoding="utf-8",
                    )
                    failures = check_repo(root)
                    self.assertTrue(any("secret scan matched" in f for f in failures), failures)

    def test_banned_legacy_hidden_and_unsafe_cli_commands_are_blocked(self) -> None:
        banned_snippets = (
            "Run `ccr switch openrouter` to change profile.",
            "Run `ccr -y delete old-profile` to delete.",
            "Run `ccr claude switch openrouter` without profile subcommand.",
            "Run `ccr claude profile set-field demo model sonnet`.",
            "Run `ccr claude profile create demo --auth-token my-value --json`.",
            "Run `ccr grok profile create demo --api-key=my-value --json`.",
            "Run `printf '%s' 'literal-token' | ccr claude profile create demo --auth-token-stdin --json`.",
            "Run `ccr claude profile export --stdout` without redaction.",
        )
        for snippet in banned_snippets:
            with self.subTest(snippet=snippet):
                with tempfile.TemporaryDirectory(prefix="ccr-skill-ban-") as tmp:
                    root = _copy_skill_fixture(tmp)
                    readme = root / SKILL_REL_DIR / "README.md"
                    readme.write_text(
                        readme.read_text(encoding="utf-8") + f"\n{snippet}\n",
                        encoding="utf-8",
                    )
                    failures = check_repo(root)
                    self.assertTrue(
                        any("banned legacy/hidden CLI syntax" in f for f in failures),
                        failures,
                    )

    def test_invalid_utf8_file_is_reported_cleanly_without_crashing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ccr-skill-utf8-") as tmp:
            root = _copy_skill_fixture(tmp)
            bad_file = root / SKILL_REL_DIR / "reports" / "prior-art-research.md"
            bad_file.write_bytes(b"\xff\xfe\x00\x00invalid")
            failures = check_repo(root)
            self.assertTrue(any("failed to read valid UTF-8 text" in f for f in failures), failures)


if __name__ == "__main__":
    unittest.main()
