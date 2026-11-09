# design — TUI 用量统计跟随 profile

## 边界与所有权

```
crates/ccr-usage            ← 唯一 SQL/投影所有者（不改）
crates/ccr-tui
  src/tui/usage/app.rs      ← UsageApp 保留：从 "Usage tab 的 TuiApp" 降级为
                              "App 持有的用量数据引擎"（loader seam + 状态机不变，
                              删除 TuiApp impl 与 handle_key/render）
  src/tui/usage/ui.rs       ← 删除整页渲染；保留/下沉 format_count、format_cost、
                              truncate 等格式化助手供详情分组复用
  src/tui/app.rs            ← 删除 synthetic Usage tab 及全部 is_usage_tab 分支；
                              usage_app 字段保留（数据引擎），激活时机改为
                              "进入任一 Profile tab 时 ensure + on_activated"
  src/tui/ui.rs             ← codex/claude/generic 详情行构造函数追加 Usage 分组
crates/ccr-config
  src/managers/tui_config.rs ← DEFAULT_TAB_ORDER 缩为 5 项；Usage 变体保留
                               （deprecated, parse-tolerant）；validate/load 过滤
```

## 数据流

```
进入 Profile tab（首次）
  → App.ensure_usage_engine() → UsageApp::on_activated()（沿用 1-tick 延迟）
  → spawn_blocking(loader) → provider_breakdown_by_source([Claude,Codex], default)
  → mpsc → on_tick drain → UsageLoadState::Loaded(UsageDataset)

渲染详情（每帧，纯内存）
  profile.provider + tab.platform(source)
  → dataset.platform_rows(source).find(|r| r.breakdown.provider == profile.provider)
  → usage_detail_lines(state, provider, row) → Vec<Line>
```

关键决策：

1. **一次加载、内存查找**。数据集就是现 Usage tab 的同一次查询结果（每平台一行
   ×provider），选中项切换零查询。`r` Reload 时 `usage_app.refresh()` 一并触发。
2. **UsageApp 降级而非重写**。状态机、loader seam、异步测试基建全部复用；只删
   TuiApp impl。避免第二套加载状态语义。
3. **匹配语义**：`profile.provider: Option<String>` 与
   `breakdown.provider: Option<String>` 按 `Option<&str>` 精确相等；
   `None` 不匹配 unattributed 桶（该桶混入全部历史未归因量，展示会误导），而是
   显式提示 `no provider label — usage unattributed`。
4. **detail lines 函数签名**：三个 `*_profile_detail_lines` 追加参数
   `usage: Option<&UsageLoadState>`（None = 引擎未初始化，渲染 `loading...`），
   分组构造收敛到单个 `usage_section_lines(platform, provider, state) -> Vec<Line>`
   避免三份复制。

## Usage 分组渲染规格

```
▌ Usage (provider: anyrouter)
requests        56.2K
input           9415.6M
output          133.7M
cache           17444.1M
total           19382.1M
approx_cost     $24571
note            approx official-equivalent · provider-level (all-time)
```

- 命中行走上表全量字段；Compact 视口合并为 3 行（requests / tokens in·out·cache /
  cost）以控制详情高度。
- 非命中状态渲染分组头 + 单行状态文案（muted/warning/error 风格复用
  `detail_value_style` 语义色）。

## TuiTabId 兼容策略（回滚点）

- enum 保留 `Usage` 变体 + `#[doc(hidden)]` 注明 deprecated；`as_str` 保留。
- `DEFAULT_TAB_ORDER: [TuiTabId; 5]`（去掉 Usage）。
- `validate_tab_order`：先 `filter(|id| *id != Usage)` 再校验"5 项恰好各出现一次"；
  含 usage 时 `tracing::warn!` 提示已忽略。
- `load()` 返回前同样过滤，保证 App 拿到的顺序永不含 Usage。
- 回滚：恢复 6 项常量 + 撤销过滤即可，无数据迁移。

## 权衡记录

- 不做 per-selection SQL（每次选中查询单 provider）：查询面已有一次性全量调用，
  行数 = provider 数（个位数~十位数），内存查找足够；per-selection 查询增加
  异步竞态面（快速 j/k 时任务风暴）。
- 不把数据集塞进 `PlatformTab`：Claude/Codex 共享同一数据集，App 级单例即可。
- 不新增 `ccr-usage` API：`provider_breakdown_by_source` 已满足；时间窗过滤
  （QueryFilter.since/until）留给后续任务。

## 兼容与测试面

- 旧 `tui.toml`（6 项含 usage / 自定义顺序）→ 顺序保留、usage 忽略（新单测）。
- `crates/ccr-tui` 既有 usage tab 测试：状态机/分类测试保留（引擎层不变），
  tab 路由与整页渲染测试删除，替换为详情分组渲染测试。
- 验证命令见 implement.md；额外跑 `cargo test -p ccr -- --test-threads=1`
  （guidelines：binary/TUI feature surface 变化时）。
