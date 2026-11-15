# ccr-cli — CCR 路由 Profile 管理与诊断技能包

面向已安装 [`ccr`](https://github.com/bahayonghang/ccr)（Claude Code Router CLI）的开发者，让 Claude Code、Codex、Cursor、Grok Build、OpenCode 等兼容 Agent Skills 的编码代理安全管理 **`claude`、`codex`、`grok`** 三平台的路由 Profile、脱敏导入导出（`ExchangeV1`）与运行态健康诊断。

## 一键安装

通过 `npx skills` 直接从官方仓库安装顶层 `ccr-cli` 技能包（仅安装面向最终用户的 `skills/ccr-cli`，不会误装仓库内部 `.github/skills/` 研发技能）：

```bash
npx skills add bahayonghang/ccr --skill ccr-cli
```

## 前提条件

1. 本机已安装并在 `PATH` 中可调用 **`ccr >= 7.5`**（可通过 `ccr --version` 验证）。
2. `ccr claude profile export --help` 与 `ccr claude profile switch --help` 均返回退出码 `0` 且支持 `--json`（技能首次执行时会自动完成零副作用双步探测；若本机版本过旧，Agent 会停止写操作并提示升级，绝不回退调用已废弃的旧命令）。
3. 若需为新建或编辑的第三方 Profile 注入 API Key / Token，请预先将凭据存入本机环境变量或受保护权限文件（`0600`），供非交互 `--auth-token-stdin` / `--api-key-stdin` 管道读取，或使用 `--env-key <ENV_NAME>` 引用环境变量名。

## 你可以直接这样说

安装完成后，在支持技能调用的 AI 编码代理中可直接用自然语言委派任务：

- **只读查看与跨平台诊断（L0 免确认）**
  - “用 `ccr` 看看我当前激活的 `claude` 和 `codex` Profile 是哪一个，顺便跑一次 `validate` 检查配置健康度。”
  - “列出 `grok` 平台下的所有 Profile，并检查是否存在缺密钥被禁用的条目。”
- **零明文接触新建或修改 Profile**
  - “在 `ccr` 里帮我创建一个名叫 `openrouter-sonnet` 的 `claude` Profile，地址用 `https://openrouter.ai/api`，密钥从我本机环境变量 `$OPENROUTER_API_KEY` 通过 stdin 传入，先不要切换。”
  - “帮我新建一个 `grok` Profile `xai-team`，使用环境变量名 `XAI_API_KEY`（不要落盘明文密钥），模型设为 `grok-4`。”
- **脱敏导出与干跑优先导入（L1 强制先 `--dry-run`）**
  - “把我当前的 `codex` Profile 脱敏导出到 `./codex-team.export.toml`。”
  - “先预演把 `./codex-team.export.toml` 导入到本机 `codex` Profile，告诉我 6+1 桶各自会新增、跳过或覆盖哪些项。”
- **显式授权切换或停用（L2 运行态门禁）**
  - “把我的 `claude` 当前激活 Profile 切换到 `openrouter-sonnet`，切换后验证状态卡是否健康。”
  - “停用当前的 `claude` 官方直连覆盖（执行 `claude profile off`），并告诉我是否还有残留外部环境变量抑制源。”

## 能力范围与安全边界

### 唯一教学命令面

- **能力自省**：`ccr --version`、`ccr <claude|codex|grok> profile <action> --help`
- **跨平台只读诊断**：`ccr platform list --json`、`ccr current --json`、`ccr validate --json`、`ccr doctor --json [--platform <p>]`
- **三平台 12 个非交互规范动作**（`ccr <claude|codex|grok> profile ...`，支持 `--json` 的命令一律传 `--json`）：
  `list`、`current`、`init`、`create`、`edit`、`switch`、`enable`、`disable`、`delete`、`off`、`export`、`import`

### L0 / L1 / L2 安全门禁与零接触密钥铁律

- **L0 只读与预演免确认**：只读列表、状态卡、诊断、`export --no-secrets` 与 `import --dry-run --json` 直接执行。
- **L1 导入必先预演**：任何 `profile import` 强制先跑 `--dry-run --json` 展示 6+1 桶计划；涉及 `--on-conflict overwrite` 或 `--replace` 时必须列出受影响 Profile 并获明确同意后才加 `-y`。
- **L2 运行态与破坏性保护**：`switch`、`enable`、`off`、`disable`、`delete` 需具备本轮明确意图；未经针对当前激活项的明确授权，绝不擅自附加 `--force` 或 `-y`。
- **密钥零接触**：绝不在聊天窗口索要明文 API Key，绝不在命令行参数拼接密钥字面量，绝不省略 `--no-secrets` 配合 `--stdout` 导出，也绝不读取 `profiles.db`、`profile_secrets.json` 或含密钥的交换文件。

### 明确排除项（Out of Scope）

- **不负责** `ccr` 仓库自身的源码开发、构建测试与 CI 修复（由仓库内部 `.github/skills/` 承担）。
- **不代跑** 官方订阅 OAuth 账号快照管理（`ccr <platform> auth ...` 登录、保存、切换、删除及导入导出）。
- **不启动** 全屏交互 TUI、桌面端或阻塞编辑器（裸 `ccr`、裸 `ccr claude` / `ccr codex`、`ccr ui`、`profile open`、`ccr temp`）。
- **不封装** `sync`、`update`、`clean`、`project init`、`skills`、`prompts`、`sessions`、`stats`、`budget`、`pricing`、`temp-token`，以及 `gemini`、`droid`、`qwen` 的 Profile 写操作。

## 本地校验与仓库门禁

在 `ccr` 仓库根目录运行自包含只读门禁（纯 Python 标准库，无外部依赖）：

```bash
just skill-pack-check
```

若本地环境装有 `qiaomu-meta` 校验套件（将 `$QIAOMU_META_DIR` 指向其技能目录），也可运行完整治理校验链（严禁运行外部发布器）：

```bash
python "$QIAOMU_META_DIR/scripts/validate_skill.py" skills/ccr-cli
python "$QIAOMU_META_DIR/scripts/export_skill_ir.py" skills/ccr-cli --output reports/skill-ir.json
python "$QIAOMU_META_DIR/scripts/trigger_eval.py" skills/ccr-cli --output reports/trigger-eval.json
python "$QIAOMU_META_DIR/scripts/release_check.py" skills/ccr-cli --phase local
```

## Troubleshooting

1. **能力探测提示 `ccr` 版本过旧或子命令缺少 `--json`**
   - **原因**：本机 `PATH` 中的 `ccr` 版本低于 `7.5`，尚未包含规范三平台 `profile export/import` 或 `switch --json`、`validate --json` 接口。
   - **解决**：升级本机 `ccr` 至最新版本后再试；切勿让 Agent 回退拼装旧版顶层命令。
2. **创建或编辑 Profile 时未传管道导致终端等待输入（TTY `rpassword` 阻塞）**
   - **原因**：直接运行 `ccr <platform> profile create ... --auth-token-stdin` 而没有标准输入管道时，CLI 会在交互式终端调用密码读取提示等待人工键入。
   - **解决**：由用户先在 Shell 环境变量或 `0600` 文件中准备好密钥，再以 `printf '%s' "$MY_API_KEY" | ccr ... --auth-token-stdin --json` 非交互传入；或在用户自己的终端运行 `ccr` TUI（按 `n`）手工录入。
3. **`profile import` 导入脱敏文件后新 Profile 显示为禁用（`disabled_missing_secret`）**
   - **原因**：`--no-secrets` 导出的 `ExchangeV1` 文件脱敏移除了内联凭据；导入需要凭据的认证模式时，`ccr` 会安全落盘为 `enabled = false` 并在 `missing_secrets` 中列出名称，防止激活残缺配置。
   - **解决**：使用 `printf '%s' "$MY_API_KEY" | ccr <platform> profile edit <name> --auth-token-stdin --enable --json` 非交互管道补齐凭据并启用，再按需执行 `switch`。
4. **`profile edit` 或 `profile import` 报错 `RevisionConflict`（CAS 修订号不匹配）**
   - **原因**：读取 `profile list --json` 之后，另一个终端、TUI 或桌面端修改了同一平台的 `profiles.toml` / `profiles.db`，触发 `--expected-revision` 乐观锁拦截。
   - **解决**：重新运行 `ccr <platform> profile list --json` 检查最新配置差异与新 `revision`，确认无冲突后再重试。
5. **命令返回退出码 `0`，但 Agent 仍提示配置异常并触发 `ccr validate --json`**
   - **原因**：`ccr current --json` 与 `<platform> profile current --json` 遵循状态卡契约，即使运行态漂移或配置损坏也会返回退出码 `0`，并将故障编码在 JSON 的 `"health": "error"` 或 `"invalid"` 中。
   - **解决**：查看紧随其后的 `ccr validate --json` 或 `ccr doctor --json --platform <p>` 诊断明细；若属 Codex `app-server` 残留或运行态漂移，可在确认后参考 [references/diagnostics-and-recovery.md](references/diagnostics-and-recovery.md) 执行恢复。
