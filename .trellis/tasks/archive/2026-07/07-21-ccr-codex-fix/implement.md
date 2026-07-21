# Implement — ccr codex fix

执行顺序自底向上：先域服务（可单测）→ 再 CLI 装配 → 再编排/渲染 → 最后校验。每步带验证与回滚点。

## 前置

- [ ] 读 `.trellis/spec/ccr-codex/backend/backend-guidelines.md`（auth 安全、错误、日志、测试、验证）。
- [ ] 读 `crates/ccr-codex/src/services/codex_auth_service.rs:1308` 既有 `cached_codex_processes()`（sysinfo 用法参考；注意本任务改用 `cmd()` 窄匹配而非 `name()`）。
- [ ] 确认 `crate::services::install_detect::which_on_path` 在 ccr-cli 内可见（`pub fn`）。

## 步骤 1 — 域服务 CodexProcessService

- [ ] 新增 `crates/ccr-codex/src/services/codex_process_service.rs`：
  - `CodexAppServer` / `TerminationKind` / `CodexAppServerCleanup` 结构（见 design §3.1）。
  - 纯函数 `is_codex_app_server(cmdline_lower: &str) -> bool`（design §3.2）。
  - `CodexProcessService::new()`（用 `CodexPaths::resolve()` 保持域一致；即便本命令不强依赖路径，也统一构造）。
  - `find_app_servers()`：`sysinfo::System` 刷新 → `cmd()` join/lower → 分类 → best-effort user_id 过滤（design §3.3/3.4）。
  - `cleanup(dry_run)`：SIGTERM（`kill_with(Signal::Term)`，`None`→`kill()`）→ Unix 轮询 ~3s → SIGKILL 存活者 → sleep ~1s 复检 respawned。
- [ ] `services/mod.rs`：`pub mod codex_process_service;` + `pub use`。
- [ ] `lib.rs`：re-export `CodexProcessService, CodexAppServer, CodexAppServerCleanup, TerminationKind`。
- [ ] 单测：`is_codex_app_server` 正反用例矩阵（`codex app-server ✓`、`codex exec ✗`、`codex resume ✗`、`codex ✗`、含 `ccr` ✗、`node .../codex app-server ✓`）。
- 验证：`cargo test -p ccr-codex -- --test-threads=1` → 分类器测试通过。
- 回滚点：删除新增文件 + 撤销 mod/lib 两处导出。

## 步骤 2 — clap 命令面

- [ ] `crates/ccr-cli/src/cli/subcommands/codex.rs`：`CodexAction` 新增 `Fix { dry_run: bool, json: bool }`（design §1；含中文 doc 示例）。
- [ ] `crates/ccr-cli/src/cli/dispatch.rs`：`CodexAction` 匹配臂新增 `Fix` 路由到 `commands::codex::fix::fix_command`。
- [ ] `crates/ccr-cli/src/commands/codex/mod.rs`：`pub mod fix;`。
- [ ] 检查 `ccr codex help` 来源（`cli/help_config.rs::configure_codex_command` / `cli/help.rs`）：若手工枚举子命令则补 `fix` 行；clap 派生则跳过。
- 验证：`cargo build -p ccr-cli`（先放一个 `fix_command` 空壳返回 `Ok(())` 以便编译）。
- 回滚点：撤销上述装配点。

## 步骤 3 — CLI 编排与渲染 fix.rs

- [ ] 新增 `crates/ccr-cli/src/commands/codex/fix.rs`，实现 `fix_command(dry_run, json)`（design §4/§5/§6）：
  - A 进程清理：调用 `CodexProcessService::cleanup` + `render_cleanup`。
  - B 环境提示：`render_env_hints`（`CODEX_HOME`/`OPENAI_BASE_URL` 显值，`OPENAI_API_KEY` 只显存在性）。
  - C doctor：`which_on_path("codex")`（无 → error + `exit(127)`）；`run_codex_doctor`（`tokio::process` + `timeout(30s)` + `--json`，失败降级 `codex doctor` 文本）；保存报告到 temp 并打印路径；`render_doctor` 高亮字段。
  - D 退出码：`respawned` 非空 → warning + `exit(2)`；否则 `Ok(())`。
  - 可选 `--json`：结构化输出（若评审纳入）。
- [ ] doctor 解析/渲染抽为纯函数（输入 stdout bytes/str → highlights/raw），便于单测。
- 验证：`cargo build`（全 workspace）通过。
- 回滚点：删除 fix.rs + 撤销步骤 2 装配。

## 步骤 4 — 测试补齐

