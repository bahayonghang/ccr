# Codex 平台配置指南

Codex 平台当前采用“账号面”和“运行时面”分离的设计。

## 当前主路径

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
ccr codex fix
```

## `auth` 与 `profile`

- `ccr codex auth ...`：保存 / 切换 / 导入导出 official auth 账号
- `ccr codex profile ...`：把 profile 写入 `~/.codex/config.toml` 与 `~/.codex/auth.json`
- `ccr codex profile off`：退出 profile mode，清除 CCR profile 路由与运行期 `auth.json`，为 `codex login` 准备干净的 official runtime

## 关键路径

- Runtime config：`~/.codex/config.toml`
- Runtime auth：`~/.codex/auth.json`
- Profiles：`~/.ccr/platforms/codex/profiles.toml`
- Registry pointer：`~/.ccr/config.toml` 中 `[codex].current_profile`

## DeepSeek Responses API

DeepSeek 的 Codex 接入需要 **Codex >= 0.144.0**。当前可用模型为
`deepseek-v4-flash`；模型目录 `~/.codex/models.json` 不由 CCR 下载或覆盖，请先按
[DeepSeek 官方 Codex 接入文档](https://api-docs.deepseek.com/zh-cn/quick_start/agent_integrations/codex/)
生成。官方同时提供
[Shell 配置脚本](https://cdn.deepseek.com/api-docs/codex-deepseek-setup.sh) 与
[PowerShell 配置脚本](https://cdn.deepseek.com/api-docs/codex-deepseek-setup-en.ps1)。

在 ccr-ui 的 Codex Profiles 页面选择 DeepSeek Provider 模板，然后：

1. 认证模式选择 `Provider Bearer Token` 并输入 DeepSeek API Key。
2. 模型保留 `deepseek-v4-flash`，模型目录填写 `~/.codex/models.json`。
3. 推理强度选择 `high`，保存并应用 profile。

模板只填充非敏感的 Provider、端点和模型，不保存或覆盖 API Key。保存后的 profile
会使用以下非密字段；bearer 由 CCR runtime secret store 单独托管，不应手工写入
`profiles.toml`：

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

应用时 CCR 固定使用 `[model_providers.custom]`，并自动派生
`preferred_auth_method = "apikey"` 与 `forced_login_method = "api"`。最终 runtime
形态见 [`codex-cli-config.toml`](https://raw.githubusercontent.com/bahayonghang/ccr/main/docs/examples/codex-cli-config.toml)。
切换到其他 profile 会替换上述字段。执行 `ccr codex profile off` 会删除根级
`model_provider`、CCR 管理的 `model_providers.custom`、其他 profile 根字段和运行期
`auth.json`，但会原样保留 `model_reasoning_effort`。没有 profile 指针、旧版入口快照或
第三方 runtime 时，命令保持现有 official `auth.json` 不变。

::: warning 凭据与同步边界
`~/.codex/config.toml` 和 CCR 创建的 `~/.codex/backups/config.*.bak` 会包含明文 bearer，
不得提交、分享或作为普通诊断附件。`config.toml` 同时是敏感同步资产
`codex-config`：同步到 WebDAV 时 bearer 会包含在 v2 加密信封内，需要独立的操作口令；
它不会从同步内容中被剔除。
:::

## Runtime 诊断与修复

先切换到需要排查的 profile，再运行诊断：

```bash
ccr codex profile switch future
ccr codex fix
```

`ccr codex fix` 会清理残留 app-server，并比较调用瞬间的 registry pointer、`profiles.toml`、`config.toml`、`auth.json` 与当前进程环境。结果分别报告 `process_state`、`runtime_consistency` 和 `provider_auth_validity`。默认不调用上游 `codex doctor`。

进程发现显式读取 cmdline 与 owner，只处理当前用户的 Codex `app-server`。清理期间按
`PID + start_time` 识别进程。TERM 之后每 300ms 重查，匹配目标已空则结束宽限；
settle 里才出现的新身份记为 `respawned`，不补发 deadline KILL。每次信号前重新验证
owner 与 argv；输出只显示脱敏摘要。无法建立安全快照时会报告
`process_state = unavailable`，不会把未知状态当成 `clean`。

裸命令只做本地诊断，不重写 runtime，也不运行上游 doctor。发现可安全修复的本地漂移后，显式运行：

```bash
ccr codex fix --repair-runtime
ccr codex fix --dry-run --repair-runtime
ccr codex fix --doctor
```

`--repair-runtime` 只通过既有原子应用路径重放当前保存的 profile，不修改或轮换保存的 secret。组合 `--dry-run` 时既不终止进程，也不写 `config.toml` / `auth.json`。`--repair-runtime` 不隐含 `--doctor`。

进程、runtime inspection/repair 和 doctor 是独立阶段。默认跳过 doctor。runtime 阶段不可用时
以退出码 `1` 报告；进程仍存在或进程发现不可用时退出码 `2` 优先。只有传入 `--doctor`
且 PATH 中没有 `codex` 时才退出 `127`。

CCR 自己的 reconciliation 不新增第三方凭据探测。需要上游健康检查时再运行 `ccr codex fix --doctor`，其具体检查行为由当前 Codex 版本决定。即使 `runtime_consistency = match`，`provider_auth_validity` 仍为 `not_checked`；若此时 Provider 返回 `INVALID_API_KEY`，应核验或更新该 profile 保存的 key，而不是继续清理 app-server。

## 历史同步补充

`ccr codex sync-history ...` 仍用于修复 provider namespace 切换后旧历史不可见的问题。跨官方/第三方 profile 迁移时优先使用：

```bash
ccr codex sync-history --bridge official-custom --dry-run
ccr codex sync-history --bridge official-custom --all-history
```

bridge 模式只修复列表可见性；如果历史含 `encrypted_content`，CCR 会提示不能重加密，后续 continue/compact 仍可能受原账号/ provider 加密边界限制。

## 迁移提醒

以下旧路径不再推荐：

- `ccr switch <profile>`
- `ccr platform switch codex`
- `ccr platform current`

改用：

- `ccr current`
- `ccr codex profile switch <profile>`
- `ccr codex profile off`
