---
target: Claude Code / Codex Profiles 管理页
total_score: 34
max_score: 40
na_heuristics:
p0_count: 0
p1_count: 0
timestamp: 2026-07-30T01-49-38Z
slug: ccr-ui-src-views-claudecodeprofilesview-vue
---
# Critique: Claude Code / Codex Profiles 管理页

⚠️ DEGRADED: single-context (project AGENTS requires sequential execution)

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 4 | 当前态、总量、健康、行级 busy 与结果 toast 均有明确反馈 |
| 2 | Match System / Real World | 4 | Profiles、Provider、认证与备份文案符合目标专家用户语言 |
| 3 | User Control and Freedom | 3 | 返回、取消、清除筛选与 Esc 完整；本地备份仍无 UI 恢复入口 |
| 4 | Consistency and Standards | 4 | Claude/Codex 共用同一骨架、组件、token 与交互契约 |
| 5 | Error Prevention | 4 | Apply 展示 base_url/model/auth diff，Delete 明示真实备份位置 |
| 6 | Recognition Rather Than Recall | 3 | 预览检查器补足关键字段；空 QuickRail 时钉选入口仍依赖行级菜单发现 |
| 7 | Flexibility and Efficiency | 4 | Ctrl/⌘K、数字键稳定钉选、搜索快捷键、卡片/列表双视图完整 |
| 8 | Aesthetic and Minimalist Design | 3 | 主次层级与 accent 稀缺性成立；大量健康问题时右栏仍偏密 |
| 9 | Error Recovery | 3 | 健康问题可定位、错误保留上下文；备份恢复仍需外部路径操作 |
| 10 | Help and Documentation | 2 | 控件提示与快捷键文案清楚，但缺少就地恢复说明 |
| **Total** | | **34/40** | **Good** |

## Design Specificity Verdict

页面已经从通用 SaaS 管理页转为 CCR 专用的 Profiles 工作台。Claude 与 Codex 共享 Header / StatStrip / QuickRail / Toolbar / 主列表 / Inspector 骨架，平台差异只通过槽位与 descriptor 注入；陶土 accent 只落在当前态和主操作，符合产品的安静、精确、编辑式基调。

detector 对两个页面与 `components/profiles` 报告 73 条设计系统 advisory：56 条字阶、17 条圆角。移除 `ProfilesInspector` 的 `transition: width` 后，不再有布局动画或其他功能级规则。多数 `0.75rem` 命中是密集元数据的有意扩展点；其余非标准字号与圆角属于设计系统登记债务，不构成 P0/P1。

## Overall Impression

重构后的首屏层级清楚：标题与 Add 是行动入口，统计条建立全局状态，工具栏承担检索，卡片承担选择，Inspector 承担核对。最大剩余机会是进一步压缩异常配置很多时的健康审计密度，而不是继续调整主骨架。

## What's Working

1. 稳定钉选编号彻底与筛选、排序、搜索解耦，数字快捷键恢复可靠的肌肉记忆。
2. Apply/Delete 把关键差异与真实备份边界放回高风险决策现场，消除了原先的信息缺席。
3. 两页与双视图统一消费 `--cp-*` token，共享页面 CSS 让同构关系成为实现事实。

## Priority Issues

- **[P2] 健康审计在问题很多时偏密**：宽屏 Inspector 可一次出现二十余条问题，虽然支持定位，但扫描负担仍高。后续可按问题类型折叠或先展示前 N 条。
- **[P3] 空 QuickRail 的钉选发现性一般**：没有钉选/最近项时整栏合理隐藏，但首次钉选入口只在行级更多菜单中。后续可在命令面板搜索结果中同步暴露钉选动作。
- **[P3] DESIGN.md 刻度登记落后于实现**：detector 的 73 条结果均为字号/圆角 advisory。应先登记 Profiles 的 `0.75rem` 密集元数据字阶，再单独决定是否统一其余微型圆角。

## Persona Red Flags

**Alex（键盘流专家）**：原 P0 已关闭；数字键只命中最多 8 个稳定钉选，筛选和 Apply 不再改变映射。残余问题是首次钉选仍需发现行级菜单。

**Sam（屏幕阅读器/纯键盘）**：QuickRail 使用 roving tabindex，Inspector 有语义区域与 `aria-live`，弹层支持 Esc 和焦点返回。残余风险主要是健康列表很长时的 Tab 成本，而非不可达控件。

**CCR 配置维护者**：Apply 前可直接核对 base_url/model/auth，Delete 文案不再虚构 Sync 恢复入口；真实本地备份边界清晰。

## Minor Observations

- `0.75rem` 在 Profiles 中承担标签、元数据、键帽与审计行，是有意的密度层级，应登记而不是机械放大。
- Inspector 的分布洞察已折叠，健康审计仍保持展开，异常项很多时两者的渐进披露策略不完全一致。
- 当前没有 P0/P1；剩余项不阻断父任务归档。

## Questions to Consider

1. 健康审计是否应默认只展示最严重的 5 条，再按类型展开？
2. 钉选动作是否应同时进入命令面板，让空 QuickRail 的首次使用更容易发现？
