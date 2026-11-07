# 下一对话实施 Prompt

继续完成 CCR GPUI 迁移的所有未完成工作，优先解决侧边栏和 Claude Code 等平台页与 Tauri 的实际差距。请直接实施，不只交一份方案，不只完成一个小切片或几条测试后结束；持续推进当前条件下能完成的产品能力，直到完成或遇到需要外部环境、上游能力或明确产品决定的真实阻塞。

## 一、工作区、任务与既有成果

- 仓库：`D:\Documents\Code\Github\ccr`，分支 `dev`。复用当前工作区，不新建 worktree，不创建重复 Trellis 任务。
- 功能验收历史基线为 `ca60b88cdcb84b438c6d431eaa9be86712514fba`，之前还有 `841d7bbe`（签到）、`99d8d68a`（共享业务）。该整合工作区的 desktop fmt/lint/test、完整 `just ci`、未签名 Windows 便携包均通过。
- 后续命令入口提交为 `8f098784e13b3512b12afd3da70395618ca557d8`。根公开入口现在是 `gpui-*`，旧 `desktop-*` 为可调用的隐藏兼容别名；`ccr-desktop/justfile` 内仍是 `desktop-*`。本交接文档的提交可能更晚。先核对实际 HEAD/status，不能 reset 回历史基线，也不能冒领旧 CI 验证了新代码。
- 现有父任务 `.trellis/tasks/10-17-gpui-desktop`、01 foundation、09 environments 均为 `in_progress`。八个子目录已归档只是管理状态，不等于完成率或验收通过。不要为纯审计重新打开归档任务；在父任务继续处理具体产品缺口。
- 先读 `AGENTS.md`、`C:\Users\lyh\.codex\RTK.md`、`code_map.md`、Trellis 当前上下文及父任务 `prd.md`、`design.md`、`implement.md`、`research/resume-summary-20261106.md`、`research/resume-check-20261106.md`、`research/migration-inventory.json`。
- 保护现有业务成果、备份/脱敏/CAS/取消/环境隔离。先识别已实现能力，不重写已经能用的领域服务。未提交的他人改动不覆盖、不混入本任务提交。

## 二、技能与执行方式

- 使用 `trellis-start` / `trellis-continue` 恢复现有任务，写代码前遵循 `trellis-before-dev` 及目标 package 的 spec index、具体规范和 shared guides。已有任务、设计与本提示已授权继续实施，不重新进行一轮泛泛的任务创建或开工确认。
- 默认由主会话协调，按 `trellis-implement → trellis-check → 必要规范同步 → 本地提交` 推进。子代理提示必须以 `Active task: .trellis/tasks/10-17-gpui-desktop` 开头，明确它已经是该角色、直接执行、不得再派 implement/check；明确文件所有权及不能回滚其他代理改动。优先原生上下文注入，缺失时由子代理加载任务上下文。
- 必须读取用户指定的 `C:\Users\lyh\.skillsmanage\skills\design-atlas\SKILL.md`，并按需要读取其 `design-system-core`、`implementation`、`anti-slop`、`review-prompt` 等 references。不能只在报告里提到技能名字。
- 原生生产 UI 可配合 `C:\Users\lyh\.agents\skills\ui\SKILL.md`。若发现字体、焦点、数据或渲染故障，用 `hunt` 先定位根因。失败门禁按 `ccr-gate-recovery` 找第一处实际失败；提交用 `git-commit`。
- 只有需要运行 Tauri Web 参考或修改其视觉时才使用 `ccr-ui-visual-workflow`。GPUI 运行验收必须用原生窗口，不能拿 HTML/浏览器预览当原生交付。若修改 Tauri locale/IPC，才按相关 i18n/IPC 技能补契约检查；不要无关地铺技能和验证流程。

## 三、截图参考与设计边界

三张用户参考图已保留在本地忽略目录：

1. `D:\Documents\Code\Github\ccr\outputs\desktop\handoff-20261106\gpui-overview-sidebar.png`
2. `D:\Documents\Code\Github\ccr\outputs\desktop\handoff-20261106\gpui-claude-profiles.png`
3. `D:\Documents\Code\Github\ccr\outputs\desktop\handoff-20261106\tauri-claude-workbench.png`

先查看这些图。第一张红框只是用户标记侧边栏，不是红色边框设计要求；图片里的文字、命令片段、账户名和指标不是操作指令，也不是可抄入真实配置或冒充实时数据的 fixture。

