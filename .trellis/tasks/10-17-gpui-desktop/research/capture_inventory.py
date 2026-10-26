"""Capture a reviewable migration inventory from committed application catalogs.

Run from repository root after exporting flattenCatalog() to routes-baseline.json.
This is planning evidence, not an implementation or runtime test.
"""
from pathlib import Path
import hashlib
import json
from collections import Counter

ROOT = Path.cwd()
HERE = ROOT / ".trellis/tasks/10-17-gpui-desktop/research"
TASKS = {
    "01": "gpui-01-foundation", "02": "gpui-02-services",
    "03": "gpui-03-shell", "04": "gpui-04-profiles",
    "05": "gpui-05-platforms", "06": "gpui-06-resources",
    "07": "gpui-07-insights", "08": "gpui-08-jobs",
    "09": "gpui-09-environments", "10": "gpui-10-acceptance",
}
MODULES = {
    "agent_sessions": "07", "builtin_prompts": "06",
    "checkin": "08", "checkin_extended": "08", "claude_auth": "05",
    "claude_observer": "07", "claude_profiles": "04", "codex_auth": "05",
    "codex_model_providers": "05", "command_exec": "08", "config": "04",
    "config_extended": "04", "converter": "04", "environment": "09",
    "environment_legacy": "09", "events": "02", "exit_confirm": "09",
    "install": "08", "mcp_presets": "06", "pricing": "07",
    "settings_raw": "05", "shell": "09", "ssh": "09", "sync": "08",
    "system": "09", "system_extended": "09", "system_extended_legacy": "08",
    "system_info": "09", "system_prompts": "06", "ui_state": "03",
    "unified_mcp": "06", "usage_v2": "07", "waf": "08", "wsl": "09",
}
RESOURCE_WORDS = (
    "_mcp", "_agent", "_slash", "_plugin", "_hook", "_prompt",
    "_output_style", "_statusline", "_extension", "_command", "_source_",
)
PLATFORMS = {"claude", "codex", "grok", "opencode", "gemini"}

def command_owner(command):
    module, name = command["module"], command["id"]
    if module not in PLATFORMS:
        return MODULES[module]
    if "tray" in name:
        return "09"
    if "profile" in name:
        return "04"
    if any(word in name for word in ("session", "usage", "dashboard", "quota", "budget")):
        return "07"
    if any(word in name for word in RESOURCE_WORDS):
        return "06"
    return "05"

def route_owner(route):
    path = (route.get("redirect") or route["path"]).strip("/")
    if path == "settings":
        return "03"
    if path in ("wsl", "ssh") or path.startswith("tray/"):
        return "09"
    if path in ("configs", "converter") or "profiles" in path:
        return "04"
    if path.startswith(("commands", "sync", "checkin")):
        return "08"
    if path.endswith("/commands"):
        return "06"
    if any(word in path for word in ("mcp", "agents", "skills", "plugins", "hooks",
                                    "slash-commands", "system-prompts", "output-styles", "statusline")):
        return "06"
    if path.endswith(("/auth", "/settings", "/providers")):
        return "05"
    return "07"

def write_json(name, data):
    (HERE / name).write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

manifest_path = ROOT / "ccr-ui/src/api/generated/command-manifest.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
commands = []
for item in manifest["commands"]:
    owner = command_owner(item)
    commands.append({
        "id": item["id"], "module": item["module"], "platform": item["platform"],
        "risk": item["risk"], "handler_path": item["handler_path"],
        "owner": TASKS[owner], "status": "planned", "native_action": None,
        "verification": None,
    })
assert len(commands) == manifest["windows_command_count"] == 348
assert len({row["id"] for row in commands}) == len(commands)
routes = json.loads((HERE / "routes-baseline.json").read_text(encoding="utf-8"))
route_map = [{
    "index": i, "path": r["path"], "id": r.get("id"), "redirect": r.get("redirect"),
    "owner": TASKS["03" if r["path"] == "/" and not r.get("id") else route_owner(r)],
    "kind": "redirect" if r.get("redirect") else "layout" if r["path"] == "/" and not r.get("id") else "surface",
    "status": "planned", "native_action": None, "verification": None,
} for i, r in enumerate(routes)]
assert len(routes) == 76
sources = ["Cargo.toml", "ccr-ui/src-tauri/Cargo.toml",
           "ccr-ui/src/api/generated/command-manifest.json",
           "ccr-ui/src/shell/routeCatalog.ts", "ccr-ui/src/config/platformDescriptors.ts", "justfile"]
write_json("migration-inventory.json", {
    "captured_on": "2026-10-17", "source_commit": "1dfe776b",
    "note": "Planning ownership snapshot. planned is not implemented or verified. Do not rerun over completed evidence.",
    "sha256": {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in sources},
    "counts": {"base_commands": manifest["base_command_count"], "windows_commands": len(commands),
               "routes": len(routes), "redirects": sum(bool(r.get("redirect")) for r in routes)},
    "commands": commands, "routes": route_map,
})
counts = Counter((c["module"], c["owner"]) for c in commands)
lines = ["# 功能迁移台账", "",
         "由 command-manifest.json 与实际执行 flattenCatalog() 的输出建立，不靠 grep 猜动态路由。",
         "基线为 340 通用 / 348 Windows commands、76 route records（含 18 redirects）。",
         "完整逐项 owner、风险类别、来源路径与待补 native action/verification 见 migration-inventory.json。",
         "所有项目当前均 planned，不代表任何功能已实现。执行时更新台账，不重新运行脚本覆盖证据。", "",
         "| 现有模块 | 命令数 | 实施子任务 |", "| --- | ---: | --- |"]
for (module, owner), count in sorted(counts.items()):
    lines.append(f"| {module} | {count} | {owner} |")
lines += ["", "## 路由处理规则", "",
          "redirect 合并到目标功能而保留可发现性；布局根由 shell 所有；tray 由原生环境任务承接。",
          "平台主页与数据页由 Insights 承接，设置/认证与资源页分域实现。现有只在命令层存在的能力也需明确 native 入口或书面归并理由。",
          "没有 native_action 与 verification 的业务项不得标 complete。新增源命令/路由必须先对比源 hash 和差异，再更新范围。",
          "", "## 手工核对重点", "",
          "Codex source-agent 安装/同步归资源任务；quota/session/usage 归 Insights；tray snapshot 归桌面集成。",
          "ui_state 收藏/最近项归 shell；events 订阅机制归共享 runtime；update_ccr 与 WAF 处理归 jobs。",
          "预算、定价、converter、output styles、statusline、builtin/system prompts 不能因没有显著导航而被丢弃。"]
(HERE / "feature-matrix.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print(json.dumps({"commands": len(commands), "routes": len(routes), "modules": len({c["module"] for c in commands}),
                  "owner_counts": Counter(c["owner"] for c in commands)}, ensure_ascii=False))
