# 平台能力对照（当前实施台账）

基线 `90131cd6`；此表随本轮实现更新，不把页面存在、旧 planned 数量或子任务归档当作功能对等。Tauri 参照：`ClaudeCodeView.tsx`、`config/moduleSubnav.ts`、`PlatformSubnav.tsx` 和对应领域页面/commands。`native` 一栏只接收本轮实际二进制收据。

稳定提交批次已通过 GPUI fmt/lint/test（lib202、native_shell9、19个实际1MiB子进程）和根完整CI（HEAD65a5fe1f）。随后新增的Benchmark/System Info/Codex diagnostics/SSH detect/账户凭据回填仍需各自独立check及新一轮集成门禁；不得复用该CI结论。原生窗口验证由用户明确延期。

## 默认入口与平台上下文

| 平台 | 入口/可发现性 | 数据显示 | 操作与领域 owner | 空/错/忙/确认 | 键盘/焦点 | 测试 | 当前 native |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Claude | 默认 Workbench；Profiles/Auth/Settings/MCP/Agents/Plugins/Commands/Instructions/Usage/Observer/Sessions 上下文 | 当前profile、本地凭据事实、平台过滤30天记录费用与7日序列；Observer今日/月/全部及订阅独立读取 | 保留ConfigReadService、platforms auth owner、claude_files settings集合、MCP source-bound writer、共享usage/observer | 各来源错误独立；不把凭据存在说成远端认证成功；常用集合表单/Observer可读投影已实现并通过独立check | 原有focus_main与shell导航；新增上下文回归待实际结果 | 稳定批次 native_shell 9通过；原profile/资源与普通表单实际控件通过 | not_run |
| Codex | 默认 Workbench；Profiles/账户/供应商/模型/MCP/Agents/指令/用量/会话 | 当前profile、本地凭据事实、平台用量；账户/模型/配额留在所属功能 | ccr-codex账户/供应商/配额/会话owner；显式配额刷新可能写凭据和历史，非页面自动请求 | 缺账户/模型/来源明确；真实网络不由fixture冒领 | 保留原控件和确认焦点；上下文回归待跑 | 稳定批次账户/供应商/会话控件及工作台路由通过；新诊断另验 | not_run |
| Grok | 默认 Workbench；Profiles/Auth/Settings/来源/用量 | 当前profile、凭据检测、平台用量；不增加虚构MCP服务器入口 | Grok原领域含轮换token保存/关闭；来源层复用共享inventory | 保留激活/managed锁与来源拒绝；不将用户配置当最终有效值 | 原账户/设置焦点保留 | 账户隔离、来源、激活概览控件通过 | not_run |
| Antigravity | 默认 Workbench；实际legacy设置/MCP/Commands/指令/扩展 | 所属来源的实际状态；不编造Profiles/账户 | 原文件格式与MCP writer；扩展manifest只读 | 读失败不变成空清单；不把manifest清单称安装管理 | 上下文入口与实际文件编辑focus | platform_files/资源与平台上下文回归通过 | not_run |
| OpenCode | 默认 Workbench；Providers/Settings/TUI/MCP源/Agents/Commands/Plugins/指令 | 实际settings/provider及平台用量 | 原OpenCode JSON/JSONC与Markdown owner；plugins只读metadata，不执行代码 | 不虚构认证/Profile；保留legacy源和目标版本 | 实际资源编辑与返回上下文 | OpenCode source/TUI/文件控件与上下文回归通过 | not_run |

## 跨平台具体缺口与本轮状态