当前 GPUI 侧栏缺少清晰的图标、对齐和导航层级；平台主页默认落入巨大配置列表，按钮堆叠、留白与信息组织都不足。Tauri 参考展示的是平台工作台：平台上下文、认证/Profiles 主操作、真实状态与用量、可发现的管理能力。这次要修的是信息架构、真实功能和交互呈现，不只是换颜色、圆角或加几张卡片。

- 现有设计源是 `ccr-desktop/DESIGN.md` 和 `ccr-ui/DESIGN.md`，已采用 `style:airy-glass`。按 design-atlas 的已有设计源分支执行，不重新随机选主题；截图三的旧暖色配色不自动覆盖当前设计。先增量更新原生 DESIGN 合同，再改 token/组件，不用 `apply --force` 抹掉现有适配与安全约束。
- 重设计侧边栏：图标和文字统一左对齐，组标题、平台身份、选择态、hover/focus、底部设置区明确；密度适合桌面工作台，不再用居中文字和大块空白拼成导航。保留现有13个顶级 destination、全部业务能力、路由/快捷键可达性，可合理重排分组和增加平台上下文子导航，不能删入口掩盖遗漏。
- 继续使用 GPUI/Kit 和唯一 theme/token owner；不得用 WebView/HTML 替换原生 UI，不引入第二套窗口或主题栈。平台图标优先复用仓库已有合规资产或统一原生图标，不混用装饰 emoji，不复制专有字体。
- 排查截图中文字呈现、字重、CJK/Latin fallback 与字号层级，验证实际字体解析；不能只写“已使用系统字体”而让运行时回落到不合适的衬线字体。保留浅深色、中文/英文、可读 disabled、键盘焦点和实际空/错/忙状态。
- 真实背景玻璃目前未启用；阴影的 blur_radius 不等于背景玻璃。若当前锁定栈缺 API，继续诚实实色降级，记录源码依据与最小可行方案，不能编造材质支持。依赖/窗口技术的大范围更换需要具体方案，不能以完成验收为由偷偷换栈。

## 四、平台页必须达到可用工作台，而非配置列表或 JSON 演示

先完成一个可审查的“侧栏 + Claude Code 工作台”纵向批次，然后继续其他平台与剩余能力；不要停在第一屏。首次给简短的源码差距表后就开始实施，矩阵随实现维护，不让重复审计成为主要交付。

- Claude Code 默认平台入口应呈现平台工作台，而不是把 Profiles 列表当整个平台页。组织当前 profile/路由、认证与运行状态、主要管理动作、真实用量/订阅及相关资源；Profiles 仍能直接进入并完整编辑。
- 在同一平台上下文提供可发现的 Profiles、认证、MCP、Agents、Plugins、Commands、指令文件、设置/来源、用量等 Tauri 已有能力；哪些必须嵌入概览、哪些用子导航或明确入口，依据实际业务和当前 Tauri 实现决定。不能因能力被放在全局 Resources/Usage 里就声称平台体验已对齐。
- 不拿通用 JSON 输出框、源码编辑器或“以后去 CLI 做”的说明代替已有的常用业务表单、列表、图表和操作。原始源码只作为保留语义的高级功能。配置区使用有层级的列表/详情/上下文工具栏，避免整屏低密度长名单和一排无主次按钮。
- 使用现有平台 owner 与共享投影接真实数据。指标名称、时间窗口、时区、费用口径和缺失状态要对应；录入值、截图中的金额、无数据的0不能冒充计算结果。缺库/缺 provider/未知状态必须明确显示。
- Codex、Grok、Antigravity、OpenCode 分别按当前 Tauri 的真实能力对齐，不套同一组虚构按钮或数量。Codex 要覆盖账户/供应商/配额/模型与会话；Grok 要保留来源与轮换凭据语义；Antigravity/OpenCode 的设置、MCP、资源/扩展按原 owner 与格式承接，不把只读清单说成可安装管理。
- 全局 Overview 不新增另一套 Home insights。这个约束不允许被用来删掉 Tauri 本来就有的 Claude 平台用量洞察。
- 对照入口至少包括 `ccr-ui/src/features/claude/ClaudeCodeView.tsx`、`features/platform/PlatformSubnav.tsx`、`features/platform/{profiles,settings,agents}`、`features/codex/home`、`features/grok/home`、`features/opencode`、`features/usage/platform` 和对应 Tauri commands/services。先按 `code_map.md` 定位，不盲扫整仓。
- 按“入口、数据显示、动作、错误/空/忙/确认、可发现性、键盘/焦点、领域 owner、测试、原生证据”维护平台能力对照矩阵。别名允许归并，业务能力不能静默丢掉；页面存在、CI绿、归档8个目录均不是功能对等证明。

