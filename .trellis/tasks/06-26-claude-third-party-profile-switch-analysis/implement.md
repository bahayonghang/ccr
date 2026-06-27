# 实施计划：Claude 第三方 Profile 官方配置补齐

## Preconditions

- 当前任务仍为 `planning`。只有用户明确同意后才运行 `python ./.trellis/scripts/task.py start 06-26-claude-third-party-profile-switch-analysis` 并开始改代码。
- 实施前先让用户贴出截图里的报错原文，或重新执行最小复现命令记录 stderr/stdout。
- 不写真实智谱 API key；如需本机验证，使用临时 fixture 或用户手动配置后的只读检查。

## Checklist

1. 复现和锁定当前问题
   - 读取 `ccr claude profile current --json`、`ccr current --json`、`ccr doctor --json`。
   - 对比 active profile、`profiles.toml` 期望 env、`~/.claude/settings.json` 实际 env。
   - 记录截图报错文本。

2. 扩展后端 profile schema
   - 在 `crates/ccr-config/src/managers/config/types.rs` 和 `crates/ccr-config/src/models/platform.rs` 增加运行时 env typed 字段。
   - 更新 `crates/ccr-config/src/platforms/base.rs` 双向转换。
   - 更新 `crates/ccr-cli/src/managers/settings.rs` 常量、`update_from_config`、`clear_managed_vars`、测试。
   - 更新 `crates/ccr-cli/src/platforms/claude.rs::get_env_var_names`。

3. 增加 onboarding helper
   - 在 Claude profile apply 的 api_key/third-party 路径中幂等写 `~/.claude.json.hasCompletedOnboarding = true`。
   - 保留 OAuth/account 字段。
   - 增加 fixture 测试覆盖 missing/existing/invalid JSON 三类行为。

4. 更新 CLI / UI / TUI 表面
   - CLI profile JSON 输出新增字段。
   - Tauri `ccr-ui/src-tauri/src/commands/claude.rs` parse/patch/profile_to_json 支持新增字段。
   - UI type、form default、editor section、template patch 支持新增字段。
   - 如 TUI 只负责切换，不做编辑，则至少 current/detail 能显示诊断提示。

5. 更新 GLM preset / template
   - `ccr-ui/src/configs/providerPresets/claude.ts` 的 `zhipu-glm` 对齐官方推荐字段。
   - 如 provider template catalog 另有 GLM 条目，同步更新。
   - 保持 token 为空/占位，禁止真实 key。

6. 增强 doctor
   - 检测 placeholder token。
   - 检测 profile expected env 与 runtime settings mismatch。
   - 检测 GLM 1M 模型缺 compact window。
   - 检测 onboarding 缺失。

7. 更新文档
   - `docs/reference/platforms/claude.md`
   - `docs/en/reference/platforms/claude.md`
   - 如 troubleshooting 有第三方模型章节，同步更新。

## Tests

- `just fmt-check`
- `cargo test -p ccr-cli -- --test-threads=1`
- `cargo test -p ccr --test commands -- --test-threads=1 claude_profile`
- `cargo test -p ccr --test commands -- --test-threads=1 doctor`
- `cd ccr-ui && bun run test -- claude`
- `just frontend-check-quick`
- `just lint-strict`

## Manual Verification

不含真实 token 的 fixture 验证：

1. 创建临时 `CCR_ROOT` 和 `CLAUDE_CONFIG_DIR`。
2. 写入 GLM profile。
3. 执行 `ccr claude profile switch glm`。
4. 校验 `settings.json` env 等价于任务 PRD 的 AC1。

真实本机验证仅在用户同意后执行：

1. 备份 `C:\Users\lyh\.ccr\platforms\claude\profiles.toml`、`C:\Users\lyh\.claude\settings.json`、`C:\Users\lyh\.claude.json`。
2. 用户填入真实智谱 API key。
3. 执行 `ccr claude profile switch glm`。
4. 新终端运行 `claude`，若提示使用 API key 则确认。
5. 用 `/status` 和一次短请求确认实际 provider 生效。

## Rollback Points

- 后端 schema 改动后先跑 Rust 单测；失败则不继续 UI。
- UI 改动后先跑快速前端检查；失败则不改 docs。
- 本机真实验证失败时，用备份恢复三个配置文件。