| 能力 | 入口/显示/动作 | 安全与状态 | 所有者/关键测试 | 实施与证据 |
| --- | --- | --- | --- | --- |
| 图标侧栏/字体 | 13顶级destination保留；图标左对齐、组分隔、底部设置 | 选中/hover/focus和浅深token；字体请求/实际解析诊断 | shell/theme唯一owner；native_shell/实际字体receipt | implemented；本轮native pending |
| Source layers | Settings中的配置来源清单与显式项目scope；路径/存在/size/mtime/只读 | 未选项目=unknown；managed/requirements/overlay只读；fixture系统根隔离 | ccr-app::platform_settings::layers；共享1/native runtime1/control1/Tauri raw7通过 | implemented_pending_native |
| GUI Recent | retained命令终态自动记录；离页无需callback | 参数值从持久化载荷删除；唯一admission；取消/失败非成功 | RuntimeHandle::job_with_finalizer；CommandJobs owned terminal；ui_state manager | 已独立check并提交96376961；runtime11、commands10、native jobs4和ui_state3通过；native交互延期 |
| 分来源用量导入 | 全部/九来源与7/30/90天；平台上下文锁定来源 | retained状态scope、取消/离页；custom home拒绝；不扫描不相关Claude工具 | 已安装llmusage CLI协议；runtime import5/5；控件pending | runtime5及全局/平台scope控件通过；native user-deferred |
| MCP多目标/整批 | 预设与来源批量；逐目标Add/Replace计划及结果 | opaque秘密session，原源/目标版本；Replace默认关；部分失败明确 | 原source-bound writer扩展save_batch；backend3/3；控件pending | backend3/实际批量确认控件通过；普通MCP修复后13项回归通过；native user-deferred |
| 工具桥接/手动更新 | Install内CLI版本/SkillPort检测打开/CCR更新确认 | fixture拒绝真实动作；用户.cargo/bin目标；拒绝sidecar重合；无自动updater | 共享tools7/7；native runtime/control与Tauri消费者pending | 共享/native/Tauri消费者及实际控件通过；实际外部打开/更新not_run |
| 全局监控 | Overview内监控入口；跨业务事件/持久日志/runtime指标 | main DB独立于archive，脱敏/上限，隐藏不poll；RSS真实可选；未知p95不编造 | ccr-app::monitoring、ccr-db::log_repo严格native查询；旧Tauri宽容查询保留 | runtime/control共5通过；DB严格/旧语义回归1通过；ListBox包装、隐藏停止与1MiB守卫通过；native延期 |
| Observer可读投影/集合常用表单 | 普通表格含完整值tooltip和数值对齐；Claude Agents/Plugins/Commands常用字段；原始JSON作为高级功能 | 未知字段/省略秘密保持；确认挂载后安全焦点与generation保护 | 既有shared owner/源版本；独立check报告platform-check-20261106.md | Claude确认回归红→绿；最新tooltip已纳稳定批次门禁与1MiB守卫；native延期 |
| Codex本地滚动用量 | 显式扫描5小时/7天/全部历史；区别llmusage与远端quota | 捕获Codex home/locks，缺源不是零，部分扫描诊断，页面打开不扫描 | 原CodexUsageService owner；runtime2与实际control1通过 | implemented_fixture_headless；native延期 |
| Grok激活概览 | 激活/漂移/恢复基线缺失、记录配置凭据形态、启用/总数 | 原inspect_activation_state只读；不会修复文件或把本地形态当远端认证 | ProfileLifecycleContext::grok_platform；runtime2/control1通过；独立源码check无新增finding | implemented_fixture_headless；1MiB/最终门禁与native待验 |
| 用量能力报告 | Usage内结构化存储/CLI能力；所有实际12键 | 缺库/不可读/schema/表/字段区分；fixture禁止真实CLI；不导入/迁移；本地取消区别上游能力 | ccr-usage::DbCapabilities＋新shared capabilities owner，Tauri wire不变；shared2/nativecontrol1通过 | implemented_fixture_headless；最终门禁与native待验 |
| 普通Settings/供应商/MCP表单 | Settings dirty字段、元数据供应商表单、MCP常用字段；保留高级源编辑 | 原版本CAS、managed锁、空凭据保留、未知字段保持 | platforms/forms实际control2（4平台settings与provider）、MCP typedHTTP control1/domain1（3目标）、Grok patch1通过 | MCP秘密/HTTP切换/焦点回调修复，旧MCP回归13项通过；独立check完成 |
| 会话直接保存/恢复 | 预览内容经原生Save对话框写新文件；versioned trash确认恢复 | 新文件CAS、捕获home/源版本、部分完成独立报告；旧CLI恢复行为保留 | Codex trash owner与同archive registry刷新 | 共享增量刷新已修复并提交a25df5f；domain11/shared3/session7通过，稳定批次全GPUI门禁通过；native延期 |

## 扩展批次：已检查的实际消费者

