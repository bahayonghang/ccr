# Research: CCR Tauri 前端视觉迁移覆盖清单

- Query: 轻盈玻璃迁移会影响哪些真实页面、样式所有者和行为合同？
- Scope: internal，静态源码检查；不操作 UI、不运行产品测试、不修改产品文件。
- Date: 2026-10-26

## Findings

### 1. 规模与真正的入口

本次扫描 `src/features/**/*.css` 为 52 文件，`src/styles/**/*.css` 为 18 文件，`src/ui/**/*.tsx` 为 32 文件。数量仅表示这些 glob 范围，不等于必须修改的文件数。应按公共令牌和消费关系迁移，禁止以逐文件重写作为完成指标。

`src/shell/routeCatalog.ts` 有 71 个静态 child 声明，加 `platformDescriptors.ts` 当前生成的 3 个 Antigravity child，再加 layout parent 和 tray，共 76 条 catalog 路径记录（包括重定向、重复 `/` 布局/首页记录，不是 76 个独立页面）。证据：`ccr-ui/src/shell/routeCatalog.ts:14` 静态 children、`:245` genericChildren、`:265` 合并、`:267` tray、`:290` flatten；generic 描述在 `ccr-ui/src/config/platformDescriptors.ts:84`。实施前应从 flattenCatalog 导出全清单并核对当前计数，避免注释变更不同步。

`ccr-ui/src/shell/router.tsx:27` 聚合 feature routeLoaders，`:45` 有缺失 loader 的 placeholder 回退。验收不能仅断言 URL 可打开；必须同时验证目标页面标识，避免把 placeholder 当作迁移成功。

### 2. 样式所有者与首屏链

| 所有者 | 源码证据 | 迁移风险与建议 |
|---|---|---|
| 启动 | `ccr-ui/src/main.tsx:17` 导入 index.css；`:24` runtime nonce；`:25` reduced motion；`:30` 延迟样式 | boot loader、首屏令牌与之后样式必须一致；不能靠延迟 CSS 修正闪烁 |
| 主令牌 | `ccr-ui/src/styles/tokens.css:50`、`:196`、`:233`、`:765`、`:776` | 主色有默认与 accent override 的重复定义点；只改默认会被用户 accent 覆盖 |
| Tailwind 桥 | `ccr-ui/src/styles/core.css:15` 两层模型；`@theme inline` 命名映射 | 保留现有运行时 token → 工具类结构；不要新增独立玻璃主题系统 |
| 首屏 CSS | `ccr-ui/src/styles/index.css:6` | core、checkin/codex-auth shared、shell.css；核心外壳不能依赖空闲加载 |
| 延迟 CSS | `ccr-ui/src/utils/deferredStyles.ts:14`、`:44` | interactive 首帧后、decorations 空闲后；两层加载失败必须仍可读 |
| 表面别名 | `ccr-ui/src/styles/components/surfaces.css:11`、`:21`、`:32`、`:43`、`:53` | shell/workspace/card/modal/status 多点消费；不能把 workspace 全局别名改为 blur 而不检查所有消费者 |
| 主布局 | `ccr-ui/src/shell/MainLayout.tsx:86`、`:91`、`:139` | hideGlobalBackground/theme-stage 分支不能漏迁；保留原滚动容器 |
| 壳 CSS | `ccr-ui/src/shell/shell.css:71`、`:109`、`:120` | 多个 shell-blur 消费点意味着简单开放 shell blur 会产生多层；应明确唯一承载者 |
| 原语 | `ccr-ui/src/ui/primitives.css:95`，`ui/dialog.tsx:43`，`ui/base-modal.tsx:337` | 按语义表面与组件状态迁移；保留焦点/滚动锁/层级 |
| 旧调用点 | `ccr-ui/src/features/configs/styles/config-card.css:3` | 仍使用 legacy liquid-glass-blur，不能只改新 surface token 认定覆盖完整 |
| 数据表面 | `ccr-ui/src/features/usage/styles/platform-usage-insight-panel.css:12` | 直接消费 workspace blur；若 workspace 开 blur，数据面板也会模糊，必须分离唯一外壳与实底内容 |

发现规范漂移：`tokens.css:490` 注释仍写同屏 ≤3，但下一段及当前 theme spec、DESIGN 要求 floating-only ≤1。此处不是采用 3 层的依据；新方案必须统一文字与可执行合同。`utilities.css:777` 的“blur 不应完全关闭”旧注释也需对照后续实际 reduced-transparency reset 审核。这里只记录，不越界修改。

