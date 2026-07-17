# Design — 系统配置管理完善(settings.json / config.toml 分层与 raw 编辑)

> 对应 prd.md;父任务契约 C1–C6 全部落到本设计。本任务交付两个共享前置物:ccr-core versioned 写入 API、共享编辑器组件。

## D1 ccr-core:versioned 写入 API(契约 C1)

位置:`crates/ccr-core/src/core/guarded_write.rs`(同文件扩展,不新建模块)。

```rust
/// 版本令牌 = blake3(文件字节) 的 hex;目标文件不存在时令牌为空串 ""。
pub fn content_version_token(bytes: &[u8]) -> String;   // blake3 hex(64 chars)

/// 锁内 CAS 写入:同一把路径锁内 读当前内容 → 比对令牌 → 备份 → 原子写。
/// expected_token == "" 表示期望文件不存在(首次创建语义)。
/// 令牌不匹配 → Ok(VersionedWriteOutcome::Conflict),不落盘、不备份。
pub fn write_guarded_versioned(
    path: &Path, bytes: &[u8], expected_token: &str, opts: &WriteOptions,
) -> Result<VersionedWriteOutcome>;

pub async fn write_guarded_versioned_async(...);        // spawn_blocking 包装,同 write_guarded_async
```

实现要点:

- **锁不可重入**:把 `write_guarded` 现有锁内主体抽成私有 `fn write_locked(target, bytes, opts)`;`write_guarded` 与 `write_guarded_versioned` 各自取锁一次后调用它。versioned 变体在锁内先 `fs::read(target)`(NotFound → 视为空令牌)再比对。
- `CcrError` 已由 `.trellis/spec/ccr-core/backend/ccr-error-freeze.md` 冻结,不得新增变体。新增 `VersionedWriteOutcome::{Written, Conflict}` 作为成功通道上的预期结果;I/O、锁、join 等异常仍走既有 `CcrError`。Tauri 层据此映射结构化 status。
- 依赖:ccr-core `Cargo.toml` 增加 `blake3.workspace = true`(root 已有 `blake3 = "1.8.5"`)。
- 单测(同文件 tests mod):令牌匹配写入成功并返回 `Written`、不匹配返回 `Conflict` 且文件未动且无备份产生、空令牌首建成功/文件已存在时空令牌拒写、并发 4 线程 CAS 下无覆盖丢失(失败方收到 `Conflict`)、备份轮换不回归。

## D2 Tauri 层:raw 读写命令

新文件 `ccr-ui/src-tauri/src/commands/settings_raw.rs`,注册进 `handler_registry.rs`(config 域或新 `settings_raw` 域)。

### 结构化返回协议(预期失败不走 Err)

所有命令 `Result<Value, String>`;`Err` 仅留给意外 IO/锁/join 错误。预期结果用 `status` 判别:

```jsonc
// get_*_raw_text →
{ "status": "ok", "content": "...", "token": "<blake3hex|''>", "path": "C:\\Users\\..", "exists": true }
{ "status": "unsupported_environment", "envType": "wsl" }

// save_*_raw_text(content, token) →
{ "status": "saved", "token": "<新令牌>" }
{ "status": "conflict" }
{ "status": "invalid", "kind": "syntax" | "semantic", "message": "...", "line": 3, "column": 14 }  // line/column 可缺省
{ "status": "unsupported_environment", "envType": "ssh" }
```

### 命令清单

| 命令 | 目标 | 校验链 |
| --- | --- | --- |
| `claude_get_settings_raw_text` | `~/.claude/settings.json` | — |
| `claude_save_settings_raw_text` | 同上 | `serde_json::from_str::<Value>`(必须 object,取 `line()/column()`)→ `serde_json::from_value::<ccr_types::ClaudeSettings>`(semantic)→ D1 CAS |
| `codex_get_config_raw_text` | `~/.codex/config.toml`(`codex_config_path()`) | — |
| `codex_save_config_raw_text` | 同上 | `toml::from_str::<CodexConfig>`(flatten other 保未知键;`toml::de::Error` span → 行列)→ D1 CAS;成功后 `invalidate_codex_dashboard_overview_cache` |
| `claude_list_settings_layers` | 分层探测 | 只读,见 D3 |
| `codex_list_config_layers` | 分层探测 | 只读,见 D3 |

关键语义:

- **保存写入的是用户原文 verbatim**(不经反序列化-再序列化),TOML 注释/键序/JSON 手写格式全保留;语义校验只做"能否解析",不做规范化。
- **Local 环境门禁**(C2):helper `async fn ensure_local_env(state) -> Result<(), Value>`——`state.env_registry.read().await.active()` 的 `env_type() != EnvironmentType::Local` 时返回 `unsupported_environment` payload。get/save 双端都挂(后端为纵深防御,前端按环境 store 预先禁用入口)。
- **备份策略**:`BackupPolicy::Dir { dir: PlatformPaths::new(Platform::Claude|Codex)?.backups_dir, prefix: "settings.json" | "config.toml" }`——复用 profiles 既有 `~/.ccr` 备份目录约定,不在 `~/.claude`/`~/.codex` 里堆 .bak。
- **日志红线**(C3):错误路径只记录 path 与错误类别,`message` 取 serde/toml 的错误描述(不含原文行内容);禁止 `tracing` 输出 content。单测断言 invalid payload 的 message 不包含探针内容(在测试 fixture 里放独特标记字符串验证)。

## D3 分层探测(只读)

返回 `{ "layers": [ { "id", "label", "path", "exists", "size", "mtime", "editable" } ] }`:

- Claude:`user`(`~/.claude/settings.json`,editable=true)+ `managed`(Windows `C:\ProgramData\ClaudeCode\managed-settings.json`、macOS `/Library/Application Support/ClaudeCode/`、Linux `/etc/claude-code/`,exists 探测,editable=false)+ `project`/`local` 两个说明性条目(path=null,exists=null,前端展示"需项目上下文,本工具不管理")。
- Codex:`user`(`~/.codex/config.toml`,editable=true)+ glob `~/.codex/*.config.toml` 逐个生成 `profile_overlay` 条目(editable=false,只读列出)。

## D4 修复既有裸写(R4)

`LocalEnvironment::write_config`(`platform/local.rs`)改为:`create_dir_all` 后调 `ccr_core::core::write_guarded_async(path, bytes, WriteOptions { backup: SameDir { tag: Some("ccr_ui".into()) }, ..Default::default() })`。

- 决策:表单模式 merge-patch **本期不做 versioned CAS**(需改动全部表单命令签名,收益低——字段级合并冲突面小),记录为已知限制;raw 模式是并发安全兜底。
- WSL/SSH `write_config` 等价保障不做,`platform/wsl.rs` 注释留痕。
- SameDir(而非 Dir)理由:`write_config` 是通用多平台入口,`resolve_config_path` 的平台映射与 `ccr_config::Platform` 不一一对应(如 opencode),SameDir 无需路径推导;备份就近可见。

## D5 共享编辑器组件(契约 C4)

`ccr-ui/src/components/editor/CodeSourceEditor.vue`:

- Props:`modelValue: string`、`language: 'json' | 'toml' | 'markdown'`、`readonly?: boolean`、`errorMarker?: { line: number; column?: number; message: string } | null`(后端 invalid 回包直接映射,编辑器滚动定位并加行内 lint 标记)。
- Emits:`update:modelValue`、`save`(Ctrl/Cmd+S)。
- **选型:CodeMirror 6**,细粒度包:`@codemirror/state`、`@codemirror/view`、`@codemirror/commands`、`@codemirror/language`、`@codemirror/lint`、`@codemirror/search`、`@codemirror/lang-json`、`@codemirror/lang-markdown`、`@codemirror/legacy-modes`(TOML 走 StreamLanguage)。
  - 理由:错误行列锚定 + lint 面板是 C6/UX 硬需求,textarea 方案无法做行内定位;CM6 模块化后 gzip ~100–150 KB。
  - 加载:组件内 `onMounted` 动态 `import()`,与消费视图的路由级懒加载叠加 → 主包零增量;加载中显示骨架占位。
  - 备选(否决记录):textarea + 行号叠层——零依赖但无错误锚定/高亮,大文件编辑体验差。
- 主题:单个 CM theme 从既有 CSS 变量(`styles/tokens.css`)取色,明暗自动跟随;无动画(reduced-motion 天然兼容)。
- 前端不持久化内容(C3):纯 props/emit,无 store/localStorage;消费方负责卸载时丢弃。
- 测试:vitest smoke——挂载渲染、v-model 往返、errorMarker 定位调用(mock CM 视图)。

## D6 Settings 视图双模式

- `ClaudeCodeSettingsView`:既有 `activeTab`(model/permissions/env/ui/sandbox/git)追加 `source` tab。`CodexSettingsView` 若同构则同法,否则 header 加 segmented 切换(实现时按实际结构,不动既有表单交互)。
- `source` tab 状态机:`idle → confirming(requestConfirm 明文警示) → loading(get) → editing → saving → saved | conflict | invalid`。
  - conflict:提示"文件已被外部修改",动作仅 [重新加载] / [取消],无静默覆盖。
  - invalid:errorMarker 传编辑器,消息条展示 kind + message。
  - 环境非 Local(environment store):source tab 禁用态 + tooltip 原因文案。
  - raw 保存成功 → 触发表单态 reload;表单保存成功且 source 已加载 → source 标记过期(顶部条 + [重新加载])。
  - 离开有未保存修改 → requestConfirm。
- "配置层级"面板:两视图各加一块折叠卡片,消费 D3 数据,按优先级排序,不可编辑层灰显。
- 表单缺失字段清单(PRD R5 遗留决策):**本期不补任何表单字段**——摸底未发现造成数据破坏的表单缺口,raw 模式即完整兜底;该决策冻结,避免范围膨胀。

## D7 前端 API 与 i18n

- `src/api/domains/claude.ts` / `codex.ts` 追加 raw 与 layers 包装;TS 判别联合类型(`RawFileGetResult` / `RawFileSaveResult` / `ConfigLayer`)定义在各 domain 文件并经 `api/index` 导出;不触碰 `api/tauri.ts`。
- i18n:`settingsRaw.*` 命名空间(警示、冲突、无效、环境禁用、层级面板),zh-CN / en-US 同步。

## 决策记录汇总

| # | 决策 | 理由 |
| --- | --- | --- |
| 1 | 令牌 = blake3 内容哈希,空串表示"期望不存在" | mtime 不可靠;首建语义显式化 |
| 2 | core 冲突走 `Ok(VersionedWriteOutcome::Conflict)`,Tauri 预期失败走 `Ok + status`;`Err` 仅意外错误 | 遵守冻结 `CcrError` 契约,同时让前端可靠区分冲突/校验/环境 |
| 3 | raw 保存 verbatim 原文,不规范化 | 保留注释/键序,尊重手工编辑 |
| 4 | 表单模式不做 CAS(仅锁+备份+原子写) | 改动面/收益比差;记录为已知限制 |
| 5 | 编辑器 = CodeMirror 6 懒加载 | 错误行列锚定硬需求;主包零增量 |
| 6 | raw 备份进 `~/.ccr` backups_dir;`write_config` 用 SameDir | 前者复用 profiles 约定;后者平台映射不齐 |
| 7 | 本期不补表单字段 | raw 兜底;控制范围 |
