---
target: Claude Code / Codex Profiles 管理页
total_score: 23
max_score: 40
na_heuristics: 
p0_count: 1
p1_count: 2
timestamp: 2026-07-29T09-29-59Z
slug: ccr-ui-src-views-claudecodeprofilesview-vue
---
# Critique: Claude Code / Codex Profiles 管理页

Method: dual-agent (A: agent-2 · B: agent-3)

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3 | 状态标记齐全；`LAST WRITE —` 空值占位、apply 进度反馈弱 |
| 2 | Match System / Real World | 2 | `Auth Split 0 · 27`、`Unspecified Provider`、base_url 截断为 `https…` |
| 3 | User Control and Freedom | 3 | 确认框齐备；apply 后无 undo、备份信息不进危险流程 |
| 4 | Consistency and Standards | 2 | 卡片视图(Tailwind/provider 色) vs 列表视图(--cp-*) 两套语言；兄弟页统计格漂移 |
| 5 | Error Prevention | 3 | 危险操作有确认；确认文案不含将写入的 base_url/model 差异 |
| 6 | Recognition Rather Than Recall | 2 | 辨认目标 profile 所需关键字段被截断或显示 `—` |
| 7 | Flexibility and Efficiency | 4 | ⌘K / ⌘1-9 / `/` / Esc 共享 composable，专家通道扎实 |
| 8 | Aesthetic and Minimalist Design | 1 | 27 chip + 11 pill + 头部 7 动作 + 右栏 4 区块同屏竞争；四条切换路径并存 |
| 9 | Error Recovery | 2 | Health Audit 存在但与卡片无联动；备份回滚路径不在出错现场 |
| 10 | Help and Documentation | 1 | 全部快捷键教学只有一行 `+ number key`，且未说明需 ⌘/Ctrl |
| **Total** | | **23/40** | **Acceptable 下沿 — 信息架构需大修** |

## Design Specificity Verdict

视觉皮肤是编辑式的（克制炭黑底、陶土色稀缺、全大写小标签），但信息架构是通用 SaaS admin 的（功能清单平铺：27 个同质 chip、11 个 pill、五区同屏竞争）。后者决定体验，判定为"滑向品类可互换"。

**Deterministic scan**：100 条发现 — 72 条 px 字面字号偏离 rem 字阶（重灾区 `ProfilesContextRail.vue` 18 处）、27 条 radius 偏阶（多为检测器刻度与 DESIGN.md 不一致的半误报）、1 条 width transition warning。硬编码颜色 / `!important` / 裸 inline style / 缺 aria 图标按钮零发现 —— 配色确实统一走了 CSS 变量，是正向信号。检测器盲区：业务逻辑重复与 IA 过载。

## Overall Impression

键盘专家层优秀、组件族复用真实，但页面被"功能平铺"支配。最大机会：把"27 chip 横栏 + 右栏档案柜"换成"窄快捷栏 + hover 预览 X 光右栏 + 有内容的确认 diff"三件套。

## What's Working

1. 键盘专家层真材实料：`useProfilesHotkeys.ts` 收敛 ⌘K/⌘1-9/`/`/Esc 为两页共享 composable，数字键切换也走确认。
2. 状态诚实：Enabled/Disabled 显式分组、当前态三通道标记、原始 TOML 编辑有门槛确认。
3. 八件共享组件族，Claude/Codex 两页同构，扩展第三个平台成本低。

## Priority Issues

- **[P0] 快速切换栏失控**：27 个 chip 四行平铺、纯字母序、无分组；⌘1-9 编号随筛选/排序结果漂移（`useProfilesHotkeys.ts:35` 惰性求值）。修复：chip 栏只放最近使用+钉选 ≤8 个，编号绑定钉选顺序；其余收进 ⌘K。
- **[P1] 决策关键时刻信息缺席**：卡片 base_url 截断、model 显示 `—`；apply/delete 确认框是泛泛一句话。修复：hover 浮层展示完整 host/model/auth；apply 确认框改为"当前 → 目标"三行 diff；delete 确认框内联备份信息。
- **[P1] 四条切换路径 + 当前 profile 三次重复**：chip 栏/卡片 Apply/⌘1-9/⌘K 并存；当前 profile 同时出现在统计条、chip 高亮、右栏面板、卡片徽章。修复：收敛为两路；右栏改为"目标预览"（hover 哪个预览哪个 + 与当前 diff）。
- **[P2] 筛选器堆叠无组合反馈**：搜索+4 状态 pill+7 标签 pill+provider 下拉+排序同时裸露，无"n 个筛选生效"徽标；右栏 tag cloud 只读且与工具栏重复。修复：标签/provider/排序收进 Filters 弹层；tag cloud 可点击写筛选。
- **[P3] 兄弟页与双视图漂移**：卡片/列表两套视觉语言；统计格不同构；Codex 卡片显示完整 URL 而 Claude 截断。修复：统一到 `--cp-*` token；统计条同一槽位 schema。

## Persona Red Flags

**Alex（键盘流专家）**：⌘1-9 编号随筛选漂移，肌肉记忆指向错误 profile；`agentrouter-github-73367` vs `-73368` 无法靠截断 URL 区分；无"切回上一个"的 recent 栈。

**Sam（屏幕阅读器/纯键盘）**：Tab 序约 108 个停靠点走完主区，无 skip link / roving tabindex；pill 选中态仅靠颜色区分；状态变更无 aria-live 播报证据。

## Minor Observations

- `LAST WRITE —` 空值占位格不如收起。
- Codex 卡片显示完整 base_url，Claude 截断 —— 同族字段策略不一致。
- `+ number key` 未说明需 ⌘/Ctrl 修饰键。
- Health Audit 26 项平铺，与具体 profile 卡片无联动。
- 检测器：72 条 px 字号应迁 rem 字阶（advisory）。

## Questions to Consider

1. 删掉 27-chip 横栏只留窄当前条 + ⌘K，Alex 真的会变慢吗？
2. 右栏从"当前档案柜"改成"hover 目标 X 光片 + diff"，能否成为页面信心引擎？
3. Apply 能否改为"应用即成功 + 3 秒可 Undo"而非每次模态打断？
