# TUI 终端主题自适应：Catppuccin Latte/Mocha

## Goal

ccr TUI 启动时默认自动识别终端亮/暗背景，应用 Catppuccin Latte（亮）/ Mocha（暗）主题；用户可显式固定主题，固定后不再探测。

## Background / Confirmed Facts

主题系统与检测管线**已存在**，本任务只改"启动策略"：

- 中央主题模块 `crates/ccr-tui/src/tui/theme.rs`：`Palette`（:47）、`static MOCHA`（:89）/ `static LATTE`（:118）已是完整 Catppuccin 配色；生产代码零硬编码颜色（421 处调用全走 `theme::` 访问器），换色板零调用点改动。
- 探测已实现：`detect_terminal_variant()`（theme.rs:274）用 `termbg = "0.6.2"`（OSC 11 + COLORFGBG 回退），100ms 超时，非 TTY 跳过。
- 现状缺口：`resolve_startup_variant`（theme.rs:226-237）仅在 `CCR_TUI_THEME=auto` 时探测；env 未设置/非法 → 直接用持久化主题，不探测。
- 持久化配置：`TuiTheme { Mocha, Latte }`（`crates/ccr-config/src/managers/tui_config.rs:42-65`，`tui.toml`，默认 Mocha，未知值 warn 回退 Mocha :88-105）；**无 Auto 变体**。
- 启动链路：`run_tui_with()`（`tui/mod.rs:198-216`）→ `theme::init_theme(tui_config.theme)`（theme.rs:207）；运行时 Ctrl+T → `toggle_theme_and_persist()`（runtime.rs:294-305 → theme.rs:239）。
- Spec 约束：`.trellis/spec/ccr-tui/backend/backend-guidelines.md`（:283-382, :561）规定"未设置/非法 env 不得探测"，并把 auto 路径外探测的 ~100ms 首帧等待列为 Bad case；`.trellis/spec/ccr-config/backend/backend-guidelines.md`（:253-288）记录 TuiTheme 序列化规则。两份都需随本任务更新。

## Key Decisions

- D1（用户已确认）: 新增持久化 `theme = "auto"` 并作为 `TuiConfig` 默认值；存量显式固定的配置行为不变。代价：auto 路径每次启动最多 ~100ms 探测（仅 TTY）。
- D2: 旧 `tui.toml` 无 `theme` 键 → 视为未固定，随新默认进入 auto 探测（特性目标行为）。
- D3: env `CCR_TUI_THEME=mocha/latte` 永远优先固定；`auto` 强制探测；非法值 warn 后按持久化配置处理（沿用现状语义）。
- D4: Ctrl+T 手动切换 = 固定为可见变体；回到 auto 需改 `tui.toml` 或 env。不新增 UI。
- D5: 未知 theme 值的反序列化回退目标由 Mocha 改为 Auto（随默认值调整，保留 warn）。

## Requirements

- R1: `TuiTheme` 增加 `Auto` 变体（序列化 `"auto"`），成为 `TuiConfig` 默认值（tui_config.rs:42-65, :88-105, :151-159）。
- R2: `resolve_startup_variant` 支持持久化 auto：持久化 Auto 且 env 未固定 → 探测；失败回退 Mocha（theme.rs:226-237）。
- R3: 固定主题（持久化 mocha/latte 或 env 固定值）路径零探测、零启动延迟。
- R4: 探测失败（非 TTY/超时/不支持）静默回退，不阻塞启动、不报错。
- R5: 移除只服务于旧回退语义的 `From<TuiTheme> for ThemeVariant`（theme.rs:255-262，全仓库唯一消费方已确认），解析处显式 match。
- R6: 同步更新上述两份 spec 中与本策略冲突的条款。
- R7: 更新 ccr-config 与 ccr-tui 相关单元测试（tui_config.rs:323-329, :610-634；theme.rs:659-693）。

## Acceptance Criteria

- [ ] AC1: 无 env、持久化 `theme = "auto"`（或缺省）时，`resolve_startup_variant` 调用探测器：探测亮 → Latte，暗 → Mocha（mock detect 单元测试）。
- [ ] AC2: 持久化 `mocha`/`latte` 或 env 固定值时探测器**不被调用**（单元测试断言）。
- [ ] AC3: 探测失败（mock 返回 None）回退：持久化 Auto → Mocha；env=auto + 持久化固定值 → 该固定值。
- [ ] AC4: `tui.toml` 写 `theme = "auto"` 可 load/save round-trip；未知值回退 Auto 且有 warn。
- [ ] AC5: `rtk cargo test -p ccr-config`、`rtk cargo test -p ccr-tui`、`just fmt-check`、`just lint-strict`、`just test` 全绿。
- [ ] AC6: 手动冒烟：亮色终端启动呈 Latte、暗色呈 Mocha；固定主题后启动无探测延迟；Ctrl+T 切换并持久化。
- [ ] AC7: 两份 spec 与实现后的行为一致（逐条核对 check.jsonl）。

## Out of Scope

- ccr-ui（Tauri/Web）、ccr-vscode、docs 站点的主题与文档（已确认 docs 无 `CCR_TUI_THEME` 用户文档）。
- 第三套色板 / 用户自定义颜色。
- 运行时监听终端背景变化（仅启动时检测一次）。
- auto 模式下的 UI 指示器或"恢复 auto"快捷键。

## Risks / Deferred

- 存量无 `theme` 键配置的用户启动时多出最多 ~100ms 探测延迟（D1 已接受的代价）。
- 极少数不支持 OSC 11/COLORFGBG 的终端会每次等到超时后回退 Mocha —— 可通过显式固定主题规避，文档化于 spec。
