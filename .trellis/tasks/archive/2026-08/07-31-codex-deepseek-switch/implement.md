# Implement — Codex DeepSeek 第三方接入支持

> rev2（2026-08-01）：按审阅核验修订——新增 A0（secret-aware 写入统一）与 B0（Tauri 桥接层），修复命令改 `fix --repair-runtime`，models.json 检查按 design D3 语义实现。
> 执行前置：按 `implement.jsonl` 清单加载 spec/research；改动语言约定：内部实现注释中文、公共 API 文档英文。

## 阶段 A：后端核心（crates/ccr-codex）

- [ ] A0 **Secret-aware 写入统一（design D5，先行）**：`managers/codex_config.rs` 的 `atomic_write` 改走 `ccr_core::AtomicWriter`（config.toml 与 auth.json 均 `secret(true)`，权限先于内容生效）；`backup_file` 弃 `fs::copy`，改读源 + `AtomicWriter.secret(true)` 写备份。动手前通读 atomic-writer spec，保留文件锁/TTL 缓存失效行为；补权限语义测试（Unix `#[cfg(unix)]` 断言 0o600，Windows 路径行为不回归）。
- [ ] A1 `models/codex_auth.rs`：`CodexProfileAuthMode` 新增 `ProviderBearerToken`（`"provider_bearer_token"`），补 `as_str` / 解析 / `openai_login_method()`（返回 `None`）及单测。
- [ ] A2 `platforms/codex.rs` 解析层：`canonical_auth_mode` 认识新值；`resolve_profile_auth_mode` 增判定（仅显式 auth_mode 触发，不自动推断，避免破坏现有 auto-promote 逻辑）；`normalize_auth_fields` bearer 分支实现**完整不变量**——派生 `preferred_auth_method="apikey"`、`forced_login_method="api"`（显式 platform_data 值优先）、`requires_openai_auth=false`、清 `env_key`/`openai_login_method`。
- [ ] A3 新字段与承载：`resolve_model_catalog_json`（原样透传）、`resolve_preferred_auth_method`（值域 `apikey|chatgpt` 归一，非法值 `ValidationError`）；`SwitchSpec` 增两个非密字段；**`AuthSelection` 新增 `WriteProviderBearerToken(Secret)`**（维持不派生 Debug，`RouteSelection` 不新增凭据字段）；`build_switch_spec` 组装 + bearer 校验（auth_token/base_url 必填、Official 路由拒绝）。
- [ ] A4 写入/清场：`apply_common_settings` 写/删两个根级键；`apply_switch_spec`——provider 表按 `spec.auth` 匹配 bearer 变体写 `experimental_bearer_token`，auth.json 侧 bearer arm 执行 ClearOpenAi 等价清理；`apply_runtime_route_without_auth` remove 列表补两个根级键；models.json 存在性提醒按 design D3（检查副本展开 `~`，写盘保留原值；`ColorOutput::warning` + `tracing::warn`，不阻塞）。
- [ ] A5 诊断/修复：`parse_current_auth_intent`（bearer 意图，优先于 env_key）、`AuthIntent` 新变体、`diagnostic_route_status`（root 矩阵 +2 键）、`diagnostic_credential_status`（bearer arm：config.toml 实际值 vs secret，repairable=true）、`runtime_auth_source` 新枚举值（只报来源）、`spec_matches_runtime_without_auth`（含新根级键；bearer 值不在此比对）。
- [ ] A6 `services/codex_runtime_service.rs`：`persist_profile_secret` / `scrub_profile_secret_fields` / `build_env_export`（bearer 无 env 导出）新增 arm；`validate_profile` 补校验。
- [ ] A7 后端测试（`crates/ccr-codex` 单测 + `crates/ccr/tests/commands/codex_profile.rs`、`codex_fix.rs`）：
  - DeepSeek 形态 profile 切换 → config.toml 与官方样例语义等价（provider 段名 custom、派生字段齐全）、auth.json 无 OPENAI_API_KEY；
  - 幂等（连切两次稳定）；切走 / `off` 后三个新字段消失；四种存量 auth_mode 回归；
  - fix 链路：切换后 inspect 零漂移；篡改任一新字段后 **`ccr codex fix --repair-runtime`** 恢复（`--dry-run` 只预览）；
  - 明文边界断言：日志/诊断 JSON/status 输出不含明文 key；`Debug` 卫生 `rg` 核查（不整体打印配置对象）。
  - 测试环境用 `test_support::TestCodexEnv`（见 test-fixtures spec），直跑 cargo 时 `-- --test-threads=1`。

