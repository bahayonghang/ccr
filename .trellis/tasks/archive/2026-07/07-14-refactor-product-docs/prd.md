# 重构产品文档站

## Goal

重构 `docs/` VitePress 文档站，使其面向当前 CCR 用户准确说明安装入口、CLI/TUI 工作流、配置、平台支持、桌面 UI 模块与项目架构。

## Background

- 当前 workspace 有 13 个 crate，现有 crate map 未完整覆盖。
- 顶层 CLI 有 34 个命令，命令参考缺少稳定的 `claude` 命令页面。
- 当前审计错误要求内部维护文件提供英文镜像，也没有验证完整命令页面覆盖。

## Requirements

- 从 `crates/` 的实际 crate、CLI 命令、配置类型和测试提取文档事实。
- 从 `ccr-ui/` 的路由、导航、Tauri 命令与用户界面提取桌面 UI 文档事实。
- 重新审查信息架构、导航与页面归属，合并重复或过时内容。
- 中文主页面与 `en/` 英文镜像保持活跃页面、导航和关键含义一致。
- 示例命令必须能在根 `justfile`、`docs/justfile`、包脚本或实际 CLI 中验证。
- 遵守 `docs/AGENTS.md`，不修改 `.vitepress/dist/`、`.vitepress/cache/` 或 `node_modules/`。
- 区分发布页面与仓库维护资料，避免要求 `AGENTS.md`、内部报告或维护 TODO 建立英文产品镜像。
- 文档审计应自动发现稳定 CLI 命令覆盖、语言镜像和导航目标的回归。
- 保持现有公开页面 URL；除新增缺失页面和删除无价值的维护 TODO 外，不做破坏性路径迁移。
- 历史 changelog 保持历史原貌；其他页面删除无法由当前实现验证的未来承诺、模板化示例和重复说明。

## Acceptance Criteria

- [x] 文档导航与页面集合反映当前产品入口和核心工作流。
- [x] 命令参考覆盖当前稳定 CLI 命令；`help` 可由命令总览明确覆盖，概念型 `tui` 页面可作为允许的非命令页面保留。
- [x] 架构与 UI 模块页反映当前 `crates/` 和 `ccr-ui/` 边界。
- [x] 所有活跃中文页面都有对应英文页面，反之亦然。
- [x] `bun run build` 与 `bun run audit` 在 `docs/` 下通过。
- [x] 基线审计中对 `AGENTS.md`、`TODO.md` 和 `reports/` 的错误镜像要求已通过明确的发布边界修正。
- [x] 根 `just docs-check` 使用仓库约定的 Bun 工具链并执行构建与审计。

## Out Of Scope

- 不修改产品实现、发布流程或历史版本内容。
- 不新增无法从当前实现验证的承诺或路线图。
- 不翻译或发布内部审计报告与 Agent 指令。
