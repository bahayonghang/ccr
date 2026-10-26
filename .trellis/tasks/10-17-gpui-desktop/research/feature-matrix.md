# 功能迁移台账

由 command-manifest.json 与实际执行 flattenCatalog() 的输出建立，不靠 grep 猜动态路由。
基线为 340 通用 / 348 Windows commands、76 route records（含 18 redirects）。
完整逐项 owner、风险类别、来源路径与待补 native action/verification 见 migration-inventory.json。
所有项目当前均 planned，不代表任何功能已实现。执行时更新台账，不重新运行脚本覆盖证据。

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

redirect 合并到目标功能而保留可发现性；布局根由 shell 所有；tray 由原生环境任务承接。
平台主页与数据页由 Insights 承接，设置/认证与资源页分域实现。现有只在命令层存在的能力也需明确 native 入口或书面归并理由。
没有 native_action 与 verification 的业务项不得标 complete。新增源命令/路由必须先对比源 hash 和差异，再更新范围。

## 手工核对重点

Codex source-agent 安装/同步归资源任务；quota/session/usage 归 Insights；tray snapshot 归桌面集成。
ui_state 收藏/最近项归 shell；events 订阅机制归共享 runtime；update_ccr 与 WAF 处理归 jobs。
预算、定价、converter、output styles、statusline、builtin/system prompts 不能因没有显著导航而被丢弃。
