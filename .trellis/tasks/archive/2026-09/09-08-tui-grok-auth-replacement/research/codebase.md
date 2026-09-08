# 当前源码证据（2026-09-08）

只读研究，基线 dev @ d881f699。未启动 TUI、读取用户凭据或运行测试。

## 主体已经替换

提交 b4f5e462（2026-08-21）删除 OpenCode Auth TUI、专属 auth/quota/usage 服务和 CLI 命令组，并添加 Grok Auth。
- crates/ccr/src/main.rs:38：Grok launcher。
- crates/ccr-tui/src/tui/app.rs:47、586：variant 和构造。
- crates/ccr-tui/src/tui/mod.rs:199：专用入口。
- 历史任务 .trellis/tasks/archive/2026-08/08-20-claude-codex-grok-auth-off/prd.md，历史跨 UI/VS Code 范围不自动继承。

## 删除和改写

| 锚点 | 处理 |
| --- | --- |
| crates/ccr-config/src/managers/tui_config.rs:121 | OpencodeAuth 变体；138 映射；211 过滤删除 |
| crates/ccr-config/src/managers/tui_config.rs:362 | 旧断言及 366 迁移用例删除/改写 |
| crates/ccr-config/src/managers/tui_config.rs:399 | 语言 fixture 与 436/461/528/636 旧字符串替换；保留语言/主题/Usage 原断言 |
| crates/ccr-tui/src/tui/theme.rs:81 | 孤立字段，109/139 初始化，379/621 helper |
| crates/ccr-tui/src/tui/theme.rs:792 | 旧配色断言和文件头过时注释 |
| crates/ccr-codex/src/utils.rs:62 | OpenCodePaths 定义；253/266/272/284 专属测试 |
| .trellis/spec/ccr-tui/backend/backend-guidelines.md:152 | 双语范围文案换 Grok；252 的兼容规则限定既有 Usage |
| .trellis/spec/ccr-config/backend/backend-guidelines.md:266 | 默认列表从旧 OpenCode 更新 Grok，说明旧标识移除 |
| README.md:12 | 已删除 Auth 迁移宣传 |

OpenCodePaths/theme helper 全仓引用只有自身定义和测试。实现前再查引用，无需建立新抽象。

## 保留的有效功能

ccr-ui/src-tauri/src/commands/opencode.rs 的 providers/MCP/agents/commands/plugins/settings；crates/ccr-usage/src/source.rs:53、crates/ccr-usage/src/queries.rs:231 的日志/统计；docs/guide/ui-modules.md:12 的工具入口；Codex 仍使用的 openai_quota_core.rs；历史 changelog/归档。

## Grok 缺口与约束

crates/ccr-cli/src/services/grok_auth_service.rs:25 仅存在性观察，Path::exists 折叠部分 IO 错误。crates/ccr-tui/src/tui/grok_auth/app.rs:24 初始化吞错，63 刷新错误丢失，89 直接登出，91 刷新上抛。crates/ccr-tui/src/tui/grok_auth/ui.rs:28 忽略 ViewportMode，74 误把存在表示为已登录。

crates/ccr-tui/src/tui/theme.rs:21 已有布局边界；crates/ccr-cli/src/application/auth_off.rs:341 拥有 GROK_HOME 路径，350 复用删除写核；crates/ccr-tui/src/tui/app.rs:1317 附近主壳先处理全局/换页键。

当前规范：.trellis/spec/ccr-cli/backend/auth-off.md 和 grok-profile-runtime.md，约束共享写核、备份/回滚、MCP/profile 和既有“不解析 token”的能力边界。用户随后确认多账号，本任务将定点新增账号服务受控解析/存取的例外，profile/MCP 禁止触碰规则继续保留；新设计见 grok-multi-account.md 和账号服务子任务。

## 截图与安装

Get-Command ccr -All 仅发现 C:\Users\lyh\.cargo\bin\ccr.exe，18,633,216 字节，LastWriteTime 2026-09-08 13:01:07、创建时间 13:09:46，无 FileVersion/ProductVersion。不能由此识别构建 commit、安装方式或截图进程。

仅确认截图与源码不一致；旧二进制、旧会话、另一环境是候选解释。本轮不重装、不操作 UI、不检查真实认证文件。