### 3. 完整路由家族覆盖矩阵

以下覆盖当前 catalog 的全部页面家族；redirect 必须按原目标保留，不新增独立样式页面。

| 家族 / 路由 | 主要源码所有者 | 代表状态 / 必验点 |
|---|---|---|
| `/` | `features/usage/dashboard/DashboardView.tsx`，home 样式与 dashboard 组件 | 空数据、已有数据、Web 无本机能力；首屏与 Insights 下半屏均覆盖 |
| `/settings` | `features/configs/AppSettingsView.tsx`、`features/configs/styles/app-settings.css` | light/dark/system、flavor、accent、字体、长标签 |
| `/claude-code` 及 settings/profiles/auth/system-prompts | `features/claude/` | 平台首页、表单、Profile 编辑/确认、认证状态、source 模式 |
| `/codex` 及 mcp/profiles/agents/sessions/slash-commands/auth/settings/system-prompts | `features/codex/` | 同上并包含 sessions 与资源列表；不能只验平台首页 |
| `/grok` 及 auth/profiles/settings | `features/grok/` | Local-only、managed lock、dirty-key patch 与版本冲突 |
| `/antigravity` 及 slash-commands/system-prompts/mcp/agents/plugins | `features/gemini/` 与 generic descriptor | hideGlobalBackground 的平台首页、资源表单/列表；动态生成的三条路由必须纳入 |
| `/opencode` 及 providers/mcp/agents/commands/plugins/settings/system-prompts | `features/opencode/` | 这些页面多数 theme-stage 分支，需逐家族看实底与背景关系 |
| `/configs`、`/converter` | `features/configs/` | 列表/历史、筛选、多选、转换输入输出、空/错/忙状态 |
| `/commands/:client?` | `features/commands/CommandsView.tsx` | 流式输出、busy、取消/恢复/历史，原缓存语义保持 |
| `/sync`、`/wsl`、`/ssh` | `features/sync/` | 环境不可用、连接状态、确认、安全字段遮罩 |
| `/usage`、`/budget`、`/pricing` | `features/usage/` | 图表/表格、加载/空/错、长数字、切 tab 与主题变化 |
| `/monitoring` | `features/monitoring/MonitoringView.tsx` | 大量日志、严重性编码、滚动容器、过滤 |
| `/agent-sessions` | `features/agent-sessions/` | 发现/归档/详情/Transcript、分页与两处虚拟化 |
| `/mcp-manager` | `features/mcp/McpManagerView.tsx` | 平台聚合、卡片/表格、配置编辑、安装结果 |
| `/slash-commands`、`/agents`、`/agents/:name` | `features/commands/`、`features/platform/agents/` | 列表/详情、代码/Markdown、危险动作确认 |
| `/skills` | `features/claude/SkillsMigrationView.tsx`（`routeLoaders.ts:24`） | 独立 Skillport 迁移引导；检测中/已安装/未安装/不支持/拉起失败，不重建旧 Skills 管理器 |
| `/plugins`、`/hooks`、`/output-styles`、`/statusline` | `features/claude/` 及对应资源组件 | 开关、表单、列表、代码预览与空态 |
| `/checkin`、`/checkin/manage/:accountId` | `features/checkin/` | 账户、余额刷新四态、批处理进度、错误详情 |
| `/tray/codex` | `features/tray/CodexTrayPanelView.tsx` | 独立于 MainLayout 的紧凑窗口；可用 web 路由检查样式，但原生托盘语义需单独验收 |
| redirects | catalog 中 gemini-cli*、ccr-control、stats、sessions、mcp、mcp/unified、skills-manager、skillport-manager、skills/add、skills/hub、skills/:platform/:name、market、opencode/skills | 保持所有目标与 query/参数既有行为；本次不整理路由 |

### 4. 冻结的行为合同

