#!/usr/bin/env python3
"""Read-only repository governance check for the top-level skills/ccr-cli package.

Uses only the Python standard library so hosted Linux/Windows/macOS CI runners
never depend on external ~/.gemini installations or third-party YAML packages.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

try:
    from scripts.common import REPO_ROOT
except ModuleNotFoundError:
    # pywin32 registers a namespace package named `scripts` in site-packages
    _scripts_dir = Path(__file__).resolve().parent
    while _scripts_dir.name != "scripts":
        _scripts_dir = _scripts_dir.parent
    sys.path.insert(0, str(_scripts_dir.parent))
    sys.modules.pop("scripts", None)
    from scripts.common import REPO_ROOT


SKILL_NAME = "ccr-cli"
SKILL_OWNER = "bahayonghang"
SKILL_REL_DIR = Path("skills") / SKILL_NAME
MAX_SKILL_UTF8_BYTES = 14_000
MAX_DESCRIPTION_CHARS = 1_024
MAX_IR_SECTION_ITEMS = 20
MIN_TROUBLESHOOTING_ITEMS = 3
SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+$")

REQUIRED_PACKAGE_FILES = (
    "SKILL.md",
    "README.md",
    "manifest.json",
    "agents/interface.yaml",
    "references/platforms-and-fields.md",
    "references/export-import-and-safety.md",
    "references/diagnostics-and-recovery.md",
    "evals/trigger_cases.json",
    "reports/skill-ir.json",
    "reports/trigger-eval.json",
    "reports/prior-art-research.md",
    "reports/creation-handoff.md",
)

REQUIRED_H2_SECTIONS = (
    "Router Rules",
    "Compact Workflow",
    "Gate Ladder",
    "Output Contract",
)

REQUIRED_SKILL_REF_LINKS = (
    "references/platforms-and-fields.md",
    "references/export-import-and-safety.md",
    "references/diagnostics-and-recovery.md",
)

REQUIRED_README_ANCHORS = (
    "npx skills add bahayonghang/ccr --skill ccr-cli",
    "你可以直接这样说",
    "validate_skill.py",
    "Troubleshooting",
)

REQUIRED_ADAPTER_TARGETS = {
    "openai",
    "claude",
    "generic",
    "agent-skills-compatible",
}

FORBIDDEN_BRANDING_SNIPPETS = (
    "qiaomu-profile",
    "vista8",
    "joeseesun",
    "向阳乔木",
)

ENGLISH_PRONOUN_RE = re.compile(r"\b(?:i|you|your|yours|we|our|ours|us)\b", re.IGNORECASE)
CHINESE_PRONOUN_RE = re.compile(r"(?:我们|你们|我|你|您)")
MD_LINK_RE = re.compile(r"\]\(([^)]+)\)")
LIST_ITEM_RE = re.compile(r"^\s*(?:[-*]|\d+\.)\s+\S+", re.MULTILINE)
FENCE_OPEN_RE = re.compile(r"^\s*(`{3,}|~{3,})")
BLOCK_SCALAR_HEADER_RE = re.compile(r"^([|>])([+-])?$")

SECRET_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("anthropic_key", re.compile(r"sk-ant-[A-Za-z0-9_-]{10,}")),
    ("openai_like_key", re.compile(r"sk-[A-Za-z0-9_-]{20,}")),
    ("xai_key", re.compile(r"xai-[A-Za-z0-9_-]{10,}")),
    ("quoted_short_key_literal", re.compile(r"['\"](?:sk|xai)-[A-Za-z0-9._-]{3,}['\"]")),
    ("github_pat", re.compile(r"(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}")),
    ("aws_access_key", re.compile(r"AKIA[0-9A-Z]{16}")),
    ("private_key_pem", re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----")),
    (
        "inline_credential_assignment",
        re.compile(
            r"(?i)(?:api[_-]?key|auth[_-]?token|secret|password|token)\s*[:=]\s*[\"'][^\"']{8,}[\"']"
        ),
    ),
)

BANNED_CLI_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "unprefixed_legacy_top_level_command",
        re.compile(
            r"\bccr(?:\s+-[A-Za-z0-9-]+)*\s+(?:switch|list|add|delete|enable|disable|init|clear|optimize|export|import)\b"
        ),
    ),
    (
        "retired_platform_positional_shortcut",
        re.compile(
            r"\bccr(?:\s+-[A-Za-z0-9-]+)*\s+(?:claude|codex|grok)\s+(?:switch|list|add|create|edit|delete|enable|disable|init|off|export|import)\b"
        ),
    ),
    (
        "hidden_profile_set_field",
        re.compile(r"\bprofile\s+set-field\b"),
    ),
    (
        "plaintext_inline_secret_flag",
        re.compile(r"(?<!clear-)--(?:auth-token|api-key)(?!-stdin\b)(?:\s+|\s*=|$)"),
    ),
    (
        "literal_secret_pipe_to_ccr",
        re.compile(r"\b(?:printf|echo)\b[^|\n$`]*\|\s*ccr\b"),
    ),
    (
        "unredacted_profile_export_stdout",
        re.compile(r"\bprofile\s+export\b(?![^\n`]*--no-secrets\b)[^\n`]*--stdout\b"),
    ),
)


def _read_utf8(path: Path, label: str, errors: list[str]) -> str | None:
    try:
        return path.read_bytes().decode("utf-8")
    except (OSError, UnicodeDecodeError) as err:
        errors.append(f"{label}: failed to read valid UTF-8 text ({err})")
        return None


def _strip_yaml_inline_comment(line: str) -> str:
    in_single = False
    in_double = False
    escaped = False
    for idx, ch in enumerate(line):
        if escaped:
            escaped = False
            continue
        if in_double and ch == "\\":
            escaped = True
            continue
        if ch == "'" and not in_double:
            in_single = not in_single
            continue
        if ch == '"' and not in_single:
            in_double = not in_double
            continue
        if ch == "#" and not in_single and not in_double:
            if idx == 0 or line[idx - 1].isspace():
                return line[:idx].rstrip()
    return line.rstrip()


def _unquote_yaml_scalar(raw: str) -> str:
    value = raw.strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {'"', "'"}:
        inner = value[1:-1]
        if value[0] == '"':
            inner = inner.replace('\\"', '"').replace("\\\\", "\\")
        else:
            inner = inner.replace("''", "'")
        return inner
    return value


def _fold_block_scalar(lines: list[str], style: str, chomp: str) -> str:
    if not lines or all(not line for line in lines):
        return ""
    if style == "|":
        body = "\n".join(lines)
    else:
        chunks: list[str] = []
        idx = 0
        while idx < len(lines):
            line = lines[idx]
            if line == "":
                blank_count = 0
                while idx < len(lines) and lines[idx] == "":
                    blank_count += 1
                    idx += 1
                chunks.append("\n" * blank_count)
                continue
            if line.startswith(" "):
                if chunks and not chunks[-1].endswith("\n"):
                    chunks.append("\n")
                chunks.append(line)
                idx += 1
                if idx < len(lines) and lines[idx] != "":
                    chunks.append("\n")
                continue
            if chunks and not chunks[-1].endswith("\n"):
                chunks.append(" ")
            chunks.append(line)
            idx += 1
        body = "".join(chunks)

    body = body.rstrip("\n")
    if chomp == "-":
        return body
    return body + "\n"


def parse_simple_yaml_mapping(text: str) -> dict[str, Any]:
    """Parse indentation-based YAML mappings, lists, and block scalars without PyYAML."""
    clean_text = text.lstrip("\ufeff")
    lines = clean_text.splitlines()
    root: dict[str, Any] = {}
    stack: list[tuple[int, dict[str, Any] | list[Any]]] = [(-1, root)]
    pending_key_stack: list[tuple[int, dict[str, Any], str]] = []

    idx = 0
    while idx < len(lines):
        raw_line = lines[idx]
        leading_ws = raw_line[: len(raw_line) - len(raw_line.lstrip(" \t"))]
        if "\t" in leading_ws:
            raise ValueError(f"YAML line {idx + 1} uses forbidden tab indentation")

        uncommented = _strip_yaml_inline_comment(raw_line)
        stripped = uncommented.strip()
        if not stripped:
            idx += 1
            continue

        indent = len(raw_line) - len(raw_line.lstrip(" "))

        while len(stack) > 1 and indent <= stack[-1][0]:
            stack.pop()

        if pending_key_stack and indent > pending_key_stack[-1][0]:
            parent_indent, parent_dict, key = pending_key_stack.pop()
            new_container: dict[str, Any] | list[Any] = (
                [] if stripped.startswith("- ") else {}
            )
            parent_dict[key] = new_container
            stack.append((parent_indent, new_container))
        else:
            while pending_key_stack and indent <= pending_key_stack[-1][0]:
                _, parent_dict, key = pending_key_stack.pop()
                parent_dict[key] = ""

        container = stack[-1][1]
        if stripped.startswith("- "):
            if not isinstance(container, list):
                raise ValueError(f"unexpected YAML list item at line {idx + 1}: {raw_line}")
            container.append(_unquote_yaml_scalar(stripped[2:]))
            idx += 1
            continue

        if not isinstance(container, dict):
            raise ValueError(f"unexpected YAML mapping entry inside list at line {idx + 1}: {raw_line}")
        if ":" not in stripped:
            raise ValueError(f"invalid YAML line {idx + 1} without ':': {raw_line}")

        key, raw_val = stripped.split(":", 1)
        key = _unquote_yaml_scalar(key.strip())
        if key in container or any(pk == key and pd is container for _, pd, pk in pending_key_stack):
            raise ValueError(f"duplicate YAML mapping key {key!r} at line {idx + 1}")

        val = raw_val.strip()
        block_match = BLOCK_SCALAR_HEADER_RE.fullmatch(val)
        if block_match is not None:
            style = block_match.group(1)
            chomp = block_match.group(2) or ""
            block_raw: list[str] = []
            idx += 1
            while idx < len(lines):
                candidate = lines[idx]
                cand_ws = candidate[: len(candidate) - len(candidate.lstrip(" \t"))]
                if "\t" in cand_ws:
                    raise ValueError(f"YAML line {idx + 1} uses forbidden tab indentation")
                if not candidate.strip():
                    block_raw.append("")
                    idx += 1
                    continue
                cand_indent = len(candidate) - len(candidate.lstrip(" "))
                if cand_indent <= indent:
                    break
                block_raw.append(candidate)
                idx += 1
            non_empty = [line for line in block_raw if line]
            if not non_empty:
                container[key] = ""
            else:
                block_indent = min(len(line) - len(line.lstrip(" ")) for line in non_empty)
                trimmed = [line[block_indent:] if line else "" for line in block_raw]
                container[key] = _fold_block_scalar(trimmed, style, chomp)
            continue

        if not val:
            pending_key_stack.append((indent, container, key))
        else:
            container[key] = _unquote_yaml_scalar(val)
        idx += 1

    while pending_key_stack:
        _, parent_dict, key = pending_key_stack.pop()
        parent_dict[key] = {}

    return root


def split_frontmatter(text: str) -> tuple[dict[str, Any] | None, str, str | None]:
    clean = text.lstrip("\ufeff")
    if not clean.startswith("---\n") and not clean.startswith("---\r\n"):
        return None, clean, "SKILL.md must begin with YAML frontmatter delimiter '---'"
    lines = clean.splitlines(keepends=True)
    end_idx: int | None = None
    for idx in range(1, len(lines)):
        if lines[idx].strip() == "---":
            end_idx = idx
            break
    if end_idx is None:
        return None, clean, "SKILL.md YAML frontmatter is missing closing '---'"
    raw_fm = "".join(lines[1:end_idx])
    body = "".join(lines[end_idx + 1 :])
    try:
        parsed = parse_simple_yaml_mapping(raw_fm)
    except ValueError as err:
        return None, body, f"SKILL.md frontmatter YAML parse error: {err}"
    return parsed, body, None


def strip_fences_and_html_comments(markdown: str) -> str:
    """Blank out HTML comments and fenced code blocks while preserving line count."""
    without_comments = re.sub(
        r"<!--.*?-->",
        lambda m: "\n" * m.group(0).count("\n"),
        markdown,
        flags=re.DOTALL,
    )
    out_lines: list[str] = []
    fence_char: str | None = None
    fence_len = 0
    for line in without_comments.splitlines():
        if fence_char is None:
            match = FENCE_OPEN_RE.match(line)
            if match is not None:
                marker = match.group(1)
                fence_char = marker[0]
                fence_len = len(marker)
                out_lines.append("")
                continue
            out_lines.append(line)
        else:
            close_re = re.compile(rf"^\s*{re.escape(fence_char)}{{{fence_len},}}\s*$")
            if close_re.match(line):
                fence_char = None
                fence_len = 0
            out_lines.append("")
    return "\n".join(out_lines)


def extract_h2_sections(body: str) -> dict[str, str]:
    clean_body = strip_fences_and_html_comments(body)
    sections: dict[str, list[str]] = {}
    current: str | None = None
    for line in clean_body.splitlines():
        if line.startswith("## ") and not line.startswith("### "):
            current = line[3:].strip()
            sections.setdefault(current, [])
        elif current is not None:
            sections[current].append(line)
    return {title: "\n".join(lines) for title, lines in sections.items()}


def extract_list_items(section_text: str) -> list[str]:
    items: list[str] = []
    for line in section_text.splitlines():
        stripped = line.strip()
        if re.match(r"^(?:[-*]|\d+\.)\s+\S+", stripped):
            items.append(re.sub(r"^(?:[-*]|\d+\.)\s+", "", stripped))
    return items


def validate_markdown_links(md_path: Path, text: str, skill_dir: Path) -> list[str]:
    errors: list[str] = []
    skill_root = skill_dir.resolve()
    rel_doc = md_path.relative_to(skill_dir).as_posix()
    clean_text = strip_fences_and_html_comments(text)
    for raw_target in MD_LINK_RE.findall(clean_text):
        target = raw_target.strip().split()[0]
        if (
            not target
            or target.startswith(("#", "http://", "https://", "mailto:", "file://"))
        ):
            continue
        if "\\" in target:
            errors.append(
                f"{rel_doc}: markdown link '{target}' must use POSIX forward slashes ('/')"
            )
            continue
        rel_part = target.split("#", 1)[0]
        if not rel_part:
            continue
        resolved = (md_path.parent / rel_part).resolve()
        try:
            resolved.relative_to(skill_root)
        except ValueError:
            errors.append(f"{rel_doc}: link '{target}' escapes skills/ccr-cli/")
            continue
        if not resolved.is_file():
            errors.append(f"{rel_doc}: broken relative link '{target}'")
    return errors


def check_repo(root: Path = REPO_ROOT) -> list[str]:
    errors: list[str] = []
    skills_root = root / "skills"
    skill_dir = root / SKILL_REL_DIR
    skill_md = skill_dir / "SKILL.md"

    if not skills_root.is_dir():
        return ["missing top-level skills/ directory"]
    if not skill_dir.is_dir():
        return ["missing skills/ccr-cli/ directory"]
    if not skill_md.is_file():
        return ["missing skills/ccr-cli/SKILL.md"]

    # 1. Single SKILL.md discovery contract across top-level skills/
    discovered_skills = sorted(skills_root.rglob("SKILL.md"))
    expected_only = [skill_md]
    if [p.resolve() for p in discovered_skills] != [p.resolve() for p in expected_only]:
        rel_list = [p.relative_to(root).as_posix() for p in discovered_skills]
        errors.append(
            "skills/ must contain solely skills/ccr-cli/SKILL.md (no nested or extra SKILL.md); "
            f"found {rel_list}"
        )

    # 2. Required governed package files & prohibited artifacts
    for rel in REQUIRED_PACKAGE_FILES:
        if not (skill_dir / rel).is_file():
            errors.append(f"skills/ccr-cli/{rel}: missing required file")

    for child in skill_dir.rglob("*"):
        rel_posix = child.relative_to(skill_dir).as_posix()
        if child.is_file() and child.name.lower().startswith("license"):
            errors.append(
                f"skills/ccr-cli/{rel_posix}: duplicate LICENSE file is forbidden inside skills/ccr-cli/"
            )
        if "qiaomu-profile" in rel_posix.lower():
            errors.append(
                f"skills/ccr-cli/{rel_posix}: external personal profile asset is forbidden"
            )

    refs_dir = skill_dir / "references"
    if refs_dir.is_dir():
        for entry in sorted(refs_dir.iterdir()):
            if entry.is_dir():
                errors.append(
                    f"skills/ccr-cli/references/{entry.name}: references/ must stay flat (one level deep)"
                )
            elif entry.suffix.lower() == ".md":
                ref_text = _read_utf8(
                    entry, f"skills/ccr-cli/references/{entry.name}", errors
                )
                if ref_text is not None:
                    ref_lines = ref_text.splitlines()
                    if len(ref_lines) > 100:
                        head = "\n".join(ref_lines[:25]).lower()
                        if "table of contents" not in head and "目录" not in head:
                            errors.append(
                                f"skills/ccr-cli/references/{entry.name}: >100 lines requires a top Table of Contents"
                            )

    # 3. SKILL.md LF-normalized UTF-8 byte budget, frontmatter, pronouns, H2 sections, and links
    skill_text = _read_utf8(skill_md, "skills/ccr-cli/SKILL.md", errors)
    frontmatter: dict[str, Any] | None = None
    frontmatter_description: str | None = None
    sections: dict[str, str] = {}
    if skill_text is not None:
        normalized_lf_bytes = len(
            skill_text.lstrip("\ufeff").replace("\r\n", "\n").encode("utf-8")
        )
        if normalized_lf_bytes > MAX_SKILL_UTF8_BYTES:
            errors.append(
                f"skills/ccr-cli/SKILL.md: UTF-8 size {normalized_lf_bytes} exceeds {MAX_SKILL_UTF8_BYTES} bytes"
            )
        frontmatter, body, fm_err = split_frontmatter(skill_text)
        if fm_err:
            errors.append(fm_err)
        elif frontmatter is not None:
            if frontmatter.get("name") != SKILL_NAME:
                errors.append(
                    f"skills/ccr-cli/SKILL.md: frontmatter name must be '{SKILL_NAME}', got {frontmatter.get('name')!r}"
                )
            if frontmatter.get("license") != "MIT":
                errors.append(
                    f"skills/ccr-cli/SKILL.md: frontmatter license must be 'MIT', got {frontmatter.get('license')!r}"
                )
            raw_desc = frontmatter.get("description")
            if not isinstance(raw_desc, str) or not raw_desc.strip():
                errors.append("skills/ccr-cli/SKILL.md: frontmatter description must be a non-empty string")
            else:
                frontmatter_description = raw_desc.strip()
                if len(frontmatter_description) > MAX_DESCRIPTION_CHARS:
                    errors.append(
                        f"skills/ccr-cli/SKILL.md: description length {len(frontmatter_description)} exceeds {MAX_DESCRIPTION_CHARS}"
                    )
                if ENGLISH_PRONOUN_RE.search(frontmatter_description) or CHINESE_PRONOUN_RE.search(frontmatter_description):
                    errors.append(
                        "skills/ccr-cli/SKILL.md: description must not use first- or second-person pronouns"
                    )

        sections = extract_h2_sections(body)
        for h2 in REQUIRED_H2_SECTIONS:
            if h2 not in sections:
                errors.append(f"skills/ccr-cli/SKILL.md: missing required H2 heading '## {h2}'")
            else:
                items = extract_list_items(sections[h2])
                if not items:
                    errors.append(
                        f"skills/ccr-cli/SKILL.md: section '## {h2}' must contain at least one list item for IR extraction"
                    )
                elif len(items) > MAX_IR_SECTION_ITEMS:
                    errors.append(
                        f"skills/ccr-cli/SKILL.md: section '## {h2}' has {len(items)} list items (exceeds {MAX_IR_SECTION_ITEMS} IR cap)"
                    )

        clean_skill_body = strip_fences_and_html_comments(body)
        for required_ref in REQUIRED_SKILL_REF_LINKS:
            if f"({required_ref})" not in clean_skill_body:
                errors.append(
                    f"skills/ccr-cli/SKILL.md: missing direct relative link to '{required_ref}'"
                )

    # Validate relative markdown links across SKILL.md, README.md, and references/*.md
    md_files_to_check = [skill_md]
    if (skill_dir / "README.md").is_file():
        md_files_to_check.append(skill_dir / "README.md")
    if refs_dir.is_dir():
        md_files_to_check.extend(sorted(refs_dir.glob("*.md")))
    for md_file in md_files_to_check:
        md_text = _read_utf8(
            md_file, f"skills/ccr-cli/{md_file.relative_to(skill_dir).as_posix()}", errors
        )
        if md_text is not None:
            errors.extend(validate_markdown_links(md_file, md_text, skill_dir))

    # 4. README.md anchors, >=3 Troubleshooting items, and branding cleanliness
    readme_path = skill_dir / "README.md"
    if readme_path.is_file():
        readme_text = _read_utf8(readme_path, "skills/ccr-cli/README.md", errors)
        if readme_text is not None:
            for anchor in REQUIRED_README_ANCHORS:
                if anchor not in readme_text:
                    errors.append(f"skills/ccr-cli/README.md: missing required anchor '{anchor}'")
            readme_sections = extract_h2_sections(readme_text)
            ts_body = readme_sections.get("Troubleshooting", "")
            ts_items = extract_list_items(ts_body)
            # Only count top-level numbered items if nested bullets are present, or total items if flat
            top_ts_items = re.findall(
                r"^(?:\d+\.|[-*])\s+\S+", ts_body, re.MULTILINE
            )
            effective_ts_count = len(top_ts_items) if top_ts_items else len(ts_items)
            if effective_ts_count < MIN_TROUBLESHOOTING_ITEMS:
                errors.append(
                    f"skills/ccr-cli/README.md: Troubleshooting must contain at least {MIN_TROUBLESHOOTING_ITEMS} items (found {effective_ts_count})"
                )

    for check_rel in ("SKILL.md", "README.md", "manifest.json", "agents/interface.yaml"):
        check_path = skill_dir / check_rel
        if check_path.is_file():
            raw_check = _read_utf8(check_path, f"skills/ccr-cli/{check_rel}", errors)
            if raw_check is not None:
                lowered = raw_check.lower()
                for snippet in FORBIDDEN_BRANDING_SNIPPETS:
                    if snippet.lower() in lowered:
                        errors.append(
                            f"skills/ccr-cli/{check_rel}: contains forbidden personal branding '{snippet}'"
                        )

    # 5. manifest.json validation
    manifest_version: str | None = None
    manifest_path = skill_dir / "manifest.json"
    manifest: dict[str, Any] = {}
    if manifest_path.is_file():
        manifest_raw = _read_utf8(manifest_path, "skills/ccr-cli/manifest.json", errors)
        if manifest_raw is not None:
            try:
                loaded = json.loads(manifest_raw)
                if not isinstance(loaded, dict):
                    raise ValueError("manifest.json root must be a JSON object")
                manifest = loaded
            except (json.JSONDecodeError, ValueError) as err:
                errors.append(f"skills/ccr-cli/manifest.json: {err}")
            else:
                if manifest.get("name") != SKILL_NAME:
                    errors.append(f"manifest.json: name must be '{SKILL_NAME}'")
                raw_ver = manifest.get("version")
                if not isinstance(raw_ver, str) or not SEMVER_RE.fullmatch(raw_ver):
                    errors.append(f"manifest.json: version must be semver x.y.z, got {raw_ver!r}")
                else:
                    manifest_version = raw_ver
                if manifest.get("owner") != SKILL_OWNER:
                    errors.append(f"manifest.json: owner must be '{SKILL_OWNER}'")
                if not isinstance(manifest.get("updated_at"), str) or not manifest["updated_at"].strip():
                    errors.append("manifest.json: updated_at must be a non-empty string")
                if not isinstance(manifest.get("status"), str) or not manifest["status"].strip():
                    errors.append("manifest.json: status must be a non-empty string")
                if manifest.get("maturity_tier") != "governed":
                    errors.append("manifest.json: maturity_tier must be 'governed'")
                if manifest.get("context_budget_tier") != "production":
                    errors.append("manifest.json: context_budget_tier must be 'production'")
                if not isinstance(manifest.get("release_gates"), dict) or not manifest["release_gates"]:
                    errors.append("manifest.json: release_gates must be a non-empty object")
                intent = manifest.get("intent")
                if not isinstance(intent, dict):
                    errors.append("manifest.json: intent must be an object")
                else:
                    if not isinstance(intent.get("job_to_be_done"), str) or not intent["job_to_be_done"].strip():
                        errors.append("manifest.json: intent.job_to_be_done must be non-empty")
                    for key in ("target_users", "inputs", "outputs", "exclusions"):
                        val = intent.get(key)
                        if not isinstance(val, list) or not val:
                            errors.append(f"manifest.json: intent.{key} must be a non-empty list")
                gates_cfg = manifest.get("release_gates", {})
                if (
                    isinstance(gates_cfg, dict)
                    and gates_cfg.get("output_eval") == "missing_evidence"
                    and (skill_dir / "reports" / "output-evidence.json").exists()
                ):
                    errors.append(
                        "reports/output-evidence.json must not exist while release_gates.output_eval is 'missing_evidence'"
                    )

    # 6. agents/interface.yaml validation
    interface_path = skill_dir / "agents" / "interface.yaml"
    if interface_path.is_file():
        iface_raw = _read_utf8(
            interface_path, "skills/ccr-cli/agents/interface.yaml", errors
        )
        if iface_raw is not None:
            try:
                iface_doc = parse_simple_yaml_mapping(iface_raw)
            except ValueError as err:
                errors.append(f"skills/ccr-cli/agents/interface.yaml: {err}")
            else:
                iface = iface_doc.get("interface")
                if not isinstance(iface, dict):
                    errors.append("agents/interface.yaml: missing 'interface' mapping")
                else:
                    for field in ("display_name", "short_description", "default_prompt"):
                        if not isinstance(iface.get(field), str) or not iface[field].strip():
                            errors.append(f"agents/interface.yaml: missing interface.{field}")
                compat = iface_doc.get("compatibility")
                if not isinstance(compat, dict):
                    errors.append("agents/interface.yaml: missing 'compatibility' mapping")
                else:
                    targets = compat.get("adapter_targets")
                    if not isinstance(targets, list) or not REQUIRED_ADAPTER_TARGETS.issubset(set(targets)):
                        errors.append(
                            "agents/interface.yaml: compatibility.adapter_targets must cover "
                            f"{sorted(REQUIRED_ADAPTER_TARGETS)}"
                        )
                    for submap in ("permissions", "trust", "degradation"):
                        if not isinstance(compat.get(submap), dict) or not compat[submap]:
                            errors.append(f"agents/interface.yaml: missing compatibility.{submap}")
                gates = iface_doc.get("gates")
                if not isinstance(gates, dict) or not gates:
                    errors.append("agents/interface.yaml: missing 'gates' mapping")

    # 7. evals/trigger_cases.json & reports/trigger-eval.json validation
    expected_case_count: int | None = None
    cases_path = skill_dir / "evals" / "trigger_cases.json"
    if cases_path.is_file():
        cases_raw = _read_utf8(cases_path, "skills/ccr-cli/evals/trigger_cases.json", errors)
        if cases_raw is not None:
            try:
                cases = json.loads(cases_raw)
                if not isinstance(cases, dict):
                    raise ValueError("trigger_cases.json root must be an object")
            except (json.JSONDecodeError, ValueError) as err:
                errors.append(f"evals/trigger_cases.json: {err}")
            else:
                if not isinstance(cases.get("positive_concepts"), dict) or not cases["positive_concepts"]:
                    errors.append("evals/trigger_cases.json: positive_concepts must be non-empty")
                if not isinstance(cases.get("description_required_concepts"), list) or not cases["description_required_concepts"]:
                    errors.append("evals/trigger_cases.json: description_required_concepts must be non-empty")
                bucket_total = 0
                for bucket in ("should_trigger", "should_not_trigger", "near_neighbor"):
                    items = cases.get(bucket)
                    if not isinstance(items, list) or not items:
                        errors.append(f"evals/trigger_cases.json: bucket '{bucket}' must be non-empty")
                    else:
                        bucket_total += len(items)
                if bucket_total > 0:
                    expected_case_count = bucket_total

    trigger_eval_path = skill_dir / "reports" / "trigger-eval.json"
    if trigger_eval_path.is_file():
        trig_raw = _read_utf8(
            trigger_eval_path, "skills/ccr-cli/reports/trigger-eval.json", errors
        )
        if trig_raw is not None:
            try:
                trig = json.loads(trig_raw)
                if not isinstance(trig, dict):
                    raise ValueError("trigger-eval.json root must be an object")
            except (json.JSONDecodeError, ValueError) as err:
                errors.append(f"reports/trigger-eval.json: {err}")
            else:
                if trig.get("ok") is not True:
                    errors.append("reports/trigger-eval.json: 'ok' must be true")
                if trig.get("failures") != []:
                    errors.append("reports/trigger-eval.json: 'failures' must be empty")
                if trig.get("missing_description_concepts") != []:
                    errors.append("reports/trigger-eval.json: 'missing_description_concepts' must be empty")
                summary = trig.get("summary")
                if not isinstance(summary, dict):
                    errors.append("reports/trigger-eval.json: missing 'summary' object")
                else:
                    total = summary.get("total", 0)
                    passed = summary.get("passed", -1)
                    if not isinstance(total, int) or total <= 0 or passed != total:
                        errors.append(
                            f"reports/trigger-eval.json: summary passed/total mismatch ({passed}/{total})"
                        )
                    if expected_case_count is not None and total != expected_case_count:
                        errors.append(
                            f"reports/trigger-eval.json: total {total} does not match trigger_cases.json count {expected_case_count}"
                        )

    # 8. reports/skill-ir.json, prior-art-research.md, creation-handoff.md & parity
    ir_path = skill_dir / "reports" / "skill-ir.json"
    if ir_path.is_file():
        ir_raw = _read_utf8(ir_path, "skills/ccr-cli/reports/skill-ir.json", errors)
        if ir_raw is not None:
            try:
                skill_ir = json.loads(ir_raw)
                if not isinstance(skill_ir, dict):
                    raise ValueError("skill-ir.json root must be an object")
            except (json.JSONDecodeError, ValueError) as err:
                errors.append(f"reports/skill-ir.json: {err}")
            else:
                pkg = skill_ir.get("package", {})
                if not isinstance(pkg, dict) or pkg.get("name") != SKILL_NAME:
                    errors.append(f"reports/skill-ir.json: package.name must be '{SKILL_NAME}'")
                if manifest_version is not None and pkg.get("version") != manifest_version:
                    errors.append(
                        f"version mismatch: manifest.json={manifest_version!r} vs reports/skill-ir.json={pkg.get('version')!r}"
                    )
                ir_intent = skill_ir.get("intent", {})
                if (
                    frontmatter_description is not None
                    and isinstance(ir_intent, dict)
                    and str(ir_intent.get("description", "")).strip() != frontmatter_description
                ):
                    errors.append(
                        "reports/skill-ir.json: intent.description is out of sync with SKILL.md frontmatter description"
                    )
                wf = skill_ir.get("workflow", {})
                if not isinstance(wf, dict):
                    errors.append("reports/skill-ir.json: missing workflow object")
                else:
                    for wf_key in ("router_rules", "compact_workflow", "gate_ladder", "output_contract"):
                        if not isinstance(wf.get(wf_key), list) or not wf[wf_key]:
                            errors.append(f"reports/skill-ir.json: workflow.{wf_key} must be non-empty")
                resources = skill_ir.get("resources")
                if not isinstance(resources, dict):
                    errors.append("reports/skill-ir.json: missing resources object")
                else:
                    skill_root_resolved = skill_dir.resolve()
                    for res_group, res_items in resources.items():
                        if not isinstance(res_items, list):
                            errors.append(
                                f"reports/skill-ir.json: resources.{res_group} must be a list"
                            )
                            continue
                        for res_rel in res_items:
                            if not isinstance(res_rel, str) or "\\" in res_rel:
                                errors.append(
                                    f"reports/skill-ir.json: resources.{res_group} entry {res_rel!r} must use POSIX '/' paths"
                                )
                                continue
                            resolved_res = (skill_dir / res_rel).resolve()
                            try:
                                resolved_res.relative_to(skill_root_resolved)
                            except ValueError:
                                errors.append(
                                    f"reports/skill-ir.json: resources.{res_group} entry '{res_rel}' escapes skills/ccr-cli/"
                                )
                                continue
                            if not resolved_res.is_file():
                                errors.append(
                                    f"reports/skill-ir.json: resources.{res_group} entry '{res_rel}' does not exist"
                                )

    if frontmatter is not None and manifest_version is not None:
        meta = frontmatter.get("metadata")
        if isinstance(meta, dict) and "version" in meta and meta["version"] != manifest_version:
            errors.append(
                f"version mismatch: manifest.json={manifest_version!r} vs SKILL.md metadata.version={meta['version']!r}"
            )

    prior_art_path = skill_dir / "reports" / "prior-art-research.md"
    if prior_art_path.is_file():
        pa_text = _read_utf8(
            prior_art_path, "skills/ccr-cli/reports/prior-art-research.md", errors
        )
        if pa_text is not None and not pa_text.strip():
            errors.append("reports/prior-art-research.md: file must not be empty")

    handoff_path = skill_dir / "reports" / "creation-handoff.md"
    if handoff_path.is_file():
        handoff_text = _read_utf8(
            handoff_path, "skills/ccr-cli/reports/creation-handoff.md", errors
        )
        if handoff_text is not None:
            if not handoff_text.strip():
                errors.append("reports/creation-handoff.md: file must not be empty")
            if "missing evidence" not in handoff_text.lower():
                errors.append(
                    "reports/creation-handoff.md: must explicitly declare unverified online/output metrics as 'missing evidence'"
                )
            if manifest_version is not None:
                ver_pattern = re.compile(rf"(?<![\d.]){re.escape(manifest_version)}(?![\d.])")
                if not ver_pattern.search(handoff_text):
                    errors.append(
                        f"version mismatch: reports/creation-handoff.md does not reference manifest version {manifest_version!r}"
                    )

    # Confirm workspace version-sync scripts stay decoupled from skills/ccr-cli
    for sync_rel in ("scripts/version/version-sync.ps1", "scripts/version/version-sync.sh"):
        sync_path = root / sync_rel
        if sync_path.is_file():
            sync_text = _read_utf8(sync_path, sync_rel, errors)
            if sync_text is not None and (
                "skills/ccr-cli" in sync_text or "skills\\ccr-cli" in sync_text
            ):
                errors.append(
                    f"{sync_rel}: must remain decoupled from skills/ccr-cli independent versioning"
                )

    # 9. Secret-leakage scan across all files under skills/ccr-cli/
    for file_path in sorted(p for p in skill_dir.rglob("*") if p.is_file()):
        rel_str = file_path.relative_to(skill_dir).as_posix()
        content = _read_utf8(file_path, f"skills/ccr-cli/{rel_str}", errors)
        if content is None:
            continue
        for label, pattern in SECRET_PATTERNS:
            match = pattern.search(content)
            if match is not None:
                line_no = content.count("\n", 0, match.start()) + 1
                errors.append(
                    f"skills/ccr-cli/{rel_str}:{line_no}: secret scan matched '{label}'"
                )

    # 10. Banned legacy/hidden CLI command scan across SKILL.md, README.md, interface.yaml, and references/*.md
    instruction_files = [
        skill_md,
        skill_dir / "README.md",
        skill_dir / "agents" / "interface.yaml",
    ]
    if refs_dir.is_dir():
        instruction_files.extend(sorted(refs_dir.glob("*.md")))
    for doc_path in instruction_files:
        if not doc_path.is_file():
            continue
        rel_str = doc_path.relative_to(skill_dir).as_posix()
        doc_text = _read_utf8(doc_path, f"skills/ccr-cli/{rel_str}", errors)
        if doc_text is None:
            continue
        for label, pattern in BANNED_CLI_PATTERNS:
            match = pattern.search(doc_text)
            if match is not None:
                line_no = doc_text.count("\n", 0, match.start()) + 1
                errors.append(
                    f"skills/ccr-cli/{rel_str}:{line_no}: banned legacy/hidden CLI syntax '{label}' ({match.group(0).strip()!r})"
                )

    return errors


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(errors="replace")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(errors="replace")

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=REPO_ROOT,
        help="Repository root directory to validate (defaults to REPO_ROOT).",
    )
    args = parser.parse_args(argv)
    resolved_root = args.root.resolve()
    failures = check_repo(resolved_root)
    if failures:
        print("ccr-cli governed skill check failed:", file=sys.stderr)
        for item in failures:
            print(f"  - {item}", file=sys.stderr)
        return 1
    skill_text = (resolved_root / SKILL_REL_DIR / "SKILL.md").read_text(encoding="utf-8")
    skill_bytes = len(skill_text.lstrip("\ufeff").replace("\r\n", "\n").encode("utf-8"))
    print(
        f"ccr-cli governed skill check passed (SKILL.md={skill_bytes}/{MAX_SKILL_UTF8_BYTES} LF UTF-8 bytes)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
