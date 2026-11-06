# 修复 Codex 文件登录账号保存误判

## Goal

修复未配置凭据存储模式时 CCR 错判 auto 导致官方已登录账号无法保存，并校正保存失败退出码

## Requirements

- R1: 缺少 config.toml 或其顶层未设置 cli_auth_credentials_store 时，已有合法 auth.json 可以直接保存为命名账号，无需改配置、重新登录或登出。
- R2: 显式 file 保持支持；显式 auto/keyring、未知值、错误类型、不可读或损坏配置不得被误当作可管理的 file 凭据。保留现有非文件存储保护，不新增钥匙串读取。
- R3: codex auth save 失败返回非零退出码，给出真实原因和针对性建议，避免重复错误输出或无意义登出建议。
- R4: 保留原始运行时凭据、配置、凭据扩展字段、私有文件权限、原子写入与重复账号保护；不得泄露凭据。

## Acceptance Criteria

- [x] 真实 CLI 二进制在隔离 HOME 中，以未配置 store 的合成 ChatGPT 登录保存 khanh 成功；校验快照内容、注册记录及原始文件未修改。
- [x] 无 config.toml 与有效 config.toml 缺少 store 均成功；显式 file 回归通过。
- [x] 显式 auto/keyring（即使已有 auth.json）、未知值、错误类型、损坏配置拒绝保存且不写账号。
- [x] 未登录、无效 auth.json、重复账号、其他保存失败返回非零；强制覆盖仍成功。
- [x] 默认存储判断的单元测试和 CLI 回归先失败后通过；相关 crate 测试、fmt 与 clippy 检查通过。

## Confirmed Facts

- 用户明确授权创建任务并修改 CLI；前一轮仅分析，本轮授权实施。无待决产品问题。
- 本机 ccr 7.5.0 / codex-cli 0.167.0，未设置 CODEX_HOME/CCR_CODEX_DIR，配置顶层 store 缺失，auth.json 存在；默认与显式 file 的官方 login status 均报告 Logged in using ChatGPT。
- models/codex_auth.rs:199 将缺省值设为 auto；services/codex_auth_service.rs:241 检测配置，:515 在读取 auth.json 前拒绝非 file；CLI commands/codex/auth/save.rs:26 拒绝分支返回 Ok。
- 官方示例 https://learn.chatgpt.com/docs/config-file/config-sample 明示 cli_auth_credentials_store 为 file (default)。认证文档 https://learn.chatgpt.com/docs/auth 明示 auto 优先系统存储、不可用时回退文件。

## Out of Scope

- 不读取系统钥匙串，不实现完整 Codex 配置层级/企业策略解析，不扩展桌面 UI。
- 不修改用户 ~/.codex 配置或真实凭据，不自动运行真实账号 save、login、logout；不推送代码。用户后续已授权提交相关改动并归档本任务。
- 不改其他命令的遗留退出码；共享默认值调整带来的调用方影响需检查。
- 保留并发任务的 ccr-desktop/tests/runtime_shutdown.rs 及其他无关修改。
