# 功能迁移台账

由 command-manifest.json 与实际执行 flattenCatalog() 的输出建立，不靠 grep 猜动态路由。
基线为 340 通用 / 348 Windows commands、76 route records（含 18 redirects）。
完整逐项 owner、风险类别、来源路径与待补 native action/verification 见 migration-inventory.json。
初始捕获时所有项目均 planned，不代表实现。当前状态以 JSON 各项的 status/native_action/verification 与带日期 execution 字段为准；保留初始来源与历史证据，不重新运行捕获脚本覆盖执行记录。

| 现有模块 | 命令数 | 实施子任务 |
| --- | ---: | --- |
| agent_sessions | 5 | gpui-07-insights |
| builtin_prompts | 3 | gpui-06-resources |
| checkin | 25 | gpui-08-jobs |
| checkin_extended | 7 | gpui-08-jobs |
| claude | 2 | gpui-05-platforms |
| claude | 24 | gpui-06-resources |
| claude | 2 | gpui-07-insights |
| claude_auth | 6 | gpui-05-platforms |
| claude_observer | 9 | gpui-07-insights |
| claude_profiles | 10 | gpui-04-profiles |
| codex | 10 | gpui-04-profiles |
| codex | 3 | gpui-05-platforms |
| codex | 20 | gpui-06-resources |
| codex | 10 | gpui-07-insights |
| codex | 1 | gpui-09-environments |
| codex_auth | 18 | gpui-05-platforms |
| codex_model_providers | 3 | gpui-05-platforms |
| command_exec | 6 | gpui-08-jobs |
| config | 13 | gpui-04-profiles |
| config_extended | 1 | gpui-04-profiles |
| converter | 1 | gpui-04-profiles |
| environment | 4 | gpui-09-environments |
| environment_legacy | 2 | gpui-09-environments |
| events | 4 | gpui-02-services |
| exit_confirm | 2 | gpui-09-environments |
| gemini | 2 | gpui-05-platforms |
| gemini | 9 | gpui-06-resources |
| grok | 7 | gpui-04-profiles |
| grok | 7 | gpui-05-platforms |
| grok | 1 | gpui-07-insights |
| install | 8 | gpui-08-jobs |
| mcp_presets | 7 | gpui-06-resources |
| opencode | 7 | gpui-05-platforms |
| opencode | 9 | gpui-06-resources |
| pricing | 4 | gpui-07-insights |
| settings_raw | 6 | gpui-05-platforms |
| shell | 10 | gpui-09-environments |
| ssh | 13 | gpui-09-environments |
| sync | 17 | gpui-08-jobs |
| system | 1 | gpui-09-environments |
| system_extended | 2 | gpui-09-environments |
| system_extended_legacy | 1 | gpui-08-jobs |
| system_info | 2 | gpui-09-environments |
| system_prompts | 4 | gpui-06-resources |
| ui_state | 6 | gpui-03-shell |
| unified_mcp | 4 | gpui-06-resources |
| usage_v2 | 18 | gpui-07-insights |
| waf | 4 | gpui-08-jobs |
| wsl | 8 | gpui-09-environments |

## 路由处理规则

2026-11-05：08 的 6 个 command_exec 与 8 个 install 入口已有源码集成检查点（14 项），其余 54 项仍未原生接入；这不是完成比例。命令自动订阅恢复、foreground/help 与 llmusage 安装纵向链的检查、native/未运行项见 [continuation-20261105.md](continuation-20261105.md)。foreground 使用可恢复流式语义，未宣称等价于 Tauri one-shot 输出合同。完整任务和父验收保持开放。

redirect 合并到目标功能而保留可发现性；布局根由 shell 所有；tray 由原生环境任务承接。
平台主页与数据页由 Insights 承接，设置/认证与资源页分域实现。现有只在命令层存在的能力也需明确 native 入口或书面归并理由。
没有 native_action 与 verification 的业务项不得标 complete。新增源命令/路由必须先对比源 hash 和差异，再更新范围。

## 手工核对重点

### 2026-11-06 连续实施检查点

本轮复用 09、01 和父任务，修复下列实际缺口，不重新打开已归档任务进行纯审计。此表区分已接源码和未完成原生验收；未映射的命令不因同模块有页面而自动完成。