- **滚动与缓存**：`MainLayout.tsx:52`/`:53` 保存/恢复路由内部滚动；`:139` content-scroll-area 是滚动所有者。外壳加圆角/背景不能改成 body 滚动或新增竞争滚动容器。
- **虚拟化**：`AgentSessionList.tsx:83` 估高 132、overscan 8；`AgentSessionTranscript.tsx:87` 估高 148、overscan 6。不得因视觉宽松直接增加行 padding 导致测量漂移；有意几何变化需单独证明滚动定位与末项可达。
- **图表**：`UsageDashboardView.tsx:57` 维护 visitedTabs，`:60` 以 hidden 保持挂载；不得改为条件卸载。`.trellis/spec/ccr-ui/frontend/usage-chart-stability-contracts.md:11` 起约束 options/series identity 和 resize 行为。
- **确认**：`ui/confirm-modal.tsx:6` danger/info/warning，`:32`/`:38`/`:44` 分别定义语义；不能用绿紫装饰色覆盖危险/警告。`.trellis/spec/ccr-ui/frontend/confirm-interaction-contracts.md:14` 定义仅两套确认路径，`:65` 起有生产 CSP nonce/滚动锁合同。
- **草稿和源文件**：`.trellis/spec/ccr-ui/frontend/raw-config-editor-contracts.md:31` 原文不能放 store/log/localStorage，`:34` guarded version save 与 backup，`:39` 环境变化保留挂载/草稿，`:41` 取消或加载失败保留原编辑器。重构不改 mount/key/表单请求组装。
- **主题偏好**：`shell/stores/shellPreferences.ts:46` 说明 localStorage 分 key 持久化；`:125` 起读取既有 theme/flavor/accent，`:166` 起按原工具写回。不能为新皮肤换成新的单一 blob 或悄悄清空已存偏好。
- **IPC/资源**：API facade、typed IPC、Query keys、业务 hooks、Tauri backend 与 llmusage SQL 均不是本次视觉重构所有权；保留 Web 不支持能力的真实提示，不做假数据伪装原生通过。

### 5. 建议工作包与依赖

1. **基线和设计合同**：先导出 route matrix；记录当前主题/缩放/窗口；统一新版设计、旧规则冲突与色彩语义。核对透明合成对比度与当前首屏/延迟链。
2. **令牌、唯一外壳、公共组件**：覆盖 boot、theme axes、首屏/延迟样式、surface aliases、导航、输入、按钮、表格、overlay。此包是其余视觉迁移前置。
3. **平台/配置/资源工作区**：利用 shared Base 与 profiles 组件迁移全部平台及资源页面，逐族记录空/错/忙/禁用/确认态。
4. **数据/任务/环境/独立托盘**：用量、监控、会话、command、sync/checkin、WSL/SSH、tray；专项保护虚拟化、图表稳定性、流式日志与异步状态。
5. **整体验收**：全 catalog 家族审核和重点状态矩阵；真实计算样式/层数/对比度；性能前后对照；Web 与原生证据分开。

不建议同时派多名实现者改 tokens.css、shell.css 或 primitives.css。它们是单一所有者热点；先串行稳定基础，再按互不重叠 feature 目录划分工作。

### 6. 现有工作树冲突边界

主会话启动快照报告 70 个既有未提交路径，包含 `ccr-ui/src-tauri/` 多个 commands、根 Rust crates、Cargo 锁文件、GPUI 新工程和 `.trellis/spec/`。本研究未执行 git 操作。视觉任务应仅拥有批准后的前端/design/spec 路径；不得整理现有 Rust/GPUI 改动，不把它们混入本任务提交。当前前端源码未出现在主会话提供的 dirty 清单，但开始实现前仍需重新核对并记录变化。

## Related specs

- `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`
- `.trellis/spec/ccr-ui/frontend/profiles-page-contracts.md`
- `.trellis/spec/ccr-ui/frontend/raw-config-editor-contracts.md`
- `.trellis/spec/ccr-ui/frontend/confirm-interaction-contracts.md`
- `.trellis/spec/ccr-ui/frontend/usage-chart-stability-contracts.md`
- `.trellis/spec/ccr-ui/frontend/agent-session-observability-contracts.md`
- `.trellis/spec/ccr-ui/frontend/platform-surface-contracts.md`
- `.trellis/spec/ccr-ui/frontend/layering-contracts.md`

## External references

无。该研究依据当前 checkout 静态源码，未查询外部版本文档。

## Caveats / Not Found

- 本文不代表视觉、原生、性能或功能测试通过；无截图、无浏览器或桌面操作。
- 76 为当前源码声明结构推导，实施验收须运行真实 flattenCatalog 快照，区分 redirect 与独立页面。
- 原生窗口/托盘的实际尺寸与操作体验未验证；不能照搬主窗口玻璃外边距到托盘。
- `/design-md` 来源与调色板、最终玻璃承载层决策由主会话研究；本文仅提供消费关系与迁移风险。
