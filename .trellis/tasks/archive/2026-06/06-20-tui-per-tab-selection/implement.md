# 执行计划：per-tab 选中状态

仅改动 `crates/ccr-tui/src/tui/app.rs`（含其内联测试）与 `crates/ccr-tui/src/tui/ui.rs`（仅测试构造点）。

## 步骤

1. **定义 `TabSelection` 并扩展 `PlatformTab`** → verify: `cargo build -p ccr-tui` 报缺字段错误，定位全部构造点。
   - 在 app.rs `PlatformTab` 定义附近新增 `struct TabSelection { selected_index, current_page, selected_profile_name }`（`#[derive(Clone)]`）。
   - `PlatformTab` 加 `saved_selection: Option<TabSelection>`。

2. **补全所有 `PlatformTab` 构造点** `saved_selection: None` → verify: `cargo build -p ccr-tui` 通过。
   - 生产：app.rs:364/376/391/404/417/441/453。
   - 测试：app.rs:1288/1325/1591/1636/1648/1728/1740；ui.rs:1302/1339。

3. **新增三个方法**：`focus_current_profile`、`save_active_tab_selection`、`restore_active_tab_selection`（按 design.md 实现）→ verify: `cargo build -p ccr-tui` 通过。

4. **改切 tab 路径**：`dispatch` 的 `NextTab` / `PrevTab` / `SwitchTab` 用 save/restore 替换 remember+sync → verify: 手工核对三处一致。

5. **改构造首次定位**：app.rs:492 `sync_selection_to_profile_name()` → `focus_current_profile()` → verify: 构建通过。

6. **新增单测**（app.rs `#[cfg(test)]`）：
   - `first_enter_tab_focuses_current_profile`：含两个 profile tab，目标 tab 的 `is_current` 不在 index 0；`dispatch(SwitchTab/NextTab)` 后断言 `selected_profile()` 命中 is_current。
   - `revisiting_tab_restores_saved_selection`：进入 tab → 若干 `SelectNext` → 切走 → 切回，断言回到离开前的 `selected_index`/`current_page`。
   - `tabs_keep_independent_selection`：两 tab 各自选不同项，来回切换互不串扰。
     → verify: `cargo test -p ccr-tui -- --test-threads=1` 全绿。

7. **质量门**：`just fmt-check` → `cargo test -p ccr-tui -- --test-threads=1` → `just lint-strict` → verify: 全部通过。

## Review Gates

- 步骤 4 后人工确认：save 在切 active_tab **之前**、restore 在**之后**。
- 步骤 6 后确认：测试真正经由 `dispatch(Action::…)` 驱动（覆盖真实切 tab 路径），而非直接调内部方法。

## Rollback

任一步阻塞：`git checkout -- crates/ccr-tui/src/tui/app.rs crates/ccr-tui/src/tui/ui.rs`。
