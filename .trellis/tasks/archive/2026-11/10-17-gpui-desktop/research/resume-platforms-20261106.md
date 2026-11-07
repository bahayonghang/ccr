# 2026-11-06 平台操作续作

## 已接通的产品链

- Claude/Codex 账户页新增运行认证清除确认；使用捕获路径和既有回滚备份协议。Codex 保留保存账户及配置指针，auto/keyring 不冒充文件认证退出。macOS Claude 沿官方 logout owner，fixture 不启动该程序。
- Codex 新增重命名、API key 新账户、auth.json/账户导出合并导入的真实控件与确认。使用原 CodexAuthService 和 commit_account，不自动切换，不覆盖已有名称。输入保留在当前敏感会话；确认取消保留草稿，成功/离页清空。API JSON 类型错误不会静默降级成默认 base URL。
- Codex 供应商库使用原 model_providers.json owner，新增独立 Providers 页面的列表、保存和删除。界面不预填保存的 key。省略/空 key 保留原 key，更新保留 created_at；原 operation lock 内检查原 token，最终调用 guarded leaf versioned writer，旧客户端和外部源变化均不能绕过原版本。
- Grok 原全局 service 新增捕获 roots 构造；默认构造和 Tauri 签名不变。真实保存来源 scope、切换、删除、退出使用同一 Grok revision 和既有轮换凭据保存策略。所有嵌套 leaf locks 和 auth-off 回滚备份保持在捕获根；fixture 可以读取自己的保存账户，不再查询机器账户。界面只显示验证过的来源范围和账户名。
- Codex 新增选中账户、当前运行账户、全部保存账户的配额查询控件。沿原 CodexQuotaService，新增纯 from_dirs 构造；查询属于 retained job，UI 说明真实网络、凭据刷新和历史写入，不提供虚假取消。fixture 在构造配额 owner 前拒绝；远程环境继续被原环境门拒绝。只投影账户名、窗口百分比、时间和静态结果，不把原响应、错误文本、邮箱或 token 交给界面。
- Codex Models 页读取与 Tauri 相同的内置及 custom-models.toml 模型目录。现有名单、schema、清洗和合并逻辑提取到 ccr-app::platform_settings::codex，Tauri 仅委托纯函数，原路径、命令签名和载荷不变。目录不代表供应方远程可用性。

## 修改边界

Native：runtime/platforms.rs、runtime/platforms/{accounts,providers,quota}.rs、features/platforms/{mod,tests}.rs。

Owner：ccr-cli application/auth_off.rs、services/grok_auth_service.rs 和 accounts.rs；ccr-codex services/{codex_model_provider_store,codex_quota_service}.rs；ccr-app platform_settings/codex.rs。Tauri commands/codex.rs 仅共享目录模型及纯函数。

平台两个原大文件原有的 OpenCode TUI、hooks 和低栈渲染拆分是中断前工作，续作保留；不回滚其他代理改动。13 个侧栏入口未变。

## 实际检查

- `cargo test --manifest-path ccr-desktop/Cargo.toml --locked --lib platforms -- --skip export_bindings`：最终 exit=0，19 passed；默认并行。包括账户/认证与运行文件分离、外部版本变化拒绝、供应商 key 保留、Grok 轮换保存后退出/恢复、fixture 禁配额网络、模型目录及真实确认控件。
- 相同 rendered API-key 确认/写入/清空测试在进程内 `RUST_MIN_STACK=1048576` 单独运行：exit=0，1 passed。该控件树包含新增 Codex quota 控件；这是低栈 headless 回归，不是原生 OS 收据。
- `cargo test -p ccr-codex --locked --lib versioned_mutation_checks_original_token_under_the_owner_lock -- --skip export_bindings`：最终 exit=0，1 passed；使用根默认 target，sandbox 拒绝 lock 后通过 require_escalated 运行。
- `cargo clippy --manifest-path ccr-desktop/Cargo.toml --locked --lib -- -D warnings`：本轮尝试被同期 session_operations.rs:191 的 collapsible_if 阻断；平台文件无报告错误。集中全套 lint 由主会话处理。
- 修改文件已 rustfmt；最终 diff-check 由主会话整合门一起记录。

## 失败和边界

新 Grok fixture 初始缺少领域要求的 create_time，补齐合成凭据格式后通过，未放松身份规则。供应商更新初始遇到 Windows sandbox 长路径备份失败，fixture 使用同一目录的 canonical/verbatim 根后通过，未删除备份或绕过写入门。

根 owner 测试首次访问根 target lock 被拒绝；随后曾错误指定 `--target-dir ccr-desktop/target`，该非规范尝试已停止（exit=1），不认领验证、不删除混入的重建产物。之后回到根默认 target 的授权运行，实际通过如上；独立 target 隔离规则仍适用。

未运行 computer use、WSL/doctor、真实 SSH、真实认证、真实 quota 网络或真实用户文件操作。新控件无本轮原生验收收据；不能拿旧 Windows 页面巡检代替这批新操作。完整 CI、跨平台、IME、托盘及父任务归档门继续由主会话保留真实开放状态。

## 仍需区分

本批已消除认证关闭、Codex 核心账户/供应商库/模型目录和配额入口的本地代码缺口；外部 provider 登录、网络配额的实际返回及新控件的原生操作尚未验证。Codex 的进程/端口诊断工具不是本批新增动作，既有 OAuth 控制器的监听、取消、回调及 cleanup 行为不变；本收据不声称 Tauri 全命令逐项原生验收完成。

## 集中门禁后的 Grok Shell 回归更新

主会话完整 desktop-test 的库测试 161 项通过后，native_shell 仍断言旧状态“演示数据不读取 Grok 保存账户”。更新 `platform_routes_show_supported_local_status_without_secrets`：使用显式 fixture roots 和真实 Grok service 保存合成账户 `grok-fixture-only`，随后删除该 fixture 的运行凭据。实际 Shell Grok Accounts 页面必须显示该精确保存账户、同时报告未检测到运行凭据，继续拒绝显示 secret、登录成功、无关调试文案；原路由与其他平台检查保留。

`cargo test --manifest-path ccr-desktop/Cargo.toml --locked --test native_shell platform_routes_show_supported_local_status_without_secrets -- --skip export_bindings`：exit=0，1 passed、7 filtered。没有修改 HOME/USERPROFILE、全局凭据或其他代理产品代码，没有执行 CUA 或全套重复验证。