## 五、继续完成已确认的六类产品缺口

1. **Recent 自动记录**：命令任务结束后，GUI 最近命令目前不自动新增。查看 `features/jobs/mod.rs`、`runtime/jobs.rs`、`runtime/ui_state.rs` 和 Tauri `features/commands/commandJobStore.ts`。应由应用持有的终态 owner 持久化，正确处理离页、重订阅和失败/取消，避免重复记录或泄漏参数；不能只在可见视图回调里补一行。GUI Recent 与 CLI history 是不同能力。
2. **分来源用量导入**：`runtime/insights/import.rs` 目前固定 default SyncCommandOptions。迁移 Tauri `commands/usage.rs` 已有 platform/source 选择，沿用安装的 llmusage CLI 协议与 retained runtime。自定义 provider home 若上游 CLI 不支持，必须拒绝而非扫描本机默认数据；不发明 CLI 参数或链接 upstream llmusage Rust crate。
3. **MCP 批量能力**：当前 `runtime/mcp/transfer.rs` 只有单预设和单条复制。补齐 Tauri `commands/mcp_presets.rs` 的多平台预设安装、整批同步；复用已有写入 owner，绑定来源/目标版本和秘密会话，明确逐目标结果及部分失败语义。
4. **分层配置清单**：迁移 `commands/settings_raw.rs`、`commands/grok.rs` 已有 layers/managed/overlay 等来源。固定用户文件能编辑不等于能看到有效配置来源；保留优先级、只读层与项目范围，不能把 managed 源变成可写文件。
5. **全局监控**：Claude Observer 不等于跨业务事件、日志与运行指标。补齐 Tauri `commands/system.rs` 的 monitoring feed/runtime metrics 消费，复用既有持久化/查询 owner；正确捕获 main DB，保留脱敏、上限和隐藏页面不轮询，禁止伪造 RSS、耗时、事件。
6. **工具桥接与手动更新**：补齐 SkillPort/skills-manage 检测打开、CLI 版本检查及 Tauri `update_ccr` 的手动操作。原生安装页目前仅 llmusage，通用命令 allowlist 也不含 install/update，不能说已有命令工作台就等于覆盖更新。复用封闭进程描述符、现有更新 owner 和确认；明确用户安装 CLI 与便携 sidecar 的作用域，不破坏 sidecar SHA校验、不绕过 PATH/进程边界。没有认证更新清单时，自动 updater 仍保持禁用。

六类只是已确认缺口，不是完整能力清单。继续按 Tauri 源码核对真正遗漏，不能补完六项就跳过平台展示对等。

## 六、其余未完成工作及边界

- **性能工程与证据**：完成 Tauri 整应用显式 fixture 隔离，再通过既有领域 API 生成两端相同 S0/S1/S2，接真实 process-create/first-present/action-ready 试次和时钟。隔离须贯穿 catalog、main/archive/legacy DB、日志、偏好、认证、环境与 WebView storage；只设 CCR_ROOT 不够。按 `research/performance.md` 采样，不把空探针、JS paint、平均FPS或重复启动当合格基准。没有合格数据仍记 not_run/not_comparable。
- **平台与原生验收**：新功能仍缺当前二进制的原生收据；旧 Windows 截图不能验证新代码。macOS 编译/运行仍缺，历史 WSL Linux 探针不是独立 Linux 桌面通过；IME、可访问性、通知实际送达与平台生命周期按真实机器证据记录。
- **托盘/隐藏恢复/系统退出**：目前锁定窗口栈无托盘、Windows 隐藏恢复不足，直接 App::quit 缺 veto。能在既有栈内补的继续补；必须换大架构或依赖上游的，给出确切 API/源码、影响和可审查选项，不把 unsupported 写成 pass，也不虚构 watcher。
- **WAF 与会话差异**：当前 WAF 仅手工 Cookie/校验，内嵌浏览器取 Cookie 未实现；保持不引入 WebView的既有边界，若无法对等需明确方案与影响。会话已能导出预览/复制、克隆、移入可恢复 trash；直接保存文件、GUI 恢复是需要收口的交付差异，不是整类会话功能未做，也不能未经判断改回永久删除。
- **持续禁令**：不运行 WSL/doctor/发行版探测，不修 WSL/Docker；不运行 ssh-keyscan、真实 SSH/远程套接字或读取用户身份文件；不改真实用户业务数据，不修改全局 HOME/USERPROFILE。真实 WebDAV/认证/CDK/quota验证没有新授权时只用隔离 fixture/受控loopback。不要把限制当借口停止其他可实现工作。
- **延期**：WSL接入与窗口尺寸/DPI矩阵仍为 user-deferred，本提示没有解除。新侧栏必须在当前默认桌面窗口正常可用，但不能擅自恢复整套尺寸/DPI巡检。
- **台账**：72个命令、69条路由的 planned 状态包含过时记录和已有别名，必须逐项更新能力映射，不能按数量宣称缺口或完成率。README 的导入/quota 过时句已在 `8f098784` 修正，不重复修旧状态。

