# 重构 UI 工程文档

## Goal

重构 `ccr-ui/docs/`，把分散的设计规范、实现计划、评审产物和历史方案整理为可维护的 UI 工程知识库，并与当前 Vue/Tauri 实现保持一致。

## Background

- 当前目录只有五份分散资料，没有入口、生命周期规则或自动化审计。
- 设计系统文档仍有长期价值，但引用了不存在的快照脚本。
- 同步改版资料已经实施；Claude Profiles dashboard 方案尚未按文档中的结构实施；VibeDeck 报告是时间点分析。

## Requirements

- 审计现有 `design-system/`、`plans/`、`spark/`、`superpowers/` 与 `artifacts/` 内容的有效性和归属。
- 以当前 `ccr-ui/src/`、`ccr-ui/src-tauri/`、测试和样式令牌为事实来源更新长期工程文档。
- 明确长期规范、当前计划、已完成/历史材料与生成产物的目录边界。
- 阶段性资料移动到 `archive/2026/`，由归档索引和 Markdown 状态说明区分“已实施”“未实施提案”和“时间点分析”。
- 不把内部设计/计划文档重复发布到根 VitePress 站；仅在确有用户价值时建立链接或提炼用户文档。
- 修复长期设计系统文档引用不存在的 `test:playwright:snapshots` 命令等已确认失真内容。
- 新增长期 UI 架构、文档维护和验证入口，基于 `ccr-ui/code_map.md`、路由、API facade、Tauri 命令注册和现有脚本。
- 增加轻量自动审计，验证目录生命周期、归档索引、相对链接和已知失真命令不会回归。

## Acceptance Criteria

- [x] `ccr-ui/docs/` 具有清晰入口和目录约定，读者能判断每份文档是否为现行规范。
- [x] 长期 UI 架构与设计系统文档对应当前组件、路由、状态管理、Tauri 边界和样式令牌。
- [x] 现有历史计划与 HTML 产物经过逐项处置，不再与现行规范混杂。
- [x] 已完成的同步改版资料与尚未实施的 Claude Profiles 方案具有不同且准确的状态标识。
- [x] 文档内链接和引用目标有效，Markdown 格式检查无新增问题。
- [x] 与根 `docs/` 的受众和内容边界有明确说明且无冲突。
- [x] `bun run docs:audit` 能独立通过，并纳入 UI 或根文档验证入口。

## Decision

- 采用保留归档方案，不删除已有阶段性计划和 HTML 报告。

## Out Of Scope

- 不实施归档中的 Claude Profiles dashboard 提案。
- 不修改 Vue/Tauri 产品行为、视觉样式或生成的路由快照产物。