| 功能 | 当前产品入口与 owner | 验收边界 |
| --- | --- | --- |
| Local / SSH 来源、主机与配置 | Settings → RemoteView → shared SSH Workspace；指纹确认、连接、固定四平台文件 SFTP 读写 | 内存 transport/隔离 fixture；真实 SSH、原生切换未运行 |
| 退出策略、主题和语言 | Shell preferences 与同一窗口关闭 guard；设置/菜单/快捷键 Quit | 默认仍每次确认；系统直接 App::quit 无 veto；托盘仍 unsupported |
| MCP 服务 CRUD | MCP editor → 固定 source/scope/revision → shared Claude policy/JSON/TOML codec | Claude/Codex/Antigravity；OpenCode 归平台源编辑；已接单项预设/兼容源复制，批量能力未冒领 |
| WebDAV 与文件夹 | Sync settings/folders → retained operations → ccr-sync | 普通明文文件夹合同；固定敏感资产 v2 加密入口已接；真实服务未运行 |
| 服务商、账户、签到、余额、记录 | Check-in operations → captured DbAccess/crypto → ccr-checkin | 复用加密与领域服务；浏览器 WAF/CDK/导入导出等其余命令未因此完成 |
| 便携包与开发参数 | desktop-package 双二进制哈希包；desktop-dev 原样转发 argv | 工程提交 `13b29f95`；真实 release 打包结果另行记录 |

本批已接 Claude 动态资源、Codex source-agent、指令模板、MCP 预设/复制、敏感资产和窗口状态。继续实施签到常用闭环与 OpenCode/Antigravity 文件资源；逐项证据见 [core-resources-20261106.md](core-resources-20261106.md)。全门禁和原生证据未齐前，父任务保持 in_progress。

Codex source-agent 安装/同步归资源任务；quota/session/usage 归 Insights；tray snapshot 归桌面集成。
ui_state 收藏/最近项归 shell；events 订阅机制归共享 runtime；update_ccr 与 WAF 处理归 jobs。
预算、定价、converter、output styles、statusline、builtin/system prompts 不能因没有显著导航而被丢弃。

### 最后业务接合批次（2026-11-06，待最终收据）

- OpenCode/Antigravity 的资源页面复用真实文件格式与目录，不用 raw settings 冒领动态文件。仅有 Tauri 列表能力的插件/扩展保持只读。
- 签到配置导出对应既有 `export_checkin_data`/`export_checkin_config` 同一 schema；它不是包含额外账户配置与历史的全库备份。统计导出单独核对，浏览器 WAF 登录不能由手工 cookie 管理替代验收。
- 预算/定价需要从 Shell 的 Usage 入口实际可达，且不能要求先存在用量数据库；已存在的独立组件测试不代表产品接通。
- Claude statusline/hooks 与 settings 源编辑器归并，路径及类型验证必须与共享 owner 一致；只保留 typed 验证，原 JSON 的未知字段继续保留。

本节是实际接合范围说明，最终状态由本批代码检查和原生收据更新，不勾选整域完成。

### 中断续作接合（2026-11-06）

`resume-summary-20261106.md` 汇总引用聊天的实际起点与本轮收尾；JSON 本次更新 33 项明确核对的命令映射，状态仍为 `implemented_partial_pending_native_acceptance`，保留旧记录及未核对项，不用数量推导完成比例。

- 平台：认证关闭、Codex 账户与模型供应商库、原有模型目录和显式配额查询。Grok 使用捕获根目录及保留轮换凭据的原服务；配额并非无副作用纯读取。
- 同步：canonical/legacy 原版本绑定的 WebDAV 解绑，以及已启用文件夹的批量推拉/逐项结果。
- 用量：已安装 CLI 的显式本地导入、应用持有的状态与取消；自定义 provider home 拒绝，按提供方细分的旧导入命令归并为默认本地全量操作，不宣称所有参数逐项等价。
- 会话：对已索引的 Codex 原来源导出 Markdown/复制、克隆、可恢复回收；回收有现有 CLI 恢复途径，不冒领导出任意文件保存或旧永久删除合同。
- 观测：Claude 用量/工具统计与订阅管理，主库与 archive 分开；工具采集接显式导入，共享既有 JSONL/repository owner，不新增 watcher。

新增操作的 fixture/headless 收据分见 `resume-{platforms,sync,usage,sessions,observer}-20261106.md`。旧 Windows 原生候选不覆盖这些新操作。稳定部分及最终完整检查见 `resume-check-20261106.md`；集中门禁与提交结果以 summary 为准。父任务、01、09 及原生/跨平台/性能归档硬门保持开放。
