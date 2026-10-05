# Design: TUI 终端主题自适应（auto 持久化选项）

## Architecture & Boundaries

- `ccr-config` 拥有 `TuiTheme` 枚举与 `tui.toml` 序列化契约（`crates/ccr-config/src/managers/tui_config.rs`）。
- `ccr-tui` 拥有启动解析与终端探测（`crates/ccr-tui/src/tui/theme.rs`），经 `ccr_cli::managers` 复导出读取配置。
- 不新增依赖：`termbg = "0.6.2"` 已是 ccr-tui 依赖；色板 MOCHA/LATTE 已存在。

## Data Flow

```
tui.toml → TuiConfigManager::load() → TuiConfig.theme: TuiTheme
  → run_tui_with() (tui/mod.rs:198) → theme::init_theme(configured) (theme.rs:207)
  → resolve_startup_variant(env: CCR_TUI_THEME, configured, detect_terminal_variant)
  → set_theme(variant) → ACTIVE: AtomicU8 → 全部渲染经 theme::palette()
```

## Contracts

### C1. `TuiTheme` 增加 `Auto` 并设为默认（tui_config.rs:42-65）

```rust
pub enum TuiTheme {
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "mocha")]
    Mocha,
    #[serde(rename = "latte")]
    Latte,
}
```

- `as_str()`：`Auto → "auto"`。
- `toggled()`：`Auto → Mocha`（目前仅测试使用，定义为确定性行为）。
- 反序列化函数 `deserialize_theme_or_mocha`（tui_config.rs:88-105）改为 `deserialize_theme_or_auto`：`"mocha"/"latte"/"auto"` 正常映射；未知值/非字符串 → `tracing::warn!` + 回退 `Auto`（原为 Mocha，属有意的行为变更，随默认值一起调整）。
- `TuiConfig::default().theme == TuiTheme::Auto`。

### C2. 启动解析语义（theme.rs:226-237 `resolve_startup_variant`）

| env `CCR_TUI_THEME` | 持久化 theme | 行为 |
|---|---|---|
| `mocha` / `latte` | 任意 | 固定该值，**不探测** |
| `auto` | Mocha/Latte | 探测；失败回退持久化值 |
| `auto` | Auto | 探测；失败回退 **Mocha** |
| 未设置 / 非法值 | Mocha/Latte | 用持久化值，**不探测**（非法 env 保留现有 warn） |
| 未设置 / 非法值 | Auto | 探测；失败回退 Mocha |

### C3. 移除 `From<TuiTheme> for ThemeVariant`（theme.rs:255-262）

`Auto` 无法映射到具体变体。该 impl 全仓库仅 `resolve_startup_variant` 的 `configured.into()` 一处使用（已 grep 确认），改为在解析处显式 `match configured`。保留 `From<ThemeVariant> for TuiTheme`（持久化路径 theme.rs:264 使用，只会产出 Mocha/Latte）。

### C4. Ctrl+T 运行时切换语义不变

`toggle_theme_and_persist()`（theme.rs:239）持久化当前可见变体（mocha/latte）→ 在 auto 模式下手动切换即"固定"。回到 auto 需编辑 `tui.toml` 或设 env。文档化，不新增 UI。

### C5. 探测成本边界

探测（≤100ms，termbg 超时）仅发生在解析结果为 auto 且 stdout 是 TTY 时；固定主题路径零探测、零延迟。探测失败一律静默回退，不报错不阻塞。

## Compatibility

- 旧 `tui.toml` 显式写了 `theme = "mocha"/"latte"`：行为完全不变（pin，不探测）。
- 旧 `tui.toml` 无 `theme` 键：此前默认 Mocha，现在默认 Auto（每次启动探测）。这是本特性的目标行为——未固定即自适应。
- `theme = "<未知值>"`：回退目标由 Mocha 改为 Auto（保留 warn）。

## Spec / Test Impact

- 更新 `.trellis/spec/ccr-tui/backend/backend-guidelines.md` "Startup Theme" 场景（:283-382, :561）：契约改为"固定主题不探测；auto（持久化或 env）探测；~100ms 探测等待仅属 auto 路径"。
- 更新 `.trellis/spec/ccr-config/backend/backend-guidelines.md` TuiTheme 序列化章节（:253-288）：新增 auto 值、默认值与回退规则。
- 测试改造见 implement.md 第 2 步；样式测试隔离约束（禁 `set_theme`）不变。

## Rollback

改动集中在 `tui_config.rs`、`theme.rs` 两个文件 + 两份 spec；按 crate 边界拆分 commit，回退单个 commit 即恢复旧默认。