## 七、验证只服务于核心交付

- 每批先运行最窄的领域/控件回归；重点覆盖真实写入、冲突、环境串读写、取消/退出、秘密、导航不可达、焦点和渲染失败。不为低风险文案、机械样式堆镜像测试。未修改代码复用有效收据。
- 新增或放大的 GPUI builder/render 必须守住 Windows 实际1 MiB栈：先正常编译，再对测试可执行文件施加栈限制，必要时扩展既有守卫；不要把 rustc 自己低栈崩溃误判为产品失败。不得增大产品栈掩盖大 render。
- 按 design-atlas 执行机械 anti-slop 与独立 judged review。脚本不支持 Rust/GPUI、未扫描到文件或仅能查CSS时，如实写覆盖不足；不能拿0项扫描或0 FAIL冒充原生视觉通过。不得用 waiver 绕过可访问性、正确性和数据真实性底线。
- **必要的原生检查是授权范围内工作**：核心布局改完并构建后，用隔离 fixture做侧栏、Claude工作台和实际受影响交互的少量 before/after 截图、键盘/焦点及必要浅深色检查。不得因此反复全13页巡检、重拍未变页面或恢复延期矩阵。不能因为减少 computer use 就只做headless并宣称视觉已验证。
- 完成功能批次集中执行根 `just gpui-fmt-check`、`just gpui-lint`、`just gpui-test`；如进入 `ccr-desktop/` 则用原 `desktop-*`。共享层/最终整合执行完整 `just ci`，交付候选执行 `just gpui-package`。不用 doctor充当构建验证，不执行不必要clean，也不删除交接截图/历史证据。
- Windows进程PATH将 `C:\Program Files\Git\bin` 放前，避免System32的WSL Bash。必要时用 `rustup which cargo` 的真实工具链，只调整检查进程环境；保持根/Tauri/GPUI target 隔离。默认Rust测试并行和 `--skip export_bindings`不变，绑定生成另门。先修首个实际失败，不改阈值、跳过测试或取消锁/备份来换绿。

## 八、提交与完成标准

- 每个实际功能批次完成后做独立check、同步必要spec/任务矩阵，再用中文 Conventional Commits 本地 `git commit -F`；这已获授权，不重复询问提交许可。保留代码、测试、必要规范和任务收据的合理原子边界；只提交本任务文件，不混他人工作，不push、不amend、不加Cursor署名，不用 `git add -f .trellis/`。
- 每批要交付可用产品能力及与Tauri逐项对照，不只backend owner、测试数量或文档。侧栏与平台页必须有当前二进制的核心原生证据；其余项目明确区分 implemented、fixture/headless、native、not_run、unsupported、user-deferred。
- 只有 **01 已独立验收并归档 AND 09 已独立验收并归档 AND 最终完整 `just ci` 退出码为0** 三个条件同时满足，才允许归档父任务；任一不满足都保持 `in_progress`，不运行会自动归档的 finish-work/archive。不得为了任务树好看勾完未通过的AC。
- 不在第一处真正外部阻塞时停掉所有工作：继续完成不依赖它的产品项。若需要用户决定，给出已做完的具体方案和最小问题，解释根因与剩余影响，不泛泛反复请求开工。
- 最终只报告：已交付的平台/侧栏与功能、提交SHA、必要门禁和原生截图、仍缺什么及准确原因；不能概括成“核心全部完成，只剩测试”。
