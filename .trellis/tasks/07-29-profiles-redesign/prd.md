# PRD: Profiles 页面重构（Claude Code / Codex）

## 背景与问题

Claude Code Profiles（`ccr-ui/src/views/ClaudeCodeProfilesView.vue`）与 Codex Profiles（`ccr-ui/src/views/CodexProfilesView.vue`）两页共享 `ccr-ui/src/components/profiles/` 八件组件族。完整现状分析与设计评审已沉淀至 `research/current-state-analysis.md`，评审快照存于 `ccr-ui/.impeccable/critique/`（Nielsen 23/40，认知负荷 8 项清单失败 6 项）。

核心缺陷（按优先级）：

- **P0 快速切换栏失控**：27 个 chip 四行平铺、无分组；⌘/Ctrl+1-9 编号随筛选/排序结果漂移（`useProfilesHotkeys.ts` 按显示顺序惰性求值），快捷键语义不稳定。
- **P1 决策关键时刻信息缺席**：卡片 base_url 截断、model 显示 `—`；apply/delete 确认框不含将写入的配置差异。
- **P1 切换路径与当前态冗余**：四条切换路径并存（chip 栏 / 卡片 Apply / 数字键 / ⌘K）；当前 profile 同时出现在统计条、chip 高亮、右栏面板、卡片徽章。
- **P2 筛选器堆叠**：搜索 + 状态 pill + 标签 pill + provider 下拉 + 排序五类筛选同时裸露，无组合反馈；右栏 tag cloud 只读且与工具栏重复。
- **P3 视觉语言漂移**：卡片视图（Tailwind + provider 色）与列表视图（`--cp-*` token）两套语言；编辑器模态自带独立 `--editor-*` token 体系；兄弟页统计格不同构；72 处 px 字面字号偏离 rem 字阶。

## 用户已确认的方向（2026-07-29）

1. **范围**：含编辑器模态一起重构；不动数据层与 API。
2. **快捷栏**：瘦身为「钉选 + 最近使用」≤8 个 chip，编号绑定钉选顺序不再漂移；其余 profile 走 ⌘K 面板；**快捷键必须适配 Windows**（Ctrl 修饰键 + 提示文案按平台显示 Ctrl/⌘）。
3. **Apply 交互**：保留确认框，但框内展示「当前 → 目标」的 base_url / model / auth 三行 diff；delete 确认框内联备份信息。
4. **任务结构**：父任务 + 3 个子任务。

## 目标

两页收敛为同一套「编辑控制室」信息架构：清爽、现代、专家密度保留但不再平铺，符合 `ccr-ui/PRODUCT.md`（Operate 模式、Truthful operational state、One visual language）与 `ccr-ui/DESIGN.md`（Accent Scarcity Rule、rem 字阶）。

## 非目标

- 不改 Tauri/Rust 后端与 `api/domains/*` 接口形状。
- 不改命令面板（⌘K）的既有交互范式，仅调整其入口与内容来源。
- 不引入新依赖；不新增设计 token 体系之外的视觉语言。
- 备份/回滚机制本身不重建，仅在 UI 中暴露其存在。

## 任务地图

| 子任务 | 交付物 | 依赖 |
|--------|--------|------|
| `07-29-profiles-shared-layer` | 重构后的共享组件族（QuickRail / StatStrip / Toolbar / ContextRail→预览检查器 / 确认 diff 框 / 编辑器外壳 token 统一 / hotkeys 稳定编号） | 无，先做 |
| `07-29-profiles-claude-page` | Claude 页接入新组件层 + 编辑器抽取为独立模态组件 + 页面瘦身 | 依赖 shared-layer |
| `07-29-profiles-codex-page` | Codex 页接入新组件层 + 编辑器模态对齐 + 页面瘦身 | 依赖 shared-layer |

## 跨子任务验收标准

1. 两页布局骨架完全一致（Header / StatStrip 同槽位 schema / QuickRail / Toolbar / 主列表 / 预览右栏），仅平台特色槽内容与 i18n 前缀不同。
2. 快速切换栏任何时候 ≤8 个 chip；数字编号**只绑定钉选列表**（最多 8 个），仅随用户钉选/取消钉选操作变化；最近使用 chip 展示但不编号；筛选/排序/搜索/Apply 行为均不改变编号指向。
3. Apply 确认框展示当前与目标 profile 的 base_url、model、auth 差异；Delete 确认框含备份提示，且备份表述必须与真实行为一致：快照写入 `~/.ccr/backups/{platform}/`（`write_guarded` + `BackupPolicy::Dir`，`crates/ccr-config/src/platforms/base.rs:545`），当前无 UI 恢复入口，文案不得承诺「从 Sync 页恢复」（Sync 同步的是 `~/.ccr/platforms/`，与本地快照无关）。
4. 当前 profile 的**字段详情**（base_url/model/auth 等）首屏只在 Inspector 出现一次；**当前态轻量标记**（名称/高亮，不含字段详情）允许且仅允许出现在三处：StatStrip Current 槽、QuickRail 活动 chip、列表行 Current 徽章。Inspector 默认展示当前 profile，hover/聚焦其他 profile 时切换为该 profile 的预览 + 与当前的差异高亮。
5. 工具栏裸露的筛选控件 ≤3 个（搜索、状态 pill 组、Filters 按钮）；Filters 按钮显示生效筛选数徽标。
6. 卡片与列表视图统一消费 `--cp-*` token；编辑器模态不再使用独立 `--editor-*` 平行体系；新增/改动样式中无 px 字面字号（沿用 rem 字阶）。
7. Windows 下快捷键提示显示 `Ctrl`，macOS 显示 `⌘`；`+ number key` 类模糊提示全部消除。
8. `cd ccr-ui && bun run test`、`bun run typecheck`（或项目既有等价检查）通过；`just ui-check` 通过。
9. i18n：zh-CN / en-US 双语言键同步，新增键无硬编码回退文案（既有 `translateWithFallback` 硬编码中文回退在本次触及的组件中一并清理）。
10. 视觉验收按固定协议执行（见 `design.md` §8 验证协议）：固定 fixture、路由、视口、运行环境下的暗/亮双主题截图走查，陶土 accent 占比符合 Accent Scarcity Rule（<10%）。
11. 新增自动化用例覆盖：`useProfilesQuickSwitch` 持久化与 stale 名称清理、平台修饰键提示（基于 `getClientPlatform()` mock）、QuickRail roving tabindex、profile rename/delete/disable 后钉选与最近列表的清理、Codex `env_key` 仅在 `provider_env_key` 模式序列化。
