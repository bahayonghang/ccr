# 技术设计：per-tab 选中状态

## 方案选择

两种实现路径：

- **方案 A（字段下沉）**：把 `selected_index` / `current_page` / `selected_profile_name` 从 `App` 移入 `PlatformTab`，所有 `self.selected_index` 等读写点改为 `self.tabs[self.active_tab].xxx`。
  - 缺点：约 41+ 处读写点改动，且 `self.tabs[self.active_tab].field` 与其它 `self` 借用频繁冲突（borrow checker 噩梦），风险高。
- **方案 B（per-tab 快照，采用）**：`App` 保留这三个字段作为「当前活动 tab 的工作副本」（渲染与按键继续读写它，零改动），在 `PlatformTab` 上新增 `saved_selection` 快照。切 tab 时把旧 tab 工作副本存入其快照、从新 tab 快照恢复。
  - 外部可观察行为与方案 A **完全一致**（每个 tab 独立记忆 + 首次定位 is_current），但改动面集中在「切 tab」一条路径，surgical、借用友好。

采用**方案 B**。

## 数据结构

`crates/ccr-tui/src/tui/app.rs`：

```rust
/// 单个 tab 的选中状态快照（会话级，per-tab 记忆）
#[derive(Clone)]
struct TabSelection {
    selected_index: usize,
    current_page: usize,
    selected_profile_name: Option<String>,
}
```

`PlatformTab` 新增字段：

```rust
pub struct PlatformTab {
    // ...既有字段...
    /// 离开该 tab 时保存的选中快照；None = 从未访问过
    saved_selection: Option<TabSelection>,
}
```

`App` 上的 `selected_index` / `current_page` / `selected_profile_name` 语义改为「当前活动 tab 的工作副本」，字段定义不变。`page_size` 保持全局。

## 新增方法（app.rs）

```rust
/// 把光标定位到当前 tab 的已启用项（is_current）；无则定位第 0 项。所有平台统一。
fn focus_current_profile(&mut self) {
    let total = self.current_profiles().len();
    if total == 0 {
        self.current_page = 0;
        self.selected_index = 0;
        self.selected_profile_name = None;
        self.reset_profile_detail_scroll();
        return;
    }
    let target = self.current_profile_global_index().unwrap_or(0);
    self.current_page = page_for_index(target, self.page_size);
    self.selected_index = super::pagination::index_in_page(target, self.page_size);
    self.remember_selected_profile();
    self.reset_profile_detail_scroll();
}

/// 离开 tab：把工作副本写入当前 tab 的快照
fn save_active_tab_selection(&mut self) {
    self.remember_selected_profile();
    self.tabs[self.active_tab].saved_selection = Some(TabSelection {
        selected_index: self.selected_index,
        current_page: self.current_page,
        selected_profile_name: self.selected_profile_name.clone(),
    });
}

/// 进入 tab：有快照则恢复并按 name 对齐（防 reload 越界）；无快照则定位 is_current
fn restore_active_tab_selection(&mut self) {
    match self.tabs[self.active_tab].saved_selection.clone() {
        Some(saved) => {
            self.current_page = saved.current_page;
            self.selected_index = saved.selected_index;
            self.selected_profile_name = saved.selected_profile_name;
            self.align_selection_by_name(); // name 命中→定位；未命中→clamp
        }
        None => self.focus_current_profile(),
    }
}

/// 按 name 对齐光标，无平台差异；故意不复用 sync_selection_to_profile_name，
/// 以免其 Codex 分支的 is_current 抢占已恢复的快照位置。
fn align_selection_by_name(&mut self) {
    // total==0 → 清零；否则 name 命中 position → 否则 selected_profile_global_index() → clamp
}
```

## 改动点

1. **切 tab 三处**（`dispatch`，app.rs:585/594/607 的 `NextTab` / `PrevTab` / `SwitchTab`）：
   把 `self.remember_selected_profile(); … self.sync_selection_to_profile_name();`
   替换为 `self.save_active_tab_selection(); … self.restore_active_tab_selection();`
   （切 `active_tab` 的算法不变，`reset_profile_detail_scroll` + `notify_tab_activated` 保留。）

2. **构造首次定位**（`with_task_executor`，app.rs:492）：
   `app.sync_selection_to_profile_name();` → `app.focus_current_profile();`
   - `active_tab=0` 默认是 Codex Profile，旧逻辑本就 is_current 优先，行为不变；
   - 若 tab order 把 Claude 放首位，`focus_current_profile` 更正确（旧逻辑会落第 0 项）。

3. **`PlatformTab` 构造点补字段**：所有 `PlatformTab { … }` 字面量补 `saved_selection: None`（生产 7 处 + 测试 7 处 + ui.rs 2 处，共 16 处，用 perl 在每个 `instance:` 字段行后批量插入）。

4. **`notify_tab_activated` 不再为 profile tab 调 sync**：原本切到 profile tab 会再调一次 `sync_selection_to_profile_name()`，对 Codex 走 is_current 优先，会**覆盖** restore 恢复的快照（Codex tab 永远跳回已启用项）。改为切 profile tab 时直接 return（定位已由 restore/focus 完成）。auth 分支不变；`with_*_auth_tab` 仅走 auth 分支，不受影响。

## 保持不变（不要动）

- `sync_selection_to_profile_name()` 本体：仍服务 `reload_profiles()`、`sync_profile_page_size()`、`apply` 后刷新等路径，作用于当前活动 tab 的工作副本。其 Codex/else 分支差异在 reload 语义下保持现状（仅切 tab 路径改用 `align_selection_by_name`）。
- `page_size` 全局、分页 helper、鼠标 `Rect` 缓存。

## 边界与正确性

- **reload 后越界**：非活动 tab 的 `profiles` 在 `reload_profiles` 中被重建，其 `saved_selection.selected_index/current_page` 可能越界。restore 走 `align_selection_by_name()`，按 name 重定位并 clamp，安全。
- **为什么 restore 不复用 sync**：`sync_selection_to_profile_name()` 的 Codex 分支优先 `is_current`，会让 Codex tab 的快照恢复失效（永远跳回已启用项），故 restore 用纯 name 优先的 `align_selection_by_name()`，并移除 `notify_tab_activated` 的二次 sync。
- **当前活动 tab 的 saved_selection**：只在「离开时写、进入时读」，停留在某 tab 期间读的是工作副本，二者不会同时被读，下次离开用最新工作副本覆盖，保持一致。
- **apply（空格）后**：仍停留同 tab、`reload_profiles` 刷新工作副本，不触发 save/restore，无影响。
- **空 profiles tab**：`focus_current_profile` 与 `restore` 均处理 total==0 分支。

## 兼容性 / 回滚

- 纯进程内状态结构调整，无配置/DB/IPC/持久化变更，无跨 crate 影响。
- 回滚 = 还原 `crates/ccr-tui/src/tui/app.rs`（及 ui.rs 测试构造点）。

## 验证

- `just fmt-check`
- `cargo test -p ccr-tui -- --test-threads=1`
- `just lint-strict`
