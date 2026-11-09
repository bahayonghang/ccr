# 技术设计：首页右侧栏

## 1. 改动范围

| 文件                                                            | 改动                                |
| --------------------------------------------------------------- | ----------------------------------- |
| `ccr-ui/src/features/usage/dashboard/DashboardNextActions.tsx`  | 首条强调态；其余静默态              |
| `ccr-ui/src/features/usage/styles/dashboard-next-actions.css`   | 版式重写                            |
| `ccr-ui/src/features/usage/dashboard/DashboardSignalStream.tsx` | 行版式改写；筛选、channel、聚合保留 |
| `ccr-ui/src/features/usage/styles/dashboard-signal-stream.css`  | 版式重写                            |
| `ccr-ui/tests/dashboard-signal-stream.smoke.test.tsx`           | 新增或扩展：筛选、计数口径、聚合    |

props 契约不变：`DashboardNextActions` 的 `actions` / `showOnboarding`；
`DashboardSignalStream` 的 `entries` / `limit` / `className`。
不改 `DashboardView.tsx`、`dashboard-view.css`（归 `08-25-home-runtime-layout`）。
不改 `useDashboardSignals`。

测试落点先查 `rg -l 'DashboardSignalStream' ccr-ui/tests`，有则扩展，无则新建，落点写进本表。

## 2. 事件流：扩充而非替换

设计稿画的是三列行（时间戳 / 状态点 / 文本）。现有组件比设计稿多三样东西，全部保留：

| 现有能力                                    | 设计稿   | 处置                                           |
| ------------------------------------------- | -------- | ---------------------------------------------- |
| `all` / `warn` / `error` 筛选（标签带计数） | 无       | 保留。放在标题行右侧，与设计稿的标题行布局合并 |
| `channel` 列                                | 无       | 保留。行栅格由三列扩为四列                     |
| 相邻聚合 `×N`                               | 无       | 保留。`×N` 徽标跟在文本后                      |
| 空态 CTA + 页脚链接                         | 只有页脚 | 两者都保留                                     |

行栅格：

```
[时间戳 mono] [状态点] [channel] [文本 ······ ×N]
   auto        auto      auto      1fr
```

窄容器下 `channel` 列可折叠为 `title` 提示，但不删除该信息。是否折叠由实施时的实际宽度决定，
折叠也要保证 channel 可通过 `title` 获取。

## 3. 计数口径：固定现状，不改

现有实现：

```
aggregatedEntries = 按时间倒序 → 相邻同 message+channel+level 合并（累加 count）
标题计数           = aggregatedEntries 上按各档 matchesFilter 计数   ← 聚合后、筛选前、截断前
可见行             = aggregatedEntries.filter(当前档).slice(0, limit)  ← limit 默认 6
```

`matchesFilter` 的语义：

- `all` → 全部
- `warn` → `level === 'warn' || level === 'error'`（含 error，不是只看 warn）
- `error` → `level === 'error'`

**「标题计数 ≠ 可见行数」是既有的正确行为**，由聚合与 `limit` 截断造成。
上一版 PRD 的 AC3「计数与列表一致」无法唯一判定，已替换为 AC4 的口径断言。

本任务不改这套逻辑，只把它固定进测试。

## 4. 「下一步」强调态

首条用 accent 边框 + accent tint 底 + accent 图标，其余静默。

accent tint 的取值来源以令牌子任务 `research/token-name-delta.md` 的结论为准：

- 若该文件选定「选项 A」，accent 底色用 `rgb(var(--color-accent-primary-rgb) / 12%)`——
  这个形式随自定义强调色自动跟随。
- 若选定「选项 B」，用新增的 `--color-accent-tint`。

**不在本文件写死令牌名。** 开工前读那份结论。

warning / danger 的 tint 同理，按该文件的结论取名。

## 5. 非颜色可辨（R7）

三档除色点外，各配一个图标与一个可读文本（`aria-label` 或可见标签）。
灰度模拟下依靠图标形状与文本判断档位。

`channel` 列本身也是非颜色信息，保留它同时服务 R7。

## 6. 测试

| 断言     | 内容                                                                             |
| -------- | -------------------------------------------------------------------------------- |
| 筛选切换 | 点 `warn` 档后，可见行含 `error` 级条目（AC3）                                   |
| 计数口径 | 构造 > `limit` 条数据，断言标题计数等于聚合后筛选前的条数，且大于可见行数（AC4） |
| 聚合     | 构造相邻重复条目，断言渲染出 `×N` 且合并为一行（AC5）                            |
| 空态     | `entries` 为空时 CTA 可见（AC6）                                                 |

## 7. 回滚

```bash
git checkout -- ccr-ui/src/features/usage/dashboard/DashboardNextActions.tsx \
  ccr-ui/src/features/usage/dashboard/DashboardSignalStream.tsx \
  ccr-ui/src/features/usage/styles/dashboard-next-actions.css \
  ccr-ui/src/features/usage/styles/dashboard-signal-stream.css \
  ccr-ui/tests/
```

两块互相独立，可单独回滚其中一块。
</content>
