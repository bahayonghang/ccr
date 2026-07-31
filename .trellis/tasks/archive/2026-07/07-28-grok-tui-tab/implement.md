# Implement：TUI Grok Profile tab（rev2）

> 执行前置：core 已合入；`python ./.trellis/scripts/task.py start 07-28-grok-tui-tab`；读 `.trellis/spec/ccr-tui/backend/backend-guidelines.md`。

## 步骤清单

### 1. TuiTabId 与迁移语义（crates/ccr-config）

- [ ] `TuiTabId::GrokProfile` + `as_str("grok_profile")` + `DEFAULT_TAB_ORDER` 落位（CodexProfile 后）。
- [ ] `load()` 缺失默认成员补尾 + warn；`validate_tab_order` 去除"缺失即错"（保留重复检查）；改写旧测试 `load_or_default_falls_back_for_missing_tab_ids` 为补齐保序语义。
- [ ] R1 三组迁移测试（旧 5-id 补齐保序 / 乱序缺多项 / 未知与弃用 id 行为不回归）+ round-trip。
- 验证：`cargo test -p ccr-config tui_config -- --test-threads=1`

### 2. Tab 构建（crates/ccr-tui/src/tui/app.rs）

- [ ] 白名单过滤加 `Platform::Grok`；Grok 分支构建单 Profile tab（summary 均 None）。
- [ ] `display_label` / `compact_display_label` / `tab_config_id` 三处映射；i18n 文案。
- [ ] `build_profile_tab_data` 走查：runtime summary 探测平台守卫确认。
- [ ] tab 快照测试更新 + Grok 空态构建测试。
- 验证：`cargo check -p ccr-tui`

### 3. Grok 专用详情与切换（CORR-005）

- [ ] `ui.rs` 新增 `grok_profile_detail_lines`（字段表见 design §4；base_url 走 core `safe_base_url_for_display`、auth 走 core `profile_auth_mode`、任何模式不渲染 token 值）；详情分派处加 Grok 臂。
- [ ] 切换 action 分发白名单核对纳入 Grok；错误 toast 文案核对（含 CAS"请重试"）。
- [ ] footer 快捷键提示走查。
- [ ] `ui.rs` 详情单测：env_key 显示变量名 / inline 无 token 输出 / URL 剥离。
- 验证：`cargo test -p ccr-tui -- --test-threads=1`

### 4. 手动验收与收尾门

- [ ] 临时 `CCR_ROOT`+`GROK_HOME` 建 2 profile：列表/详情/切换/当前标记/顺序持久化，双语各一轮；确认 config.toml 变更符合 core 语义（真实 grok 启动验收按父 PRD 证据缺口处理）。
- [ ] `just fmt` → 查 diff → `just fmt-check` → `just lint-strict` → `just test`
- [ ] 提交拆分：① `feat(config): ✨ append missing default tui tabs on load`（步骤 1，含语义变更说明）② `feat(tui): ✨ add grok profile tab`（步骤 2-3）。

## 回滚点

- 步骤 1 是跨 tab 的加载语义变更，独立 commit 可单独 revert；步骤 2-3 依赖其存在（否则旧配置用户丢排序），revert 需成对评估。

## 明确不做

- Grok Auth tab / usage 内嵌 / ccr-ui；GrokPlatform 逻辑修改（缺陷回报 core）
- 未知 tab id 的兼容面扩大（维持现状回落）
