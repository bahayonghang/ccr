# 技术设计：卸载 basic-memory

## 来源与依据

- 官方仓库 README 给出的安装面：
  - `uv tool install basic-memory --prerelease=allow`（CLI）
  - `claude mcp add basic-memory -- uvx ...`、Codex `[mcp_servers.basic-memory]`、Cursor/VS Code `mcpServers` 片段（MCP 注册）
  - `bm install claude-code`（Claude Code 插件，会写入 settings）
  - `npx skills add basicmachines-co/basic-memory/skills -g`（skills）
- README 无卸载步骤。卸载面由以下本机检查结果确定：
  - `rg` 扫描 `.claude`、`.claude.json`、`.codex`、`.omp`、`.grok`、`.config`、`.gemini`、`.agents`、`.basic-memory` 等目录
  - `npx skills list -g` 的 `Source: basicmachines-co/basic-memory` 条目
  - `.agents\.skill-lock.json`（来源字段）

## 各工具的清理面

| 工具        | 文件 / 命令                                                                                                                                                                                 | 动作                                                           |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Claude Code | `.claude.json` → `mcpServers.basic-memory`                                                                                                                                                  | JSON 删除键                                                    |
| Claude Code | `.claude\settings.json` → `enabledPlugins`、`extraKnownMarketplaces.basicmachines-co`、`basicMemory`、`hooks.SessionStart[].hooks` 中 `basic-memory hook session-start`、`hooks.PreCompact` | JSON 删除对应项                                                |
| Claude Code | 插件 `basic-memory@basicmachines-co` 与市场 `basicmachines-co`                                                                                                                              | `claude plugin uninstall` / `claude plugin marketplace remove` |
| Claude Code | `.claude\CLAUDE.md` “## Basic Memory” 段                                                                                                                                                    | 文本删除该段（用户已确认）                                     |
| Codex       | `.codex\config.toml` → `[mcp_servers.basic-memory]`、`[mcp_servers.basic-memory.env]`、`[marketplaces.basic-memory]`                                                                        | TOML 删除表                                                    |
| OMP         | `.omp\agent\mcp.json` → `mcpServers.basic-memory`                                                                                                                                           | JSON 删除键                                                    |
| Grok        | `.grok\config.toml` → `[mcp_servers.basic-memory]`                                                                                                                                          | TOML 删除表                                                    |
| OpenCode    | `.config\opencode\opencode.json` → `mcp.basic-memory`                                                                                                                                       | JSON 删除键                                                    |
| Antigravity | `.gemini\antigravity\mcp_config.json` → `mcpServers.basic-memory`                                                                                                                           | JSON 删除键                                                    |
| 全部        | `.agents\skills\memory-*`（16 个）及各 agent 链接                                                                                                                                           | `npx skills remove ... -g -y`                                  |
| 全部        | `uv` 安装的 `basic-memory` / `bm` 命令                                                                                                                                                      | `uv tool uninstall basic-memory`                               |

## 顺序与原因

1. **先备份**：每个待改文件复制到 scratchpad 的 `uninstall-bm-<时间戳>\`。
2. **先删 Claude hook，再卸 CLI**：`settings.json` 中的 `basic-memory hook ...` 依赖 CLI。CLI 先卸会导致每次会话启动报错。
3. **先删插件，再删市场**：`claude plugin uninstall` 需要市场存在，顺序反过来会留下孤立的插件记录。
4. **MCP 配置可在任意时点删除**：各工具读取配置时找不到 server 即跳过；但为避免残留进程，先删配置再卸 CLI。
5. **skills 最后删**：`npx skills remove` 依赖 `.skill-lock.json` 的记录，先删 lock 会导致找不到条目。
6. **每次改 JSON / TOML 后立即解析校验**：非法语法会让工具的整份配置失效。

## 编辑方式

- JSON：用 Python `json` 读取、删除指定键、以 `indent=2`、`ensure_ascii=False` 写回。保持键顺序。写回前检查原文件是否带 BOM 或特殊缩进；如不同，改用 Edit 精确替换。
- TOML：优先 Edit 精确替换表块，避免重新序列化丢失注释。
- `CLAUDE.md`：Edit 精确替换 “## Basic Memory” 段（到下一个 `##` 标题为止）。

## 风险与回滚

- 回滚：备份目录内的原文件覆盖回原路径即可。
- `claude plugin uninstall` 失败：保留 `.claude\plugins\` 状态，报告错误，不手工删除缓存目录。
- `npx skills remove` 对 `design-md`、`tabbit`（来源 local）不得生效。执行前用 `npx skills list -g` 确认目标名单。
- 删除 CLI 后 `basic_memory` vault 不受影响（数据在 vault 目录，与 CLI 无关）。
