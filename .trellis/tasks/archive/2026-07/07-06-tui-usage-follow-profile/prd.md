# TUI 用量统计跟随 profile：详情面板内嵌 provider 用量并下线独立 Usage tab

## Goal

选中某个 profile 时，在其详情面板（Context）内直接看到该 profile 对应 provider
的用量统计（请求数、token 拆分、近似成本），不再需要切到独立的 Usage tab。
独立 Usage tab 同步下线。

## Confirmed Facts

- 归因机制：CCR 在 profile 切换时向 `<CCR_ROOT>/analytics/provider_activation.jsonl`
  追加 `provider`（取自 `profile.provider` 字段）激活事件；llmusage sync 通过
  `--provider-map` 把用量窗口归因到 `provider_label`（schema 14）。
- 归因粒度是 **provider 级**，不是 profile 级：共享同一 provider 的多个 profile
  （如 anyrouter2/3/4 → `anyrouter`）看到相同数字。界面必须如实标注。
- 查询面：`ccr_usage::Dashboard::provider_breakdown_by_source(&[Claude, Codex],
  &QueryFilter::default())` 即现有 Usage tab 的唯一数据调用，返回
  `TaggedProviderBreakdown`（含 request_count/token 拆分/双成本）。无需新 SQL。
- 现有 `UsageApp`（`crates/ccr-tui/src/tui/usage/app.rs`）已实现可注入 loader
  seam + 状态机（Idle/Loading/Loaded/Empty/Unsupported/Error）+ spawn_blocking
  异步加载，可整体复用为数据引擎。
- Usage tab 没有独立 CLI 入口（无 `with_usage_tab`），只能通过 Tab 循环到达。
- `TuiTabId::Usage` 出现在 `crates/ccr-config/src/managers/tui_config.rs` 的
  enum、`DEFAULT_TAB_ORDER`（6 项）与 `validate_tab_order`（要求恰好列全 6 项）。
  用户已落盘的 `~/.ccr/tui.toml` 可能含 `usage` 项。
- 历史用量大多为 `provider = null`（unattributed）：provider 激活日志 2026-07
  才引入，且 profile 未填 `provider` 字段时事件也是 null。

## Requirements

- Claude/Codex profile 详情面板新增 "Usage" 分组，展示选中 profile 的
  `profile.provider` 在对应平台（source）下的聚合用量：
  - requests、input、output（含 reasoning）、cache、total tokens、approx cost；
  - 数字沿用现有 `format_count`/`format_cost` 紧凑格式；
  - 分组标题标明 provider 名与粒度，例如 `Usage (provider: anyrouter)`；
  - 保留"approx official-equivalent price"的成本口径提示（muted 单行）。
- 状态如实呈现，不 panic、不回退到整页错误：
  - 加载中 → `loading...`；
  - profile 未设置 `provider` → 提示未归因（如 `no provider label — usage
    unattributed`）；
  - provider 有值但无对应行 → `no usage recorded`；
  - llmusage schema/feature 不支持 → 单行 Unsupported 原因；
  - 查询错误 → 单行错误摘要。
- 数据加载不得阻塞渲染循环：进入首个 profile tab 时后台加载一次数据集，
  `r`（Reload）同时刷新 profiles 与用量数据集；选中项变化只做内存查找，不发
  新查询。
- 下线独立 Usage tab：
  - `App` 不再创建 `TabVariant::Usage` synthetic tab，移除相关按键/鼠标/渲染
    路由分支；
  - `TuiTabId` 默认顺序变为 5 项；**保留** `Usage` 枚举变体以便旧 `tui.toml`
    可解析，加载时过滤并告警忽略，不得因此回退默认顺序丢失用户自定义排序；
  - 含 `usage` 的旧 6 项配置与新 5 项配置都必须通过校验。
- 用量 SQL 仍只存在于 `crates/ccr-usage`；TUI 只消费 `TaggedProviderBreakdown`，
  不引入 per-surface 影子行结构（llmusage 适配器契约红线）。

## Acceptance Criteria

- [x] Wide/Standard/Compact 三种视口下，Claude 与 Codex profile 详情均出现
      Usage 分组；选中不同 profile 时数字跟随其 provider 变化（单测以注入
      loader 驱动状态断言详情行内容）。
- [x] 六种状态（loading / 无 provider / 无记录 / Unsupported / Error / 命中）
      各有单测覆盖详情行渲染文案。
- [x] Tab 循环不再出现 Usage 页；`tab_config_id` 无 Usage 映射；
      `default_order_selects_codex_profile_first` 等既有 tab 顺序回归测试更新后
      通过。
- [x] 旧版含 `usage` 的 `tui.toml` 加载成功：自定义顺序保留、`usage` 被忽略并
      记录 warn 日志（新增 ccr-config 单测）。
- [x] `r` 刷新后用量状态机回到 Loading 并在数据返回后更新（复用现有注入 loader
      异步测试模式）。
- [x] `cargo test -p ccr-tui -- --test-threads=1`、`cargo test -p ccr-config
      -- --test-threads=1`、`cargo test -p ccr-usage`、`just fmt-check`、
      `just lint-strict` 全绿。
- [x] `rg 'usage_bucket_30m' --type rust` 的 SQL 命中仍仅在 `crates/ccr-usage`。

## Out of Scope

- 时间窗过滤（今日/7 天/30 天）——MVP 固定 all-time，与现 Usage tab 口径一致。
- profile 级（而非 provider 级）归因、历史数据回填。
- ccr-ui 桌面端与 llmusage CLI 行为改动。
- 列表页（非详情）内联用量列。

## Open Questions

- Usage 分组在详情中的位置：默认放在 Activity 之后（最后一组）。若用户希望
  提前到 Routing/Auth 之前，属一行常量调整，实现时可再确认。
