"""Validate planning artifacts only; this does not verify the future application."""
from pathlib import Path
import json
import re

root = Path.cwd()
parent = root / ".trellis/tasks/10-17-gpui-desktop"
meta = json.loads((parent / "task.json").read_text(encoding="utf-8"))
assert len(meta["children"]) == 10
issues = []
manifest_rows = 0
folders = [parent] + [parent.parent / c for c in meta["children"]]
for folder in folders:
    task = json.loads((folder / "task.json").read_text(encoding="utf-8"))
    if task["status"] != "planning":
        issues.append(f"{folder.name}: unexpected status")
    if folder != parent and task["parent"] != parent.name:
        issues.append(f"{folder.name}: wrong parent")
    for name in ("prd.md", "design.md", "implement.md"):
        content = (folder / name).read_text(encoding="utf-8")
        if re.search(r"^[-*] (?:\[.\] )?TBD\b", content, re.M):
            issues.append(f"{folder.name}/{name}: placeholder")
    for name in ("implement.jsonl", "check.jsonl"):
        rows = [json.loads(line) for line in (folder / name).read_text(encoding="utf-8").splitlines() if line.strip()]
        if not rows:
            issues.append(f"{folder.name}/{name}: empty manifest")
        for row in rows:
            manifest_rows += 1
            if not (root / row["file"]).is_file():
                issues.append("Missing context: " + row["file"])
design = (root / "ccr-desktop/DESIGN.md").read_text(encoding="utf-8")
assert "<!-- design-md: entry=style:codex;" in design
colors = dict(re.findall(r'^  ([\w-]+): "(#[0-9A-Fa-f]{6})"$', design, re.M))

def luminance(color):
    channels = [int(color[i:i+2], 16) / 255 for i in (1, 3, 5)]
    channels = [x / 12.92 if x <= .04045 else ((x + .055) / 1.055)**2.4 for x in channels]
    return sum(x*y for x, y in zip(channels, (.2126, .7152, .0722)))

def contrast(a, b):
    high, low = sorted((luminance(a), luminance(b)), reverse=True)
    return (high + .05) / (low + .05)

pairs = []
for suffix in ("", "-dark"):
    for bg in ("surface", "neutral", "elevated"):
        for fg in ("text", "text-muted", "success", "warning", "danger", "focus", "border-control"):
            minimum = 3 if fg in ("focus", "border-control") else 4.5
            ratio = contrast(colors[fg + suffix], colors[bg + suffix])
            pairs.append({"foreground": fg + suffix, "background": bg + suffix,
                          "ratio": round(ratio, 3), "minimum": minimum, "pass": ratio >= minimum})
    ratio = contrast(colors["secondary" + suffix], colors["primary" + suffix])
    pairs.append({"foreground": "secondary" + suffix, "background": "primary" + suffix,
                  "ratio": round(ratio, 3), "minimum": 4.5, "pass": ratio >= 4.5})
for pair in pairs:
    if not pair["pass"]:
        issues.append("Low contrast: " + str(pair))
(parent / "research/design-contrast.json").write_text(
    json.dumps({"method": "sRGB relative luminance, static tokens only", "pairs": pairs}, indent=2) + "\n",
    encoding="utf-8",
)
inventory = json.loads((parent / "research/migration-inventory.json").read_text(encoding="utf-8"))
assert len(inventory["commands"]) == 348 and len(inventory["routes"]) == 76
allowed_owners = {name.removeprefix("10-17-") for name in meta["children"]}
for item in inventory["commands"] + inventory["routes"]:
    if item["owner"] not in allowed_owners:
        issues.append("Invalid owner: " + item["owner"])
    if item["status"] != "planned":
        issues.append("Unexpected implementation claim: " + str(item))
files = [file for folder in folders + [root / "ccr-desktop"] for file in folder.rglob("*") if file.is_file()]
for file in files:
    content = file.read_text(encoding="utf-8")
    for number, line in enumerate(content.splitlines(), 1):
        if line.rstrip() != line:
            issues.append(f"{file}:{number}: trailing whitespace")
    if file.suffix == ".py":
        compile(content, str(file), "exec")
assert not issues, "\n".join(issues)
print(json.dumps({"tasks": len(folders), "planning_documents": len(folders)*3,
                  "curated_manifest_rows": manifest_rows, "contrast_pairs": len(pairs),
                  "commands": len(inventory["commands"]), "routes": len(inventory["routes"]),
                  "all_pass": True}))
