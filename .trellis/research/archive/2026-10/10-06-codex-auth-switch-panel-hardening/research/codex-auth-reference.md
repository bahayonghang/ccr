# codex-auth（Loongphy）参考机制纪要

来源：https://github.com/Loongphy/codex-auth （main 分支，MIT，Zig 实现），2026-10-06 通过 GitHub API 与 raw.githubusercontent.com 读取。README 另经 WebFetch 摘要核对。

## 目的

CLI 工具，管理多个 Codex 账号：保存、切换、导入/导出、配额查看、别名。CLI/App 用户切换后需重启客户端才生效（README 明示）；免重启方案为 fork `codext`。API 刷新会把 access token 发给 OpenAI（wham/usage、accounts 端点），README 声明 ToS 风险。

## 本地状态布局（docs/implement.md）

- `<codex_home>/auth.json`（活动凭据）
- `<codex_home>/accounts/registry.json`（注册表，schema_version 迁移门）
- `<codex_home>/accounts/<account file key>.auth.json`（账号快照）
- `<codex_home>/accounts/backup/`、`accounts/auth.json.bak.YYYYMMDD-hhmmss[.N]`、`accounts/registry.json.bak....`
- `<codex_home>/sessions/...`（rollout 文件，本地用量来源）

## 机制 1：登录隔离（src/workflows/login.zig）

`codex-auth login` 不在真实 `~/.codex` 登录：
1. 建临时目录 `<codex_home>/accounts/login-<timestamp>` 作为 CODEX_HOME 运行 `codex login`；
2. 把临时 auth.json 复制为真实 `auth.json` 与账号快照 `<record_key>.auth.json`；
3. upsert 注册表并设 active。

效果：登录新账号全程不触碰真实 auth.json 的旧账号 tokens，不触发旧账号 token 刷新/轮换。

## 机制 2：活动账号持续回写（src/registry/account_ops.zig syncActiveAccountFromAuthWithImporter）

每次前台账号命令前执行：
1. 读取 `~/.codex/auth.json`，解析身份（email/plan/record_key）；
2. 按 record_key 匹配注册表；无匹配 → 复制 live 文件为新快照并新建账号记录；
3. 有匹配 → 更新 email/plan/auth_mode；`fileEqualsBytes` 比较快照与 live，内容不同则用 live 覆盖快照（捕获轮换后的 refresh_token）；相同则只加固文件权限（hardenSensitiveFile）；
4. `setActiveAccountKeyPreservingPrevious` 维护激活指针。

效果：活动账号的 tokens 一旦被 codex 轮换，下一次前台命令即回写快照；快照不会长期持有已消费的 refresh_token。

## 机制 3：身份模型（docs/implement.md）

- ChatGPT：`record_key = chatgpt_user_id::chatgpt_account_id`；account context 顺序 tokens.account_id → JWT chatgpt_account_id → organizations[].id（org- 前缀回退）；email 仅用于展示分组，不参与身份。
- API key：`account_key = apikey::<me.id>::<sha256(api_key)>`；密钥只存快照，不入注册表/文件名/展示；展示名用指纹 `sk-xxxxx***xxxx`。
- 快照文件名由 record_key 派生；含不安全字符时 base64url 编码。

## 其他已核实要点

- 备份仅在内容变化时创建；同秒冲突加 `.N`；保留最新 5 个；`clean` 白名单化只清理 accounts 目录。
- registry schema 迁移：`schema_version` 门；v2（active_email/email 键）在内存迁移后重写；更新版本拒绝加载（UnsupportedRegistryVersion）。
- 本地用量：扫描最新 rollout 文件 `event_msg`/`token_count`，取最后可解析 rate_limits；早于当前账号激活时间的 rollout 事件忽略；每账号记录已消费 rollout 签名；window_minutes 300→5h、10080→7d。
- `switch -`（回到上一个账号）：registry 顶层 `previous_active_account_key`，激活变更时维护；同账号重复激活不改 previous（docs/brainstorm/2026-05-31 设计文档）。
- 显示模型：按 email 分组、别名优先、相对时间；PLAN 取自 auth claim 再回退 usage 快照。

## 对 CCR 的映射（供子任务引用）

- 机制 2 ↔ CCR 缺失的观测点回写（子任务 1 核心）。
- 机制 1 ↔ CCR 无法拦截外部 `codex login`；替代为观测点最大化 + 切换前回写 + 窗口边界文档（子任务 1）。
- 机制 3 ↔ CCR 仅按 account_id 匹配的边界（子任务 1/3）。
- 备份/权限/迁移/清理 ↔ 子任务 3 审计维度。
