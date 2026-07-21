# Implement — 系统配置管理完善

> 前置:prd.md + design.md 已评审;`task.py start` 后按序执行。每步含验证命令;标 ⛳ 为回滚点(该步失败可 revert 本步不影响前序)。

## Phase A ccr-core versioned API(共享前置物,先行)

- [x] A1 `crates/ccr-core/Cargo.toml` 加 `blake3.workspace = true`;在 `guarded_write.rs` 增加 `VersionedWriteOutcome::{Written, Conflict}`(遵守冻结 `CcrError` 契约)。
  - 验证:`cargo check -p ccr-core`
- [x] A2 `guarded_write.rs`:抽私有 `write_locked`;实现 `content_version_token` / `write_guarded_versioned` / `write_guarded_versioned_async`(设计 D1)。
  - 验证:`cargo test -p ccr-core guarded_write -- --test-threads=1`
- [x] A3 D1 全部单测(匹配写入/不匹配拒写且无备份/空令牌首建/并发 CAS/备份轮换)。⛳
  - 验证:同上 + `just lint-strict`(范围内)

## Phase B Tauri raw 命令

- [x] B1 `commands/settings_raw.rs`:`ensure_local_env` helper + 结构化 status 协议类型(设计 D2)。
- [x] B2 Claude get/save(JSON 语法 + ClaudeSettings 语义 + CAS + Dir 备份)。
- [x] B3 Codex get/save(TOML 语法 + CodexConfig 语义 + CAS + 缓存失效)。
- [x] B4 分层探测 `claude_list_settings_layers` / `codex_list_config_layers`(设计 D3)。
- [x] B5 注册 `handler_registry.rs`;命令层单测:令牌冲突、syntax/semantic invalid 带行列、unsupported_environment、message 不含 fixture 探针内容。⛳
  - 验证:`cd ccr-ui/src-tauri && cargo check && cargo test settings_raw -- --test-threads=1`

## Phase C 修复裸写

- [x] C1 `platform/local.rs` `write_config` 切 `write_guarded_async`(SameDir tag "ccr_ui");`platform/wsl.rs` 注释记录远程限制。⛳
  - 验证:`cargo test -p` 相关 + 手工:表单改一项保存,确认 `~/.claude/settings.json.ccr_ui_*.bak` 生成、内容正确

## Phase D 共享编辑器组件

- [x] D1 安装 CM6 细粒度依赖(bun);`components/editor/CodeSourceEditor.vue`(设计 D5:懒加载、三语言、errorMarker、主题跟随 tokens)。
- [x] D2 组件 smoke 测试(渲染/v-model/errorMarker)。⛳
  - 验证:`bun run type-check && bunx vitest run --config vitest.smoke.config.ts tests/<新增>.smoke.test.ts`;`bun run build` 确认 CM 独立 chunk

## Phase E 视图集成

- [x] E1 `src/api/domains/claude.ts` / `codex.ts` 加 raw + layers 包装与联合类型。
  - 验证:`bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`
- [x] E2 `ClaudeCodeSettingsView` 加 `source` tab(状态机、confirm、conflict/invalid/环境禁用、双向刷新,设计 D6)。
- [x] E3 `CodexSettingsView` 同构集成。
- [x] E4 两视图"配置层级"面板。
- [x] E5 i18n `settingsRaw.*` 双语。⛳
  - 验证:`bun run type-check && bun run lint && bun run test:i18n`

## Phase F 收口

- [ ] F1 手工验证矩阵:合法保存(备份+内容一致)/外部改文件后保存冲突/非法 JSON/非法 TOML/切 WSL 环境入口禁用/raw 后表单刷新。
- [x] F2 全量:`just fmt-check && just frontend-check-quick`,`cd ccr-ui/src-tauri && cargo clippy`;Rust 侧 `just test`。
- [x] F3 主会话内完成等价安全/前端质量复核(写路径、敏感字段、Local 门禁、冲突状态机、无持久化原文);按协作约束未派发子代理。
- [x] F4 对照 prd.md 验收清单逐项勾;通知父任务:共享前置物(versioned API + 编辑器)可供两个后继任务消费。

## 验证记录(2026-07-17)

- 已通过:`just frontend-check-quick`(87 files / 392 tests)、`just test`、handler registry focused tests、`git diff --check`、`just ci`(全部步骤绿色)。
- 已审查:锁内 CAS、备份/原子写、Local 后端门禁、校验消息不含原文、前端不持久化 raw 内容。
- 缺失证据:F1 的真实 Tauri 文件读写、外部修改冲突操作和环境切换手工矩阵未执行;in-app Browser 不可用,不得视为视觉/交互通过。

## 回滚

- 单步失败:revert 该 ⛳ 段;Phase A 是独立 crate 改动,可单独成 commit 先行合入。
- 全局回滚:功能均为增量(新命令/新 tab/新组件),revert 对应 commit 即可;唯一行为变更是 C1(write_config 加备份),回滚仅恢复裸写,无数据迁移问题。

## 提交切分建议

1. `feat(core): ✨ versioned guarded write (CAS + blake3 token)`(Phase A)
2. `fix(tauri): 🐛 local write_config 接入 guarded write`(Phase C)
3. `feat(tauri): ✨ settings/config raw 读写与分层探测命令`(Phase B)
4. `feat(ui): ✨ 共享源码编辑器组件`(Phase D)
5. `feat(ui): ✨ Settings 页源文件模式与配置层级面板`(Phase E)