- [ ] CLI 解析测试（`cli/definitions.rs` 既有风格）：`["ccr","codex","fix"]`、`["ccr","codex","fix","--dry-run"]` 解析成功。
- [ ] doctor 渲染纯函数测试：样例 `{"checks":..,"details":{...}}` → highlights 提取断言；非 JSON → 降级 raw 断言。
- 验证：`cargo test -p ccr --test commands -- --test-threads=1` 通过。

## 步骤 5 — 全量质量门（Phase 2.2 / trellis-check）

- [ ] `just fmt`（修复式，改动后看 diff）→ `just fmt-check`。
- [ ] `just lint-strict`（strict Clippy，含 panic gate）。
- [ ] `cargo test -p ccr-codex -- --test-threads=1`。
- [ ] `cargo test -p ccr --test commands -- --test-threads=1`。
- [ ] 跨平台编译确认：本机（Windows）`cargo build`；Unix 分支通过 cfg 审阅确保 `Signal`/`kill_with` 路径不使用 Windows 不存在的 API（sysinfo 已抽象，重点核对无裸 `libc`/`nix`）。
- [ ] 安全自审（可触发 rust-security-reviewer）：确认输出/日志无 token、无 `auth.json` 原文、`OPENAI_API_KEY` 未回显值。

## 步骤 6 — 手动冒烟（可选，Unix 优先）

- [ ] `ccr codex fix --dry-run`：仅列出，不终止。
- [ ] 无 codex 于 PATH 时：中文错误 + 退出码 127（`echo $?`）。
- [ ] 有 codex 时：doctor 报告生成、路径打印、关键字段高亮。

## 步骤 7 — 收尾（Phase 3）

- [ ] Spec 更新（trellis-update-spec）：在 `ccr-codex` 或 `ccr-cli` spec 记录「app-server 窄匹配契约 + 跨平台信号升级 + 退出码语义（2/127）」，防后续回归误杀。
- [ ] 文档：如需，补 `docs/` 命令参考中的 `ccr codex fix`。
- [ ] 提交（Conventional Commits）：`feat(codex): ✨ 新增 ccr codex fix 清理 app-server 并诊断配置`。

## 审查门（Review Gates）

1. **步骤 1 后**：分类器语义是否精确复刻 `codex.*app-server` 且零误杀（最高风险点）。
2. **步骤 3 后**：跨平台终止/升级与退出码是否符合 design §7/§8。
3. **步骤 5**：安全 + 全量测试全绿方可进入收尾。

## 完成记录（2026-07-21）

- 实现落地：
  - `crates/ccr-codex/src/services/codex_process_service.rs`（域服务 + 分类器单测 ×3）
  - `crates/ccr-cli/src/commands/codex/fix.rs`（编排/渲染/doctor + 纯函数单测 ×3）
  - 装配：`subcommands/codex.rs`、`dispatch.rs`、`commands/codex/mod.rs`、`ccr-codex` `services/mod.rs` + `lib.rs`
  - 解析测试 ×2：`cli/definitions.rs`
- **真实 schema 修正**：`--dry-run` 真机自测发现 `codex doctor --json`（schemaVersion 1, codex 0.144.6）的 `details` **嵌套在 `checks.<id>` 之下**，而非顶层。已据此重写 `extract_highlights`（顶层取 codexVersion/overallStatus + 遍历 checks.*.details），实测正确高亮 CODEX_HOME / config.toml / auth file / stored auth mode / model provider 等；脱敏保持（`stored API key: true`、feature flags `<redacted>`）。
- 质量门全绿：`just fmt-check`、`just lint-strict`（全 workspace）、`cargo test -p ccr-codex`（211）、`cargo test -p ccr --test commands`（53）、`cargo test -p ccr-cli --lib`（199）。Windows 编译 + 真机 dry-run（exit 0）验证通过。
- 未做（待用户确认）：Phase 3 spec 更新（3.3）、commit（3.4）。`--json` 命令级输出按简单优先未纳入。

## 关键风险

- **误杀**（最高）：分类必须窄化到 app-server；`codex exec/resume`/`ccr` 绝不命中 → 由步骤 1 单测锁死。
- **跨平台信号**：Windows 上 `Signal::Term` 不支持，依赖 `kill_with` 返回 `None` 回退 `kill()`；须在 Windows 实测/审阅编译。
- **doctor 挂死/旧版**：必须超时 + `--json` 降级，避免命令卡住或崩溃。
- **密钥外泄**：环境提示区 `OPENAI_API_KEY` 只显存在性；doctor 输出转发但不反脱敏。