| 能力 | 实际入口/行为 | 确认/隔离/生命周期 | 源码与headless证据 | native |
| --- | --- | --- | --- | --- |
| 系统信息 | 全局监控内显式读取CPU/逻辑核、OS、主机、RAM/swap、uptime；区别进程RSS | bounded worker、采样时间/间隔、缺失非零；隐藏清除；fixture/benchmark/remote拒绝主机探测 | System5、旧Monitoring2；独立check；扩展全GPUI门禁通过 | user-deferred |
| Codex保存/端口诊断 | Save review内best-effort PID警告，固定1455预检，清理核对 | safe Cancel/迟到generation；opaque scope/login/nonce/revision；commit-aware join，unknown PID仅指导手动关闭 | pending7/OAuth10/diagnostics7；独立check；预检拒绝与临时端口取消各自核验 | user-deferred |
| SSH CLI检测 | 已验证连接中固定四项PATH查询，展示installed/path | capture host+session UUID；断开/重连/换环境/离页拒绝迟到结果；不读私钥、不查版本 | shared26、remote3、adapter1；新区域SFTP确认裁切已修；独立源码check | user-deferred |
| 账户原值编辑 | 只回填所选账户，普通Session/API用户字段；全部Cookies JSON为显式高级入口 | unknown Cookie保留、masked inputs、cancel/leave清除、nonce+account门；缺钥typed提示，metadata仍可保存 | checkin17/domain10；缺钥生成风险红→绿；独立check | user-deferred |
| Benchmark数据/时钟 | 两端显式模式、共同文件校验、UTC默认报表窗口，旧Fixture/Local语义保留 | WAL/journal/links/闰秒早拒绝；不改Date或elapsed；实际启动校验成本计入 | 两端真实S0/S1/S2读回、前端90、strict检查；最终原生全量分页读回1通过；性能非可比PASS | 未采样 |

本扩展批次完整 `gpui-fmt-check/lint/test` 已通过：lib228、native_shell9、native_ui2、runtime_shutdown2，26个实际1MiB/毒化锁子进程通过。共享改动后的最终Root CI和交付包另行取得，不复用HEAD65a5fe1f旧CI。S2持续日志源尚在独立工程批次。

## 尚未获得或需后续闭合的证据

当前表不宣称完整Tauri命令/route对等。两端显式fixture核心业务隔离与paired S0/S1/S2领域数据已实现和核对；共同报表时钟与同定义验证已闭合；仍缺真实进程创建/呈现/交互计时及S2持续流量，性能保持not_run/not_comparable。原生OS/IME/通知/隐藏恢复/托盘/系统直接quit-veto继续开放。会话保存/恢复已完成源码与headless回归。WSL与窗口尺寸/DPI矩阵为user-deferred；真实glass仍为实色降级。后续按具体能力与owning source闭合，不按旧计数写完成率。

## 历史窄收据（最终稳定批次结果以 integration-gates-20261106.md 为准）

- Source layers：共享1、native runtime1/control1、Tauri raw7通过。
- 分来源导入：runtime5、全局control1、平台scope control1通过。
- MCP批量：backend3/control1通过；后来普通表单批次使两条既有MCP用例失败，需修复并重跑，旧通过不能覆盖当前改动。
- Tools：共享tools/bridge过滤器11（与gateway过滤器有重叠，不累计唯一数量）、native runtime3/control1通过；Tauri system6/raw7通过，生产严格clippy通过。实际外部GUI启动/真实更新未运行。
- Windows 1MiB守卫：扩展12个实际控件子进程通过；随后Observer tooltip、Monitoring ListBox与普通表单仍需最终守卫。
- 侧栏/工作台native_shell9通过；新表单与最新监控包装不由此收据覆盖。
- 所有收据均为源码/fixture/headless；当前原生交互按用户决定延期。被游戏遮挡的截图不作为证据。

## 源码复核修正

- Antigravity 的通用 `configs/agents.ts` / `configs/plugins.ts` 虽然连接了表单函数，实际 `src/api/domains/gemini.ts:170,201` 返回空清单，178–198 / 209–229 的 mutation safe-stubs 返回不支持；handler registry 没有对应业务命令。`commands/gemini.rs:338–347` 只提供扩展目录清单。因此兼容路由不是待迁移的后端CRUD，不向原生settings发明新数组。扩展manifest只读语义继续保留。
- Codex Slash Commands 页面本身是兼容快捷入口（Sessions/Agents/MCP），没有可管理命令；Skills 页面本身是迁移到独立SkillPort的检测/打开入口。旧路由数量不表示这些存在另一套待重建的CRUD。
- 本轮全部72条旧planned命令和76条route已逐项source reconciliation，保留历史字段；`partial` / `unsupported` / `user_deferred` 不记成完成。用户随后明确选择“稍后再做原生验证”，本轮后续native交互继续not_run，字体请求/解析日志与短暂可见侧栏不代表Claude工作台交互验收。
