# 执行计划：卸载 basic-memory

所有命令在 Git Bash 中执行。备份目录：`BK=<scratchpad>/uninstall-bm-$(date +%Y%m%d-%H%M%S)`。

## Step 0 基线记录

- [ ] 记录 `npx skills list -g` 全文到 `$BK/skills-before.txt`
- [ ] 记录 `claude plugin list`、`claude plugin marketplace list` 到 `$BK/plugins-before.txt`
- [ ] 记录 `uv tool list` 到 `$BK/uv-before.txt`
- [ ] 记录 vault 与 `.basic-memory` 文件数：`find <path> -type f | wc -l`，写入 `$BK/counts-before.txt`

验证：三个文件存在且非空。

## Step 1 备份

- [ ] 复制 `.claude.json`、`.claude\settings.json`、`.claude\CLAUDE.md`、`.codex\config.toml`、`.omp\agent\mcp.json`、`.grok\config.toml`、`.config\opencode\opencode.json`、`.gemini\antigravity\mcp_config.json`、`.agents\.skill-lock.json` 到 `$BK/`

验证：`cmp` 与原文件一致。

## Step 2 Claude Code hook 与设置

- [ ] `settings.json`：删除 `hooks.SessionStart` 中含 `basic-memory hook session-start` 的 hook 项；删除 `hooks.PreCompact` 中含 `basic-memory hook pre-compact` 的整条；删除 `enabledPlugins["basic-memory@basicmachines-co"]`、`extraKnownMarketplaces.basicmachines-co`、`basicMemory` 块。空的 matcher 对象一并删除。
- [ ] 解析校验：`python -m json.tool .claude/settings.json >/dev/null`

## Step 3 Claude Code 插件与市场

- [ ] `claude plugin uninstall basic-memory@basicmachines-co`
- [ ] `claude plugin marketplace remove basicmachines-co`
- [ ] `claude plugin list` 与 `claude plugin marketplace list` 中不含 basic-memory / basicmachines-co

失败处理：记录错误输出，停止本步，不删缓存目录。

## Step 4 MCP 注册（六个工具）

- [ ] `.claude.json`：删除 `mcpServers.basic-memory`。保留 `basic-memory@basicmachines-co` 与 `basic-memory@inline` 的 usage 统计项（属于统计记录，不属于注册）。
- [ ] `.codex/config.toml`：删除 `[marketplaces.basic-memory]`、`[mcp_servers.basic-memory]`、`[mcp_servers.basic-memory.env]` 三段。
- [ ] `.omp/agent/mcp.json`：删除 `mcpServers.basic-memory`。
- [ ] `.grok/config.toml`：删除 `[mcp_servers.basic-memory]` 段。
- [ ] `.config/opencode/opencode.json`：删除 `mcp.basic-memory`。
- [ ] `.gemini/antigravity/mcp_config.json`：删除 `mcpServers.basic-memory`。

验证：

- `python -m json.tool` 对 JSON 文件校验
- `python -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" .codex/config.toml` 与 `.grok/config.toml`
- `rg -n "basic-memory|basic_memory" <六个文件>` 只允许剩余 usage 统计行

## Step 5 CLAUDE.md 段

- [ ] `.claude/CLAUDE.md`：删除 “## Basic Memory” 段（至下一个 `##` 标题前）。保留其他段。

验证：`rg -n "Basic Memory" .claude/CLAUDE.md` 无结果。

## Step 6 npx skills

- [ ] `npx skills list -g` 确认 16 个 `memory-*` 名称与来源
- [ ] `npx skills remove memory-capture memory-comark memory-continue memory-curate memory-defrag memory-ingest memory-lifecycle memory-literary-analysis memory-metadata-search memory-notes memory-onboarding memory-quest memory-reflect memory-research memory-schema memory-tasks -g -y`
- [ ] `npx skills list -g` 确认 `memory-*` 不存在；`design-md`、`tabbit` 仍在

验证：`ls ~/.claude/skills ~/.agents/skills | rg memory-` 只剩 `memory-ci-capture`（已记录，不删除）。

## Step 7 uv CLI

- [ ] `uv tool uninstall basic-memory`
- [ ] `uv tool list` 不含 basic-memory
- [ ] `where basic-memory` 与 `where bm` 无结果（如 scoop shim 仍在，记录原因）

## Step 8 终检

- [ ] 六个工具配置与 `.claude/` 中 `rg -il "basic-memory"` 只剩备份副本、session 日志与 usage 统计
- [ ] 文件数与 Step 0 一致（vault、`.basic-memory`）
- [ ] 关闭 Claude 会话的 SessionStart 报错检查：`claude --version` 正常

## 回滚

- 从 `$BK/` 复制原文件回原路径。
- 插件回滚：`claude plugin marketplace add basicmachines-co/basic-memory` 后 `claude plugin install basic-memory@basicmachines-co`。
- CLI 回滚：`uv tool install basic-memory --prerelease=allow`。
- skills 回滚：`npx skills add basicmachines-co/basic-memory/skills -g`。

## 检查点

- Step 2 之后：若 JSON 校验失败，先恢复该文件再继续。
- Step 3 之后：插件命令失败时停止，报告用户。
- Step 7 之前：确认 Step 2–5 全部通过，因为 CLI 卸载后 hook 不可用。

## 执行结果（2026-10-15）

- 备份目录：`C:\Users\lyh\AppData\Local\Temp\claude\D--Documents-Code-Github-ccr\7a318f55-85a1-435d-b748-406a104772ef\scratchpad\uninstall-bm-20261015-042746\`
- 已完成：Claude Code（`.claude.json`、`settings.json` hooks/plugin/marketplace/basicMemory、`CLAUDE.md` 段、插件与市场卸载）、Codex、OMP、Grok、OpenCode、Antigravity 的 MCP 配置；额外清理 Cursor `mcp.json`；`npx skills remove` 删除 16 个 `memory-*`；`memory-ci-capture` 悬空链接已删除；`uv tool uninstall basic-memory`。
- 未执行（权限拒绝 `rm -rf`）：`~/.gemini/antigravity{,-ide,-cli}/mcp/basic-memory/` 三个缓存目录；`~/.grok/marketplace-cache/8c7b19b67750d973*`。
- 保留：vault、`C:\Users\lyh\basic-memory`、`~/.basic-memory`（文件数前后一致：346 / 282 / 1）；`.claude.json` 内两条插件使用统计；各工具 memory/会话日志/`.bak` 中的文字引用；`.codex\.codex-global-state.json` 的导入发现缓存。
