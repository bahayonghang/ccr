# Implement — 系统提示词管理

> 前置(硬性,首项检查):platform-settings-enhancement 的 `write_guarded_versioned`、`CodeSourceEditor`、`ensure_local_env` 已合入可消费。未满足则不 `task.py start`。可与 profile-toml-editing 并行(互不依赖)。

## Phase A 前置确认

- [x] A1 确认三个共享前置物可用;复核 design.md D0 冻结表在实现当下仍成立(文件路径未变)。

## Phase B 后端

- [x] B1 `commands/system_prompts.rs`:PromptFileSpec 注册表 + 四命令(list/get/save/create,设计 D1);OpenCode 路径复用 `opencode_config_dir()`;注册 handler_registry。
- [x] B2 Rust 单测(design D3:令牌冲突/首建/已存在 conflict/不存在分支/备份含 opencode 目录/大小告警/探针不入错误)。⛳
  - 验证:`cd ccr-ui/src-tauri && cargo check && cargo test system_prompts -- --test-threads=1`

## Phase C 前端

- [x] C1 `api/domains/systemPrompts.ts` + 类型。
  - 验证:`bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`
- [x] C2 `views/generic/SystemPromptsView.vue`(卡片列表 + 编辑区 + 创建流 + Claude 层级说明/rules 折叠区)。
- [x] C3 路由 4 条 + gemini-cli redirect;四个平台 View 子页导航加入口。
- [x] C4 i18n `systemPrompts.*` 双语;非 Local 环境入口禁用。⛳
  - 验证:`bun run type-check && bun run lint && bun run test:i18n`;路由 smoke

## Phase D 收口

- [ ] D1 手工矩阵:编辑保存 `~/.claude/CLAUDE.md` 与 `~/.codex/AGENTS.md`(磁盘一致 + `~/.ccr` 备份);OpenCode 创建流;Antigravity GEMINI.md 读写;外部修改后冲突;明暗主题走查。
- [x] D2 `just fmt-check && just frontend-check-quick`;`just test`(新增单测范围)。
- [x] D3 对照 prd.md 验收清单;父任务集成检查项联动(编辑器单实现三消费)。

## 验证记录(2026-07-17)

- 已通过:系统提示词 Rust 单测、路由/API/视图 smoke、`just frontend-check-quick`、`just test`、`just ci`。
- 已审查:四平台路径注册表、Local 后端门禁、CAS 冲突、首建、备份目录、大小告警与错误消息脱敏。
- 缺失证据:D1 的四平台真实文件编辑/创建、外部修改冲突操作及明暗主题手工走查未执行;in-app Browser 不可用。

## Phase E Bug 修复:production WebView 编辑器内容空白

> 详见 `fix-blank-content.md`。`just tdev` 启动的是 `tbuild` 生成的 release EXE；production WebView 证实内容已进入 CodeMirror,实际故障是运行时样式缺少 Tauri CSP nonce 后整张样式表被拒绝,不是后端空内容、正文颜色或 `v-model` 竞态。

- [x] E1 `CodeSourceEditor.vue`:读取页面已有 Tauri nonce,通过官方 `EditorView.cspNonce` facet 传给 CodeMirror,使运行时基础布局与主题样式通过 production CSP;同时为 `.cm-content` / `.cm-line` 锁定显式正文颜色。
- [x] E2 删除错误的首次 `docChanged` 抑制、挂载后重同步、`flush: 'post'` 与空内容日志;聚焦测试证明首次事件抑制会吞掉用户第一次真实编辑。
- [x] E3 `code-source-editor.smoke.test.ts`:锁定 CodeMirror 运行时样式继承 CSP nonce、显式前景色规则与第一次编辑立即回写。
- [x] E4 验证:`bun run type-check && bun run lint && bun run test`;`bun run tauri:check`;`just tbuild`;release WebView2 明暗主题走查。

### Phase E 验证记录(2026-07-17)

- 自动化:`code-source-editor` 聚焦 smoke 3/3、`system-prompts-view` 聚焦 smoke 2/2、完整前端 87 files / 394 tests、type-check、lint、Tauri `cargo check` 均通过;lint 仅保留既有 `DashboardSignalStream.vue` raw text warning。
- 修复前 release WebView2:CodeMirror 运行时 `<style>` 无 nonce 且 `style.sheet === null`;`.cm-scroller` 非 flex,gutter 约 389.6px x 2220.6px,正文 top 约 2372.9px,内容被布局到视口之外。
- 修复后 release WebView2:Claude/Codex 在浅色与深色下正文均可见;运行时 `<style>` 均带 Tauri nonce、`style.sheet` 可读、约 147 条规则;`.cm-editor` / `.cm-scroller` 均为 flex,gutter 宽 38.5px 且与正文 top 完全相同。验证过程未编辑或保存磁盘文件。

## 回滚

- Phase A–D 全部为增量(新命令模块/新视图/新路由/导航项),无既有行为变更;revert 对应 commit 即可。
- Phase E 修改集中在 `CodeSourceEditor.vue` 的 CSP nonce 接入、显式正文颜色和对应 smoke 回归;不改变文件读取、保存或 `v-model` 数据流。

## 提交切分建议

1. `feat(tauri): ✨ 系统提示词文件读写命令`(Phase B)
2. `feat(ui): ✨ 各平台系统提示词管理子页面`(Phase C)
