# Grok TUI：Grok Profile tab 切换界面

## Goal

在 ccr TUI 中新增 `Grok Profile` tab：列出 grok profiles、Grok 专用详情面板（脱敏）、回车切换（GrokPlatform.apply_profile）、当前 profile 标记与反馈，并修复 tab 顺序持久化的迁移语义使旧 `tui.toml` 无损升级。

> rev2：吸收审阅 CORR-005（通用详情渲染仅 description/base_url/model/account 四字段且 URL 原样输出，需 Grok 专用 builder + core helper）；tab_order 迁移问题经核验**比审阅所述更重**：`validate_tab_order` 对缺失项直接报错，`load_or_default` 整体回落默认值——旧 5-tab 配置在新增枚举后会**静默丢弃用户自定义排序**（`tui_config.rs` 测试 `load_or_default_falls_back_for_missing_tab_ids` 即现状证据），必须改加载语义。

## Requirements

### R1 TuiTabId 与迁移语义修复（crates/ccr-config/src/managers/tui_config.rs）

- 新增 `TuiTabId::GrokProfile`（`as_str = "grok_profile"`），加入 `DEFAULT_TAB_ORDER`。
- **加载语义变更**：`load()` 对 `tab_order` 缺失的默认 tab 不再报错，改为**按默认顺序追加到序列尾部 + warn 日志**，保留用户既有排序；未知/弃用 id 的过滤与回落行为维持现状。该变更影响所有 tab（不止 grok），需独立测试锁定：
  - 旧 5-id 文件 → 加载成功，grok_profile 追加，前 5 位顺序不变
  - 用户自定义乱序 + 缺多项 → 自定义顺序保留，缺失项按默认相对顺序补尾
  - 含未知 id / 弃用 usage → 现有行为不回归

### R2 Tab 构建与渲染（crates/ccr-tui/src/tui/app.rs 等）

- 平台过滤白名单（`app.rs:483-485`）加入 `Platform::Grok`，构建单一 `TabVariant::Profile` tab（无 Auth tab、runtime summary 均 None）。
- `display_label`（`tui_text!("Grok Profile", "Grok 配置")`）、`compact_display_label`（"Grok"）、`tab_config_id` → `TuiTabId::GrokProfile` 三处映射。
- **Grok 专用详情 builder**（CORR-005）：通用 `generic_profile_detail_lines` 仅渲染 description/base_url/model/account 且 URL 原样——新增 `grok_profile_detail_lines`，展示：description、base_url（**经 core `safe_base_url_for_display`**）、model、api_backend、auth 模式（**经 core `profile_auth_mode`**，显示 inline_api_key/env_key/session，env_key 模式附变量名）、context_window、supports_backend_search、tags/usage。**不渲染 token 本体**（掩码值也不显示，避免长度泄露）。

### R3 切换动作

- 回车切换走通用 `apply_profile` 异步动作通道 + toast 反馈 + 当前标记刷新；action 分发处如有平台白名单则纳入 Grok，不引入 Grok 特判业务逻辑。
- 切换失败（校验/CAS 冲突/IO）以现有错误呈现路径显示中文信息，不崩溃不静默。

### R4 约束

- 不做 Grok Auth tab、usage 内嵌。
- i18n 中英双语齐全；tab 快照类测试同步更新。
- 业务判定（auth 模式、安全 URL）一律调 core helper，TUI 不重复推断（CORR-005 审阅要求）。

## Acceptance Criteria

- [ ] TUI 可见 `Grok Profile` tab；空 profiles 走既有空态。
- [ ] 两个 grok profile：列表/切换/当前标记/toast 正确，`~/.grok/config.toml` 实际变更符合 core 语义。
- [ ] 详情面板：无 token 输出（含掩码形态）、base_url 剥离 userinfo/query、auth 模式与 Grok 特有字段正确显示，中英双语。
- [ ] 旧 5-tab `tui.toml` 启动：Grok tab 追加且用户排序保留（R1 三组测试通过）；调整顺序后持久化含 grok_profile。
- [ ] `cargo test -p ccr-config tui_config -- --test-threads=1`、`cargo test -p ccr-tui -- --test-threads=1`、`just lint-strict`、`just test` 通过。

## Notes

- 前置依赖：`07-28-grok-platform-core`（`create_platform(Grok)`、`profile_auth_mode`、`safe_base_url_for_display` 契约）。
- 与 cli-surface 无相互依赖，可并行。
