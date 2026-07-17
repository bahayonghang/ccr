# Implement — Profile 管理:profiles.toml 直接编辑

> 前置(硬性,首项检查):platform-settings-enhancement 的 `write_guarded_versioned`(含 `VersionedWriteOutcome::Conflict` 结果)与 `CodeSourceEditor` 组件已合入可消费。未满足则本任务不 `task.py start`。

## Phase A 前置确认

- [x] A1 确认 `ccr_core::core::write_guarded_versioned` 与 `components/editor/CodeSourceEditor.vue` 已在 dev 分支可用;确认 `ensure_local_env` helper 可复用(位于 settings_raw)。

## Phase B 核心库重构

- [x] B1 ccr-config `base.rs`:抽 `parse_profiles_from_str`,`load_profiles_from_toml` 改调;raw 校验层拒绝空集合;结构化 profiles 写入统一 `secret:true`(原读文件行为不变)。⛳
  - 验证:`cargo test -p ccr-config -- --test-threads=1`;`just test` 中 profiles 相关无回归

## Phase C Tauri 命令

- [x] C1 `profile_lifecycle.rs` 共享 helper:get(原文+令牌)/save(D2 四步校验链 + activation 保护 + CAS)。
- [x] C2 `claude_profiles.rs` / `codex_profiles.rs` 平台包装,注册 handler_registry。
- [x] C3 Rust 单测(design D4 全清单:syntax/semantic/空集合/force 二段式/令牌冲突/备份/探针不入错误消息)。⛳
  - 验证:`cd ccr-ui/src-tauri && cargo check && cargo test profile -- --test-threads=1`

## Phase D 前端

- [x] D1 domains API 包装 + 类型(claude.ts / codex.ts)。
  - 验证:`bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`
- [x] D2 `ClaudeCodeProfilesView` Edit TOML 入口 + 全屏编辑面板(状态机:confirm 打开 → 编辑 → invalid 定位 / activation_conflict force 重发 / conflict 重载;成功后全量刷新)。
- [x] D3 `CodexProfilesView` 同构。
- [x] D4 i18n `profilesRaw.*` 双语;非 Local 环境入口禁用。⛳
  - 验证:`bun run type-check && bun run lint && bun run test:i18n`

## Phase E 收口

- [ ] E1 手工矩阵:合法编辑保存后列表刷新 + `ccr current` 正常;删除激活 profile 走 force 流;外部改文件后保存冲突;非法 TOML 行号定位。
- [x] E2 `just fmt-check && just frontend-check-quick`;`just test`。
- [x] E3 主会话内完成等价安全复核(credential、secret 权限、CAS/force、错误脱敏);按协作约束未派发子代理,并已对照 prd.md 验收清单。

## 验证记录(2026-07-17)

- 已通过:`ccr-config` 65 tests、profile lifecycle/handler focused tests、Profiles 面板 smoke、`just frontend-check-quick`、`just test`、`just ci`。
- 已审查:原文直读、空集合拒绝、activation_conflict + force 二段式、stale token、备份、结构化/raw 写入 `secret:true`、保存后全量刷新。
- production WebView2 编辑器复核:当前 `HEAD` 重新 `just tbuild` 后,Claude/Codex Profiles raw 编辑器的 CodeMirror 运行时样式均带 nonce、`style.sheet` 可读且含 147 条规则、`.cm-scroller` 为 flex,gutter 与正文 top 均为 171.9375px;分别渲染 61/60 行。验证过程未编辑或保存磁盘文件。
- 截图所用旧 release EXE 构建于 19:19,早于共享 CSP 修复提交 `81dc31fc`(19:31);当前 release EXE 已于 19:41 重建并包含该修复。
- 缺失证据:E1 的真实 Tauri 保存、`ccr current`、外部修改冲突及非法 TOML 行号定位未手工执行。
- 用户验收:2026-07-17 用户确认问题已解决并明确要求归档;上述未手工执行项继续按缺失证据保留,不改写为通过。

## 回滚

- B1 是唯一触碰既有行为的重构(纯抽函数),独立 commit,可单独 revert。
- C/D 均为增量命令与 UI,revert 对应 commit 即可,无数据/格式迁移。

## 提交切分建议

1. `refactor(config): ♻️ 抽出 parse_profiles_from_str`(Phase B)
2. `feat(tauri): ✨ profiles raw TOML 读写命令`(Phase C)
3. `feat(ui): ✨ Profiles 页 TOML 直接编辑`(Phase D)
