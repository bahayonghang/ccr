# Implement: Profiles 页面重构（父任务 · 集成与最终验收）

父任务不直接承担实现，承担：子任务排序、跨子任务验收、旧代码路径清除、最终集成复验。

## 执行顺序（强依赖）

1. `07-29-profiles-shared-layer` — 纯新增式共享层（新文件 + 可选新 prop，默认旧行为）。
2. `07-29-profiles-claude-page` — Claude 页接入新 API。
3. `07-29-profiles-codex-page` — Codex 页接入新 API。
4. 本文件的集成步骤（步骤 4 开始）— 只有前两页都迁移完成后才能执行。

子任务归档顺序 = 上述顺序；一个子任务归档后才 `task.py start` 下一个。

## 集成检查清单

4. [x] **旧路径清除**：删除共享组件中仅为兼容保留的旧 props 分支/旧槽位（QuickRail 旧数据源、StatStrip Last Write/sparkline、Toolbar 旧平铺筛选、ContextRail 旧 descriptor 面板）；删除视图侧遗留的内联 `ProfilesSection` 与 `.cp-list-head` 重复 markup 的最后残余。
5. [x] **死代码终审**：对照 `research/current-state-analysis.md` 的死代码清单逐项确认删除或未删原因（未删项写入 spec 或后续任务）。
6. [x] **跨子任务验收标准 1–11 逐条核验**（见 `prd.md`），每条附证据（代码位置 / 测试 / 截图）。证据见 `acceptance.md`。
7. [x] **截图走查协议执行**（`design.md` §8.1）：两页 × 暗/亮 × 2543px/1280px，检查 accent 占比 <10%、首屏层级、1280px 右栏显隐。2026-07-30 由用户在真实 Tauri 运行态人工确认无问题。
8. [x] **最终全量门禁**：`cd ccr-ui && bun run test` → `just ui-check` →（跨模块影响时）`just ci`。本次仅触及 `ccr-ui` 前端与任务/spec 文档，执行 `just ui-check` 作为最终门禁。
9. [x] **impeccable 复评**：对两页重跑 detector + 人工走查，对比 `ccr-ui/.impeccable/critique/` 快照确认 P0/P1 关闭。复评 34/40，P0=0，P1=0；73 条 detector 记录均为字阶/圆角 advisory，保留 degraded 声明与退出码说明。
10. [x] Phase 3.3 spec 更新：0.75rem 字阶扩展点、QuickSwitch 钉选持久化约定、弹层行为契约登记到 `.trellis/spec/ccr-ui/frontend/`。

## 回滚点

- 步骤 4 前：任何子任务可独立回滚，共享层新增物无害留置。
- 步骤 4（旧路径清除）：单独一个 commit，回滚 = revert 该 commit。
- 步骤 4 之后发现共享层缺陷：按缺陷面回滚对应平台页接入，而非回滚共享层。

## 完成定义

- 验收标准 1–11 全部有证据；`just ui-check` 绿； critique 复评 P0/P1 关闭；三个子任务已归档。
