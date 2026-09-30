#!/usr/bin/env python3
"""Check shared harness contracts without inspecting personal integration files."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
HARNESS_PAGES = ("docs/agents/harnesses.md", "docs/en/agents/harnesses.md")
GATE_SKILL = ".codex/skills/ccr-gate-recovery/SKILL.md"
SHARED_FILES = ("AGENTS.md", "CLAUDE.md", *HARNESS_PAGES, GATE_SKILL, ".trellis/workflow.md")
TOOLS = ("Claude Code", "Codex", "Grok Build", "Kimi Code", "OMP")
PRE_APPROVAL = re.compile(r"before\b[^.;|。\n]{0,50}\bapprov(?:al|es|ed)\b|pre-approval|(?:审批|批准)(?:实施)?前", re.I)
POST_APPROVAL = re.compile(r"after\b[^.;|。\n]{0,50}\bapprov(?:al|es|ed)\b|批准(?:实施)?(?:之)?后", re.I)


def markdown_parts(text: str) -> tuple[str, list[str]]:
    """Separate visible prose from fenced examples; ignore HTML comments."""
    prose: list[str] = []
    code: list[str] = []
    fence = ""
    for line in re.sub(r"<!--.*?-->", "", text, flags=re.S).splitlines():
        marker = re.match(r"^ {0,3}(\x60{3,}|~{3,})(.*)$", line)
        if marker:
            run, suffix = marker.groups()
            if not fence:
                fence = run
                continue
            if run[0] == fence[0] and len(run) >= len(fence) and not suffix.strip():
                fence = ""
                continue
        (code if fence else prose).append(line)
    return "\n".join(prose), code


def plain(text: str) -> str:
    return " ".join(text.replace("\x60", "").replace("*", "").split())


def table_rows(text: str, label: str) -> list[list[str]]:
    prose, _ = markdown_parts(text)
    rows = []
    for line in prose.splitlines():
        if not line.strip().startswith("|"):
            continue
        cells = [plain(cell) for cell in line.strip().strip("|").split("|")]
        if cells and cells[0].casefold() == label.casefold():
            rows.append(cells)
    return rows


def command_examples(text: str) -> list[str]:
    prose, code = markdown_parts(text)
    examples = re.findall(r"\x60([^\x60\n]+)\x60", prose)
    pending = ""
    for line in code:
        line = line.strip()
        pending += line.rstrip("\\\x60").rstrip() + " "
        if line.endswith(("\\", "\x60")):
            continue
        examples.append(pending.strip())
        pending = ""
    if pending:
        examples.append(pending.strip())
    return examples


def cargo_commands(text: str) -> list[str]:
    return [example for example in command_examples(text) if re.search(r"\bcargo\s+test\b", example)]


def check_contracts(root: Path) -> list[str]:
    failures: list[str] = []
    texts: dict[str, str] = {}

    def require(condition: bool, code: str, path: str, message: str) -> None:
        if not condition:
            failures.append(f"{code} {path}: {message}")

    for path in SHARED_FILES:
        try:
            text = (root / path).read_text(encoding="utf-8-sig")
            if not text.strip():
                raise ValueError("empty file")
            texts[path] = text
        except (OSError, UnicodeError, ValueError) as error:
            failures.append(f"HC001 {path}: shared file cannot be read ({error})")

    if "CLAUDE.md" in texts:
        prose, _ = markdown_parts(texts["CLAUDE.md"])
        require(bool(re.search(r"^ {0,3}@AGENTS\.md[ \t]*$", prose, re.M)), "HC002", "CLAUDE.md", "keep a real standalone @AGENTS.md import outside code and comments")

    for path in ("AGENTS.md", GATE_SKILL):
        if path in texts:
            require(all(tool in texts[path] for tool in TOOLS), "HC003", path, "name all five supported harnesses")

    for path in HARNESS_PAGES:
        if path not in texts:
            continue
        text = texts[path]
        for tool in TOOLS:
            require(len(table_rows(text, tool)) == 1, "HC003", path, f"keep exactly one integration row for {tool}")
        kimi = table_rows(text, "Kimi Code")
        if len(kimi) == 1 and len(kimi[0]) >= 3:
            entry, integration = kimi[0][1:3]
            current = re.split(r"fresh checkout|新检出", integration, maxsplit=1, flags=re.I)[0]
            require(".kimi-code/agents/" in entry and all(role in current for role in ("trellis-implement", "trellis-check", "trellis-research")) and "pull" in current.lower(), "HC004", path, "describe existing Kimi project agents and their pull context")
            denies_agents = re.search(r"(?:no|without)\s+(?:custom\s+)?project agents|project agents\s+(?:are\s+)?(?:absent|unavailable|not installed)|(?:没有|未安装|没装|无)项目(?:自定义\s*)?\s*agents", current, re.I)
            require(denies_agents is None, "HC004", path, "do not deny the audited Kimi project-agent integration")
            require(bool(re.search(r"fresh checkout|新检出", integration, re.I)) and bool(re.search(r"execution tools|执行工具", integration, re.I)), "HC004", path, "keep an explicit execution-capable pull fallback for uninitialized checkouts")
        else:
            require(False, "HC004", path, "Kimi integration row is missing or malformed")

        for label in ("Read-only reviewer" if path.startswith("docs/en/") else "只读 reviewer", "Trellis check"):
            rows = table_rows(text, label)
            if len(rows) != 1 or len(rows[0]) < 3:
                require(False, "HC006", path, f"keep a separate permissions row for {label}")
                continue
            when, permissions = rows[0][1:3]
            if label == "Trellis check":
                require(bool(POST_APPROVAL.search(when)) and not PRE_APPROVAL.search(when) and bool(re.search(r"may write|可写", permissions, re.I)) and bool(re.search(r"self-fix|可自修", permissions, re.I)), "HC006", path, "writable self-fix check must run after implementation approval")
            else:
                require(bool(PRE_APPROVAL.search(when)) and bool(re.search(r"do not (?:change|edit|write).{0,16}product|不改.{0,8}产品", permissions, re.I)), "HC006", path, "pre-approval reviewer must remain read-only for product files")

        required = (".gitignore", ".omp/extensions/trellis/index.ts", "trellis init --help", "--skip-existing", "--claude", "--codex", "--grok", "--kimi", "--omp")
        require(all(token in text for token in required), "HC007", path, "document delivered files and the version-aware fresh-checkout initialization steps")
        if path.startswith("docs/en/"):
            boundary = bool(re.search(r"(?:does not|do not).{0,30}(?:prove|establish).{0,50}native", text, re.I))
            boundary = boundary and all(word in text.lower() for word in ("fresh checkout", "generated", "native", "trust", "unverified"))
        else:
            boundary = "不能证明原生" in text and all(word in text for word in ("新检出", "本地生成", "原生", "trust", "未验证"))
        require(boundary, "HC007", path, "separate local generated files from verified native loading and trust")
        repair = next((line for line in text.splitlines() if "\x60just fmt\x60" in line and "\x60just version-sync\x60" in line), "")
        require(bool(re.search(r"may rewrite|会改文件", repair, re.I)), "HC008", path, "classify fmt and version-sync as file-changing repair commands")

    for path in ("AGENTS.md", "CLAUDE.md", *HARNESS_PAGES, GATE_SKILL):
        if path not in texts:
            continue
        text = texts[path]
        require(bool(re.search(r"default (?:test )?parallelism|默认并行", text, re.I)) and "--skip export_bindings" in text, "HC005", path, "retain default test parallelism and the binding-export skip")
        require(bool(re.search(r"separate[^.。\n]*(?:binding|export|generation)|独立[^。\n]*(?:绑定|生成)", text, re.I)), "HC005", path, "keep binding exports in the separate generation gate")
        commands = cargo_commands(text)
        require(not any(re.search(r"\bRUST_TEST_THREADS\s*=", example, re.I) for example in command_examples(text)), "HC005", path, "do not override default Rust test parallelism through environment assignments")
        if path in ("CLAUDE.md", GATE_SKILL):
            require(any("--workspace" in command and "--all-features" in command for command in commands), "HC005", path, "keep a direct workspace Rust test example")
        for command in commands:
            require(not re.search(r"--test-threads\b", command), "HC005", path, "do not override default Rust test parallelism")
            require(bool(re.search(r"\s--\s.*--skip(?:=|\s+)export_bindings(?:\s|$)", command)), "HC005", path, "direct cargo test examples must skip export_bindings")

    if "AGENTS.md" in texts:
        text = texts["AGENTS.md"]
        check = re.search(r"trellis[ -]check[^.\n]*", text, re.I)
        require("read-only reviewer" in text.lower() and check is not None and bool(POST_APPROVAL.search(check.group())) and not PRE_APPROVAL.search(check.group()) and "may write" in check.group().lower() and "self-fix" in check.group().lower(), "HC006", "AGENTS.md", "separate read-only review from post-approval writable Trellis check")
        require(all(word in text.lower() for word in ("fresh checkout", "generated", "native", "trust", "separate")), "HC007", "AGENTS.md", "retain the fresh-checkout and native evidence boundary")
        repair = next((line for line in text.splitlines() if "\x60just fmt\x60" in line and "\x60just version-sync\x60" in line), "")
        require(all(word in repair.lower() for word in ("repair", "separate", "validation")), "HC008", "AGENTS.md", "keep repair separate from validation")
    if GATE_SKILL in texts:
        text = texts[GATE_SKILL]
        require(bool(PRE_APPROVAL.search(text)) and bool(POST_APPROVAL.search(text)) and "only authorized checks" in text and "approved files" in text and "self-fix" in text, "HC006", GATE_SKILL, "keep approval and file-scope limits for execution roles")
        require(all(word in text.lower() for word in ("native", "hosted", "local", "separate")), "HC007", GATE_SKILL, "keep native, hosted, and local evidence separate")
    if "CLAUDE.md" in texts:
        repair = next((line for line in texts["CLAUDE.md"].splitlines() if "\x60just fmt\x60" in line and "\x60just version-sync\x60" in line), "")
        require("repair" in repair.lower() and "modify" in repair.lower(), "HC008", "CLAUDE.md", "describe the side effects of repair commands")
    if ".trellis/workflow.md" in texts:
        require(bool(re.search(r"task creation approval[^.\n]*not[^.\n]*implementation approval", texts[".trellis/workflow.md"], re.I)), "HC006", ".trellis/workflow.md", "task creation must not grant implementation approval")
    return failures


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="repository or isolated fixture root")
    args = parser.parse_args(argv)
    failures = check_contracts(args.root)
    if failures:
        print("Harness contract check failed:")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print(f"Harness contract check passed: {len(SHARED_FILES)} shared files, {len(TOOLS)} harnesses")
    print("Static guidance only; native loading, trust, and account settings remain unverified.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
