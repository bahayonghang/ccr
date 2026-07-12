# 完善中英文 README 的 TUI 与 CCR UI 截图介绍

## Goal

让首次访问仓库的读者能够从中英文 README 直接看懂 CCR 的终端交互界面与桌面界面分别解决什么问题，并通过经过脱敏处理的真实截图快速建立产品认知。

## Confirmed Facts

- 根目录 `README.md` 与 `README_CN.md` 当前仅简短提及 TUI 和 CCR UI，没有界面截图或对应功能介绍。
- 仓库当前没有可直接复用的产品截图资产，也没有既定的 README 截图目录约定。
- TUI 当前覆盖 Claude/Codex Profile、Claude/Codex/OpenCode Auth、Runtime 状态与 provider 级用量信息等界面。
- CCR UI 当前包含 Dashboard、配置管理、同步、用量、监控、MCP、Skills、Plugins、Hooks 等多类页面，截图范围需要主动收敛。
- CCR UI Web 预览确认 Dashboard 能同时呈现运行就绪度、行动队列、用量趋势和平台状态，适合作为产品总览画面。
- CCR UI Web 预览确认 Codex Profiles 页包含配置概览、搜索过滤、Profile 列表、上下文侧栏和健康审计，适合作为详细管理画面。
- Usage 页核心数据依赖桌面后端与本地 llmusage 数据；若将其纳入截图，需要额外构造隔离用量数据并使用桌面采集。
- CCR 配置解析支持通过 `CCR_ROOT` 指向隔离目录，可为 TUI 及桌面后端准备不接触真实用户配置的临时演示数据。
- 用户明确要求 URL、API key/token 等关键信息必须打码或以其他可靠方式脱敏。

## Requirements

- 本任务只更新根目录 `README.md` 与 `README_CN.md`，不扩展到 `ccr-ui/README.md` 或 `ccr-ui/README_CN.md`。
- 中英文 README 对 TUI 与 CCR UI 的介绍保持信息等价，不出现一侧独有的关键卖点或截图。
- 使用三张职责明确的截图：一张 TUI 总览、一张 CCR UI Dashboard、一张 CCR UI Codex Profiles，并为每张截图配套简洁、与画面一致的介绍。
- 三张截图统一使用英文界面，`README.md` 与 `README_CN.md` 复用同一组图片资产，不维护重复的本地化截图。
- TUI 截图重点展示 Profile 切换、Runtime/Auth 状态与详情面板。
- CCR UI Dashboard 截图重点展示运行状态、下一步与平台概览。
- CCR UI Codex Profiles 截图重点展示配置管理、过滤与健康审计。
- 截图必须在隔离的临时 `CCR_ROOT` 与合成演示数据下生成，禁止读取或修改真实用户配置。
- 合成数据中的 URL、key/token、账号和路径使用明显虚构值，并在最终图片中以不透明遮挡覆盖，形成双重保护。
- 三张共享图片存放在 `docs/assets/readme/`，使用无损 PNG 与统一的 `1440 × 900` 画布。
- 图片文件名固定为 `ccr-tui-overview.png`、`ccr-ui-dashboard.png` 与 `ccr-ui-codex-profiles.png`。
- 敏感区域使用与界面配色协调的不透明实心遮挡，不使用模糊或马赛克。
- 截图只展示仓库当前真实存在的能力，不使用无法从当前实现验证的宣传性描述。
- 所有截图在提交前必须检查 URL、API key、token、账号标识、本地用户名、私人路径及其他可识别信息。
- 敏感值不得以清晰像素、可恢复的轻度模糊、图片元数据或 README 文本形式进入仓库。
- 图片需有中英文可访问性说明，Markdown 引用在仓库浏览器中可正常解析。
- 在现有 Features/核心特性之后、Quick Start/快速开始之前新增 Interface Preview/界面预览章节。
- 新章节按 TUI、CCR UI Dashboard、CCR UI Codex Profiles 三个画面组织；每个画面只配一段与截图一致的简洁介绍。
- 仅为衔接新章节而调整开头简介或现有功能条目，不扩写安装、迁移、命令和开发章节。

## Acceptance Criteria

- [ ] 目标中英文 README 均包含 TUI 与 CCR UI 的截图和对应介绍。
- [ ] 每份 README 恰好呈现已确认的三类画面，不额外加入 Usage 截图。
- [ ] 两种语言使用同一组截图，介绍结构与关键事实保持对称。
- [ ] 三张共享截图中的可见界面文案均为英文；中文信息只出现在 `README_CN.md` 的标题、说明和替代文本中。
- [ ] 所有截图路径存在且使用仓库内相对路径。
- [ ] 三张图片均为 `1440 × 900` 无损 PNG，文件名和目录符合已确认规范。
- [ ] 逐图人工检查与文本/OCR 辅助检查均未发现 URL、key/token、账号、本地用户名或私人路径泄漏。
- [ ] 截图中的敏感区域使用不可逆遮挡或源数据替换，不能通过放大直接辨认原值。
- [ ] README 格式检查、链接/资源检查和 `git diff --check` 通过。
- [ ] 未修改与 README 展示无关的产品行为或界面实现。

## Out of Scope

- `ccr-ui/README.md`、`ccr-ui/README_CN.md` 及其他子项目 README。
- 为了截图而新增或重设计 TUI/CCR UI 功能。
- 扩写与 TUI、CCR UI 展示无关的安装、迁移或 API 文档。
- 全面重写根 README，或补充新的安装渠道、平台支持和发布说明。
- 制作宣传视频、GIF 或交互式演示。

## Notes

- 当前任务保持 `planning` 状态；用户确认规划前不修改 README 或截图资产。
- 产品范围与截图策略已完成访谈，没有剩余的需求开放问题。
