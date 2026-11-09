# Design：TUI Grok Profile tab（rev2）

> rev2 变更：①tab_order 迁移从"确认/补齐"升级为**明确的加载语义变更**（现状核验：缺失项报错 + 整体回落默认，丢用户排序）②详情面板从"复用通用渲染"改为 **Grok 专用 builder + core helper**（CORR-005）。

## 1. 触点清单

| 文件 | 变更 |
|---|---|
| `crates/ccr-config/src/managers/tui_config.rs` | `TuiTabId::GrokProfile` + `as_str` + `DEFAULT_TAB_ORDER`；`load()`/`validate_tab_order` 迁移语义变更（见 §2） |
| `crates/ccr-tui/src/tui/app.rs` | 白名单过滤（:483-485）加 Grok、tab 构建分支、display/compact label、`tab_config_id`、快照测试更新 |
| `crates/ccr-tui/src/tui/ui.rs` | 新增 `grok_profile_detail_lines`（详情分派处按平台选 builder） |
| `crates/ccr-tui/src/tui/i18n.rs` | Grok 相关 Message（tab 名、auth 模式标签等按需） |
| 切换 action 分发处 | 核对平台白名单，纳入 Grok（预期极小） |

## 2. tab_order 迁移语义（核验后的现状与变更）

现状（已核验）：

- `validate_tab_order`（tui_config.rs:245-256）：`tab_order` 缺任一 `DEFAULT_TAB_ORDER` 成员 → `ConfigFormatInvalid` 错误。
- `load_or_default`：load 错误 → **整体回落 `TuiConfig::default()`**，用户语言/主题/排序全部丢弃（测试 `load_or_default_falls_back_for_missing_tab_ids` 锁定了该行为）。
- 推论：任何给 `DEFAULT_TAB_ORDER` 加成员的变更，在不改加载语义时都会静默重置所有存量用户的 TUI 配置。

变更：

- `load()`：反序列化后，先按现状过滤弃用 `Usage`；再对缺失的默认成员**按 `DEFAULT_TAB_ORDER` 相对顺序追加到尾部**并 `warn!`（一次性列出补齐项）；`validate_tab_order` 的"缺失即错"检查移除或改为内部一致性断言（重复项检查保留）。
- 未知 id：现状是 serde 反序列化失败 → 整体回落默认，维持不变（不在本任务扩大兼容面）。
- 语言/主题字段的既有容错（warn + fallback 单字段）不受影响。
- 旧測试 `load_or_default_falls_back_for_missing_tab_ids` 语义反转：改写为"缺失项被补齐且保留自定义排序"。

`GrokProfile` 在 `DEFAULT_TAB_ORDER` 中的位置：`CodexProfile` 之后（Profile 类聚集，实现时按现有 const 结构落位）。

## 3. Tab 构建

```
for platform in Platform::implemented() {
    if !matches!(platform, Claude | Codex | Grok) { continue; }
    match platform {
        Claude => { ClaudeAuth; Profile }
        Codex  => { CodexAuth; OpenCodeAuth; Profile }
        Grok   => { Profile }    // label "Grok Profile"，两个 runtime summary 均 None
        _ => {}
    }
}
```

- `build_profile_tab_data` 走查：claude/codex runtime summary 探测按平台守卫，Grok 不触发。
- 三处映射：`display_label` → `tui_text!("Grok Profile", "Grok 配置")`；`compact_display_label` → `"Grok"`；`tab_config_id` → `Some(TuiTabId::GrokProfile)`（漏掉则顺序不持久化）。

## 4. Grok 专用详情面板（CORR-005）

现状（已核验）：`generic_profile_detail_lines`（ui.rs:928）仅渲染 description/base_url/model/account，URL 原样。Grok 需要专用 builder；业务判定不得在 TUI 重复实现，依赖 core 契约：

- `GrokPlatform::profile_auth_mode(&ProfileConfig)` → inline_api_key / env_key / session
- `GrokPlatform::safe_base_url_for_display(&str)` → 剥 userinfo/query/fragment

`grok_profile_detail_lines` 字段与来源：

| 显示项 | 来源 | 处理 |
|---|---|---|
| Description | `description` | 通用 tone |
| Base URL | `base_url` | **safe_base_url_for_display** |
| Model | `model` | accent tone |
| API Backend | `platform_data.api_backend` | 缺省显示 `responses (default)` |
| Auth | core `profile_auth_mode` | `env_key` 附变量名（如 `env_key (XAI_API_KEY)`）；**任何模式都不渲染 token 值/掩码串** |
| Context Window / Backend Search | platform_data | 数字/布尔格式化 |
| Tags / Usage / Enabled | 通用字段 | 沿既有渲染 |

详情分派：现有按平台选择 detail builder 的分支处（claude/codex 先例）加 Grok 臂；找不到分派点则在 generic 入口按 `platform == Grok` 分流。

## 5. 切换动作

通用 Profile 通道（选中→回车→异步 `instance.apply_profile`→toast→刷新）复用；实现时核对 action 分发处的平台 `matches!` 白名单并纳入 Grok。错误（含 core CAS 冲突的"请重试"错误）走既有 toast/overlay。

## 6. 测试设计

- `tui_config.rs`：R1 三组迁移测试（补齐保序/乱序缺多项/未知与弃用不回归）+ round-trip 含 grok_profile。
- `app.rs`：tab 快照更新、`tab_config_id` 映射、Grok 空态构建不 panic。
- `ui.rs`：grok 详情行——env_key 模式显示变量名、inline 模式无 token 输出、URL 剥离断言。
- 手动：临时 `CCR_ROOT`+`GROK_HOME` 双语过一遍列表/详情/切换/顺序持久化（真实 grok 启动验收见父 PRD 证据缺口）。
