# implement — TUI 用量统计跟随 profile

前置：先读 `.trellis/spec/ccr-tui/backend/backend-guidelines.md` 与
`.trellis/spec/ccr/backend/llmusage-provider-adapter.md`（implement.jsonl 已列）。

## 步骤（按序，每步可验证）

1. **ccr-config：TuiTabId 兼容收缩**
   - `tui_config.rs`：`DEFAULT_TAB_ORDER` 缩为 5 项；`validate_tab_order` 与
     `load()` 过滤 `Usage` 并 warn；补单测：旧 6 项配置（含自定义顺序）加载后
     顺序保留且不含 usage、新 5 项配置通过、重复/缺失仍报错。
   - 验证：`cargo test -p ccr-config -- --test-threads=1`

2. **ccr-tui：UsageApp 降级为数据引擎**
   - `usage/app.rs`：删除 `impl TuiApp for UsageApp`（handle_key/render）；保留
     `on_activated`/`refresh`/`on_tick 等价的 drain+delay 逻辑`（改为供 App 的
     `on_tick` 调用的普通方法，如 `tick(&mut self) -> bool`）。
   - `usage/ui.rs`：删除整页 draw/draw_embedded/draw_loading_placeholder/footer；
     `format_count`/`format_cost`/`truncate` 下沉为 `pub(crate)`。
   - 验证：`cargo test -p ccr-tui -- --test-threads=1`（状态机测试应原样通过）

3. **ccr-tui：拆除 Usage tab 路由**
   - `app.rs`：删除 synthetic tab push、`TabVariant::Usage`、`is_usage_tab` 全部
     分支、`tab_config_id` Usage 映射、`ensure_usage_app` 的 tab 语义（改名为
     `ensure_usage_engine`，进入 Profile tab 时调用）；`usage_error` 字段跟随
     调整。`on_tick` 中接入引擎 `tick()`。Reload action 追加
     `usage_engine.refresh()`。
   - `ui.rs`：删除 usage 分支（`draw` 中 2 处）。
   - 更新既有 tab 顺序/循环回归测试。
   - 验证：`cargo test -p ccr-tui -- --test-threads=1`

4. **ccr-tui：详情面板 Usage 分组**
   - `ui.rs`：新增 `usage_section_lines(platform, provider: Option<&str>,
state: Option<&UsageLoadState>) -> Vec<Line>`；六态文案 + 命中全字段；
     Compact 视口 3 行紧凑变体；三个 `*_profile_detail_lines` 追加调用（分组置于
     Activity 之后）。
   - `render_profile_details` 把 `app.usage_app` 状态与选中 profile 的 provider
     传入。
   - 单测：注入 loader 构造六态，断言分组行文案与配色语义（沿用
     `injected_loader_*` 测试模式）；provider 匹配/None 不匹配 unattributed。
   - 验证：`cargo test -p ccr-tui -- --test-threads=1`

5. **全量回归**
   - `just fmt-check` → `cargo test -p ccr-usage` → `cargo test -p ccr --
--test-threads=1` → `just lint-strict`
   - `rg 'usage_bucket_30m' --type rust` 确认 SQL 仅在 crates/ccr-usage。
   - 手工冒烟：`cargo run -p ccr -- tui`（含中文描述与无 provider 的 profile 各
     选一个），核对六态与 Tab 循环无 Usage 页。

## Review gates

- 步骤 3 完成后自查：`rg 'is_usage_tab|TabVariant::Usage' crates/` 应无命中。
- 步骤 4 完成后跑 trellis-check（spec 对齐：adapter 契约第 99 条 TUI 约束）。

## 回滚点

- 每步独立 commit；步骤 1 与 2-4 可分别 revert。
- Usage tab 整体恢复 = revert 步骤 2-4 的 commits + 恢复 6 项 DEFAULT_TAB_ORDER。
