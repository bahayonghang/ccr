# 卸载 basic-memory MCP 与 skills

## Goal

在本机六个 AI 工具中移除 basic-memory 的 MCP 注册、插件、hook 与 skills，并卸载 uv 安装的 `basic-memory` CLI。笔记数据保留。

参考官方仓库 https://github.com/basicmachines-co/basic-memory。该仓库的 README 只给出安装方法（`uv tool install basic-memory`、`npx skills add basicmachines-co/basic-memory/skills -g`、`claude mcp add`、各客户端 MCP 配置片段），不含卸载步骤。卸载范围依据安装方法与本机实际配置文件确定。

## Requirements

- R1 移除 MCP 注册：
  - Claude Code：`C:\Users\lyh\.claude.json` 的 `mcpServers.basic-memory`
  - Codex：`C:\Users\lyh\.codex\config.toml` 的 `[mcp_servers.basic-memory]`（含 `.env` 子表）与 `[marketplaces.basic-memory]`
  - OMP：`C:\Users\lyh\.omp\agent\mcp.json` 的 `mcpServers.basic-memory`
  - Grok：`C:\Users\lyh\.grok\config.toml` 的 `[mcp_servers.basic-memory]`
  - OpenCode：`C:\Users\lyh\.config\opencode\opencode.json` 的 `mcp.basic-memory`
  - Antigravity：`C:\Users\lyh\.gemini\antigravity\mcp_config.json` 的 `mcpServers.basic-memory`
- R2 移除 Claude Code 插件与 hook：
  - `C:\Users\lyh\.claude\settings.json`：`enabledPlugins["basic-memory@basicmachines-co"]`、`extraKnownMarketplaces.basicmachines-co`、`basicMemory` 块、`hooks` 中两条 `basic-memory hook ...` 命令
  - 通过 `claude plugin uninstall basic-memory@basicmachines-co` 与 `claude plugin marketplace remove basicmachines-co` 卸载插件与市场（优先使用官方命令，不直接删缓存目录）
- R3 移除 npx skills 安装的 skills：`memory-*` 共 16 个（来源 `basicmachines-co/basic-memory`，记录于 `C:\Users\lyh\.agents\.skill-lock.json`），使用 `npx skills remove <names> -g -y` 清理所有 agent 链接。
- R4 `C:\Users\lyh\.claude\skills\memory-ci-capture`：该符号链接指向不存在的 `.agents\skills\memory-ci-capture`，来源不在 `.skill-lock.json` 中。默认不删除，列入报告（见 Open Questions）。
- R5 卸载 CLI：`uv tool uninstall basic-memory`。
- R6 `C:\Users\lyh\.claude\CLAUDE.md` 中 “## Basic Memory” 段删除（用户已确认）。
- R7 保留数据：vault `D:\Documents\LYH\100-Work\100-Notes\basic_memory`、`C:\Users\lyh\basic-memory`、`C:\Users\lyh\.basic-memory\` 均不删除（用户已确认）。

## Constraints

- 编辑任何配置文件前先备份到 `C:\Users\lyh\AppData\Local\Temp\claude\D--Documents-Code-Github-ccr\7a318f55-85a1-435d-b748-406a104772ef\scratchpad\uninstall-bm-<时间戳>\`，保留原文件名。
- 只删除 basic-memory 相关条目，保留同文件中其他 MCP、插件、hook 与设置。
- 不改动 `.bak` 历史备份文件与 `.grok\sessions\` 会话日志。
- 不改动 `.codex\config.toml.bak`、`.grok\config - Copy.toml` 等备份副本；列出给用户处理。
- 不删除 `C:\Users\lyh\.claude\projects\` 与 `.claude\backups\` 中的历史快照。
- 不把 Basic Memory 数据目录删除（用户选择“保留数据”）。

## Acceptance Criteria

- [ ] 六个工具的配置文件中搜索 `basic-memory` / `basic_memory` 命令或 server 名，只剩备份副本、历史日志与保留的笔记路径引用。
- [ ] `claude plugin list` 不再包含 `basic-memory@basicmachines-co`；`claude plugin marketplace list` 不再包含 `basicmachines-co`。
- [ ] `npx skills list -g` 不再包含 `memory-*` 与来源 `basicmachines-co/basic-memory` 的条目；`tabbit`、`design-md` 等无关 skill 保持不变。
- [ ] `C:\Users\lyh\.claude\skills\` 中 `memory-*` 链接全部移除；`memory-ci-capture` 保留并列入报告。
- [ ] `uv tool list` 不再包含 `basic-memory`；`where basic-memory` 无结果。
- [ ] `C:\Users\lyh\.claude\settings.json` 与 `.claude.json` 通过 JSON 解析；`config.toml` 与 `.grok\config.toml` 通过 TOML 解析。
- [ ] 笔记目录与 `.basic-memory` 目录文件数与操作前一致。
- [ ] 每一步的命令输出与校验结果写入报告。

## Open Questions

- `memory-ci-capture` 符号链接的来源是否是 basic-memory（当前无法从 lock 文件确认）。默认方案：不删除，报告给用户。

## Notes

- 用户已确认：建任务并规划；保留笔记数据；删除 CLAUDE.md 中 Basic Memory 段。
- 官方 README 无卸载说明，本任务的卸载清单由本机配置推导。
