# Codex 平台配置指南

Codex（`codex`）平台采用“官方账号面（`auth`）”与“运行时路由面（`profile`）”分离的一等平台设计。完整 CLI 子命令与 JSON 契约见 [`codex` 命令参考](/reference/commands/codex)。

## 当前主路径

```bash
# 初始化并维护 Codex 配置库（~/.ccr/platforms/codex/profiles.db）
ccr codex profile init
printf '%s' "$OPENAI_API_KEY" | ccr codex profile create proxy \
  --base-url https://api.example.com/v1 \
  --auth-token-stdin
ccr codex profile open

# 账号与 Profile 路由切换
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch proxy
ccr codex profile current
ccr codex profile off

# 导出、导入与运行时诊断修复
ccr codex profile export --no-secrets -o codex-profiles.toml
ccr codex profile import codex-profiles.toml --dry-run
ccr codex fix
```

## `auth` 与 `profile`

- `ccr codex auth ...`：保存 / 切换 / 导入导出官方登录账号快照
- `ccr codex profile ...`：管理 `profiles.db` 配置库并将选定 Profile 写入 `~/.codex/config.toml` 与 `~/.codex/auth.json`
- `ccr codex profile off`：退出 Profile 模式，清除 CCR Profile 路由与运行期 `auth.json`，为 `codex login` 准备干净的官方运行时环境

## 关键路径

- 宿主运行时配置：`~/.codex/config.toml`
- 宿主运行期认证：`~/.codex/auth.json`
- Profile 权威配置库：`~/.ccr/platforms/codex/profiles.db`（`profiles.toml` 仅作 `ExchangeV1` 导入导出与 `profile open` 投影格式）
- 注册表指针：`~/.ccr/config.toml` 中 `[codex].current_profile`

## DeepSeek Responses API

DeepSeek 的 Codex 接入需要 **Codex >= 0.144.0**。当前可用模型为 `deepseek-v4-flash`；模型目录 `~/.codex/models.json` 不由 CCR 下载或覆盖，请先按 [DeepSeek 官方 Codex 接入文档](https://api-docs.deepseek.com/zh-cn/quick_start/agent_integrations/codex/) 生成。官方同时提供 [Shell 配置脚本](https://cdn.deepseek.com/api-docs/codex-deepseek-setup.sh) 与 [PowerShell 配置脚本](https://cdn.deepseek.com/api-docs/codex-deepseek-setup-en.ps1)。

在 `ccr-ui` 的 Codex Profiles 页面选择 DeepSeek Provider 模板（或在 CLI 中使用 `ccr codex profile create` + `--auth-token-stdin`），然后：

1. 认证模式选择 `Provider Bearer Token` 并安全录入 DeepSeek API Key。
2. 模型保留 `deepseek-v4-flash`，模型目录填写 `~/.codex/models.json`。
3. 推理强度选择 `high`，保存并应用 Profile。

模板只填充非敏感的 Provider、端点和模型，不保存或覆盖 API Key。保存后的 Profile 使用以下非密字段；Bearer 由 CCR 运行时凭据存储单独托管，分享用的交换 TOML 不应包含明文密钥。活体目录是 `profiles.db`：

```toml
[deepseek]
description = "DeepSeek"
base_url = "https://api.deepseek.com/"
model = "deepseek-v4-flash"
provider = "deepseek"
provider_type = "third_party_model"
wire_api = "responses"
auth_mode = "provider_bearer_token"
model_catalog_json = "~/.codex/models.json"
model_reasoning_effort = "high"
enabled = true
```

应用时 CCR 固定使用 `[model_providers.custom]`，并自动派生 `preferred_auth_method = "apikey"` 与 `forced_login_method = "api"`。最终宿主形态见 [`codex-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-cli-config.toml)。切换到其他 Profile 会替换上述字段。执行 `ccr codex profile off` 会删除根级 `model_provider`、CCR 管理的 `model_providers.custom`、其他 Profile 根字段和运行期 `auth.json`，但会原样保留 `model_reasoning_effort`。没有 Profile 指针、旧版入口快照或第三方运行时时，命令保持现有官方 `auth.json` 不变。若进程继承了 `CODEX_HOME` 且未设置 `CCR_CODEX_DIR`，命令还会清理默认的 `~/.codex`，避免只清沙箱而留下官方 `codex login` 仍会读到的运行期 `auth.json`。

::: warning 凭据与同步边界
`~/.codex/config.toml` 和 CCR 创建的 `~/.codex/backups/config.*.bak` 会包含明文 Bearer，不得提交、分享或作为普通诊断附件。`config.toml` 同时是敏感同步资产 `codex-config`：同步到 WebDAV 时 Bearer 会包含在 v2 加密信封内，需要独立的操作口令；它不会从同步内容中被剔除。
:::

## Runtime 诊断与修复

先切换到需要排查的 Profile，再运行诊断：

```bash
ccr codex profile switch future
ccr codex fix
```

`ccr codex fix` 会清理残留 `app-server`，并比较调用瞬间的注册表指针、`profiles.db` 配置库、`config.toml`、`auth.json` 与当前进程环境。结果分别报告 `process_state`、`runtime_consistency` 和 `provider_auth_validity`。默认不调用上游 `codex doctor`。

进程发现显式读取 cmdline 与 owner，只处理当前用户的 Codex `app-server`。清理期间按 `PID + start_time` 识别进程。TERM 之后每 300ms 重查，匹配目标已空则结束宽限；settle 里才出现的新身份记为 `respawned`，不补发 deadline KILL。每次信号前重新验证 owner 与 argv；输出只显示脱敏摘要。无法建立安全快照时会报告 `process_state = unavailable`，不会把未知状态当成 `clean`。

裸命令只做本地诊断，不重写宿主运行时，也不运行上游 doctor。发现可安全修复的本地漂移后，显式运行：

```bash
ccr codex fix --repair-runtime
ccr codex fix --dry-run --repair-runtime
ccr codex fix --doctor
```

`--repair-runtime` 只通过既有原子应用路径重放当前保存的 Profile，不修改或轮换保存的密钥。组合 `--dry-run` 时既不终止进程，也不写 `config.toml` / `auth.json`。`--repair-runtime` 不隐含 `--doctor`。

进程、runtime inspection/repair 和 doctor 是独立阶段。默认跳过 doctor。runtime 阶段不可用时以退出码 `1` 报告；进程仍存在或进程发现不可用时退出码 `2` 优先。只有传入 `--doctor` 且 `PATH` 中没有 `codex` 时才退出 `127`。

CCR 自己的对账不新增第三方凭据探测。需要上游健康检查时再运行 `ccr codex fix --doctor`，其具体检查行为由当前 Codex 版本决定。即使 `runtime_consistency = match`，`provider_auth_validity` 仍为 `not_checked`；若此时服务商返回 `INVALID_API_KEY`，应通过 `--auth-token-stdin` 更新该 Profile 保存的密钥，而不是反复清理 `app-server`。

## 历史同步补充

`ccr codex sync-history ...` 用于修复 provider namespace 切换后旧会话历史不可见的问题。跨官方与第三方 Profile 迁移时优先使用：

```bash
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history
```

bridge 模式只修复列表可见性；如果历史含 `encrypted_content`，CCR 会提示不能重加密，后续 continue/compact 仍可能受原账号或服务商加密边界限制。

## 相关文档

- [`codex` 命令参考](/reference/commands/codex)
- [平台支持概览](./index)
- [平台命令迁移](./migration)