**验证（阶段门）**：`just fmt-check` → `just lint-strict` → `just test`。
**回滚点 R1**：A0 与 A1–A7 各自独立 commit；诊断幂等测试不过则不进入阶段 B。

## 阶段 B：Tauri 桥接层 + UI 同步（ccr-ui）

- [ ] B0 **Tauri 桥接层（发现 4/2）**：`src-tauri/src/commands/codex.rs` `EXPLICIT_PLATFORM_STRING_FIELDS` 补 `model_catalog_json` / `preferred_auth_method` / `forced_login_method`；profile DTO 投影补三个具名字段并从 `extra` 剔除；`write_codex_config` 与 `unified_mcp.rs::write_json_config`（codex 分支）改走 `AtomicWriter.secret(true)`；补命令层测试：新字段白名单/投影往返 + **flatten 保留回归**（bearer 在场时 Settings 保存 / MCP 增删改一轮，新键与 bearer 原样保留）。
- [ ] B1 类型：`src/types/codex.ts` 增 `'provider_bearer_token'`；`src/api/generated/codexAuth.ts` 按 `command-manifest.json` 生成流程再生成，禁止手改生成物。
- [ ] B2 编辑器：`src/utils/codexProfileEditor.ts` + `CodexProfileEditorModal.vue`——auth_mode 选项增 bearer（不入 DEPRECATED）；表单增 `model_catalog_json`；`preferred_auth_method` / `forced_login_method` 以派生默认态呈现、高级入口可覆盖；序列化单源规则扩展；往返不丢字段、不把 bearer 改写为 no_auth。
- [ ] B3 模板：DeepSeek 内置模板（`platforms.codex`：base_url `https://api.deepseek.com/`、model `deepseek-v4-flash`、名称/官网/apiKeyUrl），mapper 仅契约允许的非密字段；模型非三预置 → custom model path 且不写全局 custom-models.toml；i18n zh-CN/en-US。
- [ ] B4 UI 测试：`tests/codex-profile-editor.smoke.test.ts`（新字段往返 + auth_mode 不回落 + 派生态展示）、`tests/provider-templates.smoke.test.ts`（DeepSeek 模板可见性/无密钥断言）；触及 catalog JSON 则加 `providers-catalog.smoke.test.ts`；对照 profiles-page-contracts / raw-config-editor-contracts 核查既有 smoke 是否需要同步。

**验证（阶段门）**：`cargo test -p ccr-ui-tauri`（或仓库对应命令）+ `just frontend-check-quick`；针对性 smoke：`cd ccr-ui && bun run test:smoke -- tests/codex-profile-editor.smoke.test.ts tests/provider-templates.smoke.test.ts`。
**回滚点 R2**：B0（Rust 桥接）与 B1–B4（前端）分开 commit，可独立 revert。

## 阶段 C：示例、文档与收尾

- [ ] C1 `examples/codex/config.example.toml`（或新增 `config.deepseek.example.toml`）+ `docs/examples/codex-cli-config.toml`：DeepSeek 形态示例，占位密钥，注明 models.json 获取方式（官方脚本/文档链接）、Codex >= 0.144.0。
- [ ] C2 docs 用户文档（中英对应位置）补 DeepSeek 接入小节，**明确披露**：config.toml 及其备份含密、config.toml 属 `codex-config` 加密同步资产（v2 加密信封）bearer 会随之同步；`cd docs && npm run build` 验证。
- [ ] C3 评审门（design §7，三个全部必审）：`rust-security-reviewer`（bearer 分层/D5 写入契约/明文边界）→ `tauri-ipc-reviewer`（B0 白名单/投影/写入）→ `frontend-quality-reviewer`（B1–B4）。评审发现全量输出、不预过滤严重级。
- [ ] C4 末轮全量检查：`just ci`（重闸，最终验收）；fmt/version-sync 产生修复性 diff 先审再继续。

## 人工验收（不阻塞合并）

- [ ] 用户以真实 DeepSeek key 建 profile → 切换 → `codex` 启动显示 `model: deepseek-v4-flash` 并完成一次对话；切回原 profile 无残留。

## 全局回滚

单 PR revert；运行时残留用 `ccr codex off` 清场（A4 保证新字段在其 remove 覆盖内）。
