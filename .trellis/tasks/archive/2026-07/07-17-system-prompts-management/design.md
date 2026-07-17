# Design — 系统提示词管理(CLAUDE.md / AGENTS.md / GEMINI.md)

> 对应 prd.md;依赖 platform-settings-enhancement 的 versioned API 与共享编辑器。协议同其 design.md D2,本文只写差异。

## D0 平台支持级别冻结(PRD 要求,含核实证据)

核实时间 2026-07-17,开发机(Windows 11,四个 CLI 均在用)实测:

| 平台 | 目标文件 | 冻结结论 | 证据 |
| --- | --- | --- | --- |
| Claude Code | `~/.claude/CLAUDE.md` | **可编辑(P0)**;`~/.claude/rules/*.md` 只读列出(目录可不存在) | 文件存在(2.3K),官方文档 user memory 层 |
| Codex | `~/.codex/AGENTS.md` | **可编辑(P0)** | 文件存在(3.6K),官方全局 instructions 路径 |
| Antigravity | `~/.gemini/GEMINI.md` | **可编辑(P1)** | 文件存在(3.4K)于 `~/.gemini` 顶层;`~/.gemini/antigravity-cli/` 下无 GEMINI.md/AGENTS.md → Antigravity 沿用上游 Gemini CLI 的 memory 约定,settings 才在 antigravity-cli/ 子目录。残余风险:上游允许 `context.fileName` 改名,本期不解析该设置,页面固定管理 GEMINI.md 并在 UI 注明 |
| OpenCode | `~/.config/opencode/AGENTS.md` | **可编辑(P1),默认未创建** | 文件不存在但 `~/.config/opencode/opencode.json` 存在(目录约定成立,与 `opencode_config_dir()` 一致);走"未创建 + 一键创建"流程 |

四平台均有入口,无"本期不支持"平台。**路径解析强制约束**:Claude/Codex/Gemini 用 `dirs::home_dir()` 拼接;OpenCode 必须复用 `commands/opencode.rs` 的 `opencode_config_dir()`;禁止走 `ExecutionEnvironment::resolve_config_path`(其 opencode → `~/.opencode` 映射错误,gemini → `antigravity-cli/` 对 memory 文件不适用)。

## D1 后端:`commands/system_prompts.rs`

静态注册表驱动(平台 → 文件描述符列表):

```rust
struct PromptFileSpec {
    id: &'static str,        // "claude-user-memory" | "codex-agents" | "gemini-md" | "opencode-agents"
    label_key: &'static str, // i18n key
    editable: bool,
    resolve: fn() -> Result<PathBuf, String>,
}
```

命令(均挂 `ensure_local_env` 门禁,复用 settings_raw 的 helper 与 status 协议):

| 命令 | 返回 |
| --- | --- |
| `system_prompts_list(platform)` | `{ status:"ok", files: [{ id, label_key, path, exists, size, mtime, editable }] }`;Claude 平台额外附 `rules: [{ name, path, size }]`(`~/.claude/rules/*.md` 只读枚举,目录缺失 → 空数组) |
| `system_prompts_get(platform, id)` | `{ status:"ok", content, token, path, exists }`(不存在 → `exists:false, content:"", token:""`) |
| `system_prompts_save(platform, id, content, token)` | `saved / conflict / unsupported_environment`;Markdown 无语法校验;`content.len() > 64 KiB` → `{ status:"saved", warning:"size" }` 附带告警(不拒绝);Codex 文件在响应中附 `limit_hint: 32768`(project_doc_max_bytes 提示,仅展示) |
| `system_prompts_create(platform, id)` | 空令牌首建(内容 = `""` 或单行标题模板);已存在 → conflict |

- 写入:`write_guarded_versioned`;备份 `BackupPolicy::Dir`:
  - Claude/Codex/Gemini:`PlatformPaths::new(Platform::X)?.backups_dir`,prefix = 文件名(如 `CLAUDE.md`)。
  - OpenCode(`ccr_config::Platform` 无该变体):`~/.ccr/backups/opencode/`,prefix `AGENTS.md`(目录按需创建;命名与 Dir 策略格式天然一致)。
- 日志红线同 C3:内容不入 tracing;错误消息只含路径与类别。

## D2 前端

- **通用视图** `views/generic/SystemPromptsView.vue`,`props: { platform }`(仿 `PlatformMcpView` 模式),内含:
  - 文件卡片列表(label、路径、存在/大小/mtime;未创建 → "创建"按钮)。
  - 点击卡片展开编辑区:`CodeSourceEditor` markdown 模式;未保存标记;conflict → [重新加载];保存 toast;>64 KiB 告警条;Codex 卡片附 32 KiB 上限提示文案。
  - Claude 平台页顶部:memory 层级简表(managed → project → user → local 静态说明,标注"此处编辑 user 级")+ rules 只读列表折叠区。
  - 无明文警示确认(PRD R1 裁量:memory 文件非 secret 类;保留日志红线)。离开未保存 → requestConfirm。
- **路由**(`router/index.ts`,4 条,`meta: { depth: 2, group: '<platform>' }`):
  - `/claude-code/system-prompts`、`/codex/system-prompts`、`/antigravity/system-prompts`(group 'gemini',并加 `/gemini-cli/system-prompts` redirect 兜底)、`/opencode/system-prompts`(hideGlobalBackground 与同组一致)。
- **入口**:`ClaudeCodeView`、`CodexView`、`GeminiCliView`、`OpenCodeView` 各自子页导航数组追加 "系统提示词" 项(icon 建议 `FileText`/`ScrollText`)。
- API:`src/api/domains/systemPrompts.ts`(新 domain,类型复用 raw 判别联合基型);i18n `systemPrompts.*` 双语。

## D3 测试设计(C6)

- Rust 单测(注册表 resolve 用可注入 home 的内部函数变体 + tempdir):get/save 令牌冲突、create 首建 / 已存在 conflict、文件不存在分支、备份生成(含 OpenCode 专用目录)、大小告警阈值、错误消息不含探针内容。
- 前端:路由可解析 smoke(4 条 + redirect)、api-facade-boundary smoke、`test:i18n`。

## 决策记录

| # | 决策 | 理由 |
| --- | --- | --- |
| 1 | 四平台全部"可编辑"冻结 | 本机证据充分(三存在一约定成立);无需只读/隐藏档位 |
| 2 | Antigravity 固定管理 `~/.gemini/GEMINI.md`,不解析 `context.fileName` | 证据支持默认约定;改名属边缘配置,UI 注明即可 |
| 3 | 单一通用视图 + platform prop,不做四份页面 | 文件列表结构同构;Claude 特有区块按 platform 条件渲染 |
| 4 | 大小超限告警不拒绝 | memory 文件无硬 schema;Codex 32 KiB 是消费端截断而非写入约束 |
| 5 | 无明文警示确认(区别于 settings/profiles) | memory 文件设计上不含凭据;日志红线仍保留 |
| 6 | OpenCode 备份走 `~/.ccr/backups/opencode/` | `ccr_config::Platform` 无 OpenCode 变体,显式补齐目录而非硬造 PlatformPaths |
