# 总体技术设计(跨子任务共享决策)

各子任务 design.md 只写本页面/本层的细节,共享决策以本文为准。
修订记录:2026-08-01 依据 Codex 审阅(9 条发现)修订 D2-D6,新增 D8(activation inspection)、D9(Local-only)。

## 1. 分层与数据流

```
Vue View (GrokView / GrokProfilesView / GrokSettingsView)
  → src/api/domains/grok.ts(手写 wrapper,运行时校验)
  → src/api/generated/*(handler_registry 再生的薄 client + capability manifest)
  → Tauri IPC → src-tauri/src/commands/grok.rs(#[ccr_tauri_command_macros::command],入口统一 Local-only 门控)
  → ccr::platforms::GrokPlatform(= ccr_cli::platforms::grok,经 ccr facade)
  → 文件系统($GROK_HOME/config.toml、~/.ccr/platforms/grok/*)
```

**决策 D1:不下沉共享 crate;核心层改动限定在 Grok 模块的只读 inspection API(D8),以及保存时保留 inactive 空指针。** 后者修复共享 serializer 自动选择首个 profile 导致 CRUD 制造激活意图的问题,不改变其他平台语义。src-tauri 经已有的 `ccr` facade 使用 `GrokPlatform`,零 Cargo.toml 改动。锁/CAS/入口状态/脱敏 helper 由核心层保证,Tauri 层是薄适配(`spawn_blocking` + DTO 映射 + 中文错误上下文)。

## 2. Tauri 命令清单(canonical,子任务不得私自增删名字)

模块 `grok:`(handler_registry,默认 risk `SecretMutation`,Generated)。**所有命令入口先做 Local-only 门控(D9)。**

| 命令 | 语义 | 备注 |
|---|---|---|
| `grok_list_profiles` | 列表 + current + activation state | 返回脱敏 DTO(D3) |
| `grok_get_profile` | 单个详情(供编辑表单) | 与 list 同形,凭据不回传 |
| `grok_add_profile` | 创建 | 复用 `validate_profile` |
| `grok_update_profile` | 更新/改名(patch 语义 D3) | 改名顺序见 D8.3;返回结构化 status |
| `grok_delete_profile` | 删除(可选 force) | 返回 `{status: deleted \| blocked}`(D8.2),无错误文案匹配 |
| `grok_apply_profile` | 切换 | 核心层持锁 |
| `grok_profile_off` | 退出 profile 模式 | grok 特有动作 |
| `grok_get_settings` | config.toml typed 读取 | 含 `custom_models` 脱敏摘要、`activation`、`managed_keys_locked`(D4) |
| `grok_update_settings` | 字段级 patch + CAS 重试(D4) | **非整 section 覆盖** |
| `grok_get_config_raw_text` / `grok_save_config_raw_text` / `grok_list_config_layers` | config.toml 原文编辑 | 唯一明文例外通道(D5) |
| `grok_get_dashboard_overview` | 首页聚合 | 无 version 字段,版本由前端单独调 `getCliVersion`(D6) |

**已删除**(2026-08-01 评审):`grok_get_profiles_raw` / `grok_save_profiles_raw` —— profiles 源码编辑不做(非源需求;规避明文暴露与 drift 下 activation 守卫绕过,见 `profile_lifecycle.rs:89` 对 `current_profile=None` 的静默放行)。

另改既有文件:`system.rs`(`CLI_VERSION_TOOLS` + 映射补 `grok`,program=`grok`)、`platform/local.rs`(`list_platforms`/`detect_cli_status`/`config_base_dir` 补 grok)。

## 3. 凭据与 DTO(决策 D3)

- 对外 DTO(list/get 共用):`{ name, description, provider, profile_kind: "official"|"third_party", base_url_display, has_base_url: bool, model, api_backend, context_window, supports_backend_search, reasoning_effort, auth_mode: "inline_api_key"|"env_key"|"session", env_key, has_inline_credential: bool, enabled, tags }`。**没有 api_key / auth_token 字段。**
  - `profile_kind` 由后端权威判定(存在 base_url/凭据 → third_party),前端**不得**从 base_url/凭据推断类型。
  - `base_url_display` 仅用于展示;**永不作为写回值**(会丢 query/userinfo)。
- **Patch 更新语义**(update 请求,区别于整量覆盖):
  - 普通字段:请求中**缺席 = 保留原值**;`null` = 清除该键;有值 = 替换。
  - `base_url`:同上——表单输入框留空即缺席(placeholder 显示 `base_url_display` +「留空保持不变」),填写即替换。
  - 凭据:`credential_action: "preserve" | "replace_api_key" | "replace_env_key" | "clear"` 枚举 + 对应值字段;缺省 = preserve。
- 编辑表单凭据/base_url 输入框永远空白起步,状态徽章显示当前形态(inline 已配置 / 环境变量名 / 会话认证)。

## 4. Settings typed 读写(决策 D4)

- `grok_get_settings` 响应:`{ exists, activation(D8), managed_keys_locked, models: {default?, default_reasoning_effort?}, ui: {theme?}, session: {auto_compact_threshold_percent?, load_envrc?}, cli: {auto_update?, channel?, show_tips?}, hints: {new_session_worktree_mode?, fork_worktree_mode?}, custom_models: [{id, name, model, base_url_display}] }`(custom_models 来自 `[model.*]` 表,只读脱敏摘要)。
- **`grok_update_settings` 请求 = 白名单字段 patch**:`{ set: { "<dotted.key>": value, ... }, unset: ["<dotted.key>", ...] }`,key 严格限于上面五个 section 的白名单字段;白名单外 key → 校验错误。此模型天然规避 serde 的 absent-vs-null 歧义,且不承载任何 section 整体。
- **写入路径 = read/merge/CAS 重试(≤3 次)**:每次尝试:读最新原文 + content token → 解析为 `toml::Value` 全文档树 → **重新检查 activation 与托管锁**(锁定时 set/unset 含托管键 → 拒绝并返回引导错误)→ 在全文档树上应用 set/unset(未知表/未知键天然保留,因为改的是完整文档而非 typed struct 替换)→ `write_guarded_versioned`(`secret:true`,`BackupPolicy::None`)→ conflict 则重试;3 次后返回 conflict 状态。
  - 理由:`AtomicWriter` 只保证原子替换不保证 RMW 事务;apply/off 可能在读取后修改文件,CAS 是并发正确性的唯一防线(与 grok spec 的 profile 写入同一套路)。
- **托管键锁定**:`activation ∈ {active, drifted, unsafe_missing_entry_state}` 时,`models.default` 与 `models.default_reasoning_effort` 锁定(drifted 也锁——恢复语义未决,应引导 off/修复);`inactive` 解锁。`[model.*]` 表永远不在 typed 面内。
- 注释丢失限于被修改文档的序列化(toml::Value 不保注释);UI 常驻说明,source tab 原文写入无此问题。

## 5. Raw 编辑范围与备份策略(决策 D5)

- **仅 Settings source tab 一个 raw 通道**(config.toml);profiles raw 已删除(D2)。
- 遵守 `raw-config-editor-contracts.md` 全部条款:后端每个 raw 命令强制 Local-only 检查(前端禁用只是 UX);进入前明文警告;内容不得进入 store/日志/监控/localStorage/路由 state;原文 verbatim 写入。
- 保存:`write_guarded_versioned` + content token CAS + **`BackupPolicy::None`**(grok spec:runtime 备份=新增未披露明文凭据位置;与 codex raw 的 `BackupPolicy::Dir` 不同,helper 需按 kind 区分策略,codex/claude 行为不变)。
- `grok_list_config_layers`:列出 user(`$GROK_HOME/config.toml`,可编辑)、project(`.grok/config.toml`,只读)、**managed(`~/.grok/managed_config.toml`、`/etc/grok/managed_config.toml`,只读)、requirements(`~/.grok/requirements.toml`、`/etc/grok/requirements.toml`,只读)**各层的存在性;存在 managed/requirements 时前端提示「用户设置可能被组织策略覆盖」。

## 6. 首页型态(决策 D6)

数据仪表盘型(仿 CodexView 简化,不含 usage 面板):

- 头部:Grok Build 标识 + 版本 chip(前端直调 `getCliVersion({tool:'grok'})`,四态;**仅 local 环境调用**)+ 当前 profile chip + auth_mode chip;drifted/unsafe 状态显示警示 chip。
- Readiness 三卡:安装状态 / Profiles(总数+当前)/ Config(存在性、activation 状态)。
- Next actions:未安装→安装指引;无 profiles→去创建;有 profiles 无激活→去切换;drifted→提示 off 或修复。
- 管理入口:Profiles / Settings 两行;常用命令 copy 列表。
- `GrokDashboardOverview` 响应:`{ activation, current_profile, auth_mode, profiles_total, profiles_enabled, config_exists, config_path_display }`(**无 version 字段**——避免与前端独立版本探测重复)。
- 非 local 环境:overview 返回 `unsupported_environment` 状态,页面整体显示 Local-only 横幅。

## 7. 前端接线(决策 D7)

- 路由:`/grok`(depth 1, group 'grok', cache+cacheKey)、`/grok/profiles`、`/grok/settings`(depth 2)。占位视图由 grok-ui-home 创建,profiles/settings 各自替换自己的 import;**占位文件删除动作归父任务集成评审**(最后确认无引用后删除)。
- 导航:mainLayoutShell 三 map + moduleSubnav `grok:` + `nav.grok`;平台色 tailwind `platform.grok` + 主题 CSS var。
- i18n:顶层 `grok: {}` 聚合命名空间,中英同步(`cd ccr-ui && node scripts/check-i18n.mjs` 校验,注意脚本在 ccr-ui/ 下)。
- API:`src/api/domains/grok.ts` 唯一 wrapper 位置;类型 `src/types/grok.ts` 以 ts-rs 生成物(`src/types/generated/grok/`)为准。**新增 ts-rs DTO 后必须跑 `just tauri-bindings` 再生 + `just tauri-bindings-check` 校验**。

## 8. Activation inspection 与状态机(决策 D8,2026-08-01 新增)

### 8.1 核心层新增只读 API(本任务树唯一核心改动)

`get_current_profile()` 不能作为 active intent:drift 时它返回 `None` 且有清注册表指针的副作用,而 `delete_profile` 仍会因 profiles.toml 意图/运行时形态拒绝删除(实测 `grok.rs` 测试 `drifted_inline_profile_cannot_be_deleted_before_off`)。因此在 `crates/ccr-cli/src/platforms/grok.rs` 新增:

```rust
pub enum GrokActivationState {
    Inactive,
    Active { name: String },
    Drifted { name: String },              // 注册表/profiles 意图存在但运行时不匹配
    UnsafeMissingEntryState { name: Option<String> },  // 入口状态缺失但意图/受管形态仍在
}
pub fn inspect_activation_state(&self) -> Result<GrokActivationState>
```

约束:**纯只读**——读原始注册表指针(不触发 drift 清理)、profiles `current_config`、运行时等价比较、入口状态存在性;零写入、零副作用;不暴露凭据。配套:单测(四态覆盖)+ `grok-profile-runtime.md` spec 更新(签名 + 契约,3.3 步骤)。UI 各处(删除引导、off 可用性、settings 托管锁、首页状态)一律以它为准。

### 8.2 delete 状态信封

`grok_delete_profile(name, force)` 返回 Ok 信封:`{ status: "deleted" } | { status: "blocked", reason: "active" | "drifted" | "unsafe_missing_entry_state" }`。force 路径:仅 `blocked(active|drifted)` 时 off → 重删;`unsafe_missing_entry_state` 永不自动处理(fail-closed,引导手工恢复)。前端按 status 分支,禁止错误文案匹配。

### 8.3 rename 顺序与部分失败

Claude 的「存新→删旧→re-apply」对 grok 不可行(删除激活项被核心拒绝)。Grok 顺序:

```
save_profile(new)
if 原名为激活(inspection Active{old}): apply_profile(new)   // 切换后 old 不再激活
delete_profile(old)
```

返回信封:`{ status: "renamed" }` | `{ status: "rename_apply_failed" }`(新旧并存、旧仍激活;恢复=用户重试切换或删除新名)| `{ status: "rename_cleanup_failed" }`(新名已激活、旧名残留;恢复=重试删除旧名)。每种状态带中文 message;前端对两种部分失败给出明确后续操作按钮。

## 9. Local-only 门控(决策 D9,2026-08-01 新增)

- 所有 grok 命令(含 typed settings 与 overview,不止 raw)入口统一调用现有 `ensure_local_env` 式检查;非 local 返回 `{ status: "unsupported_environment", env_type }` 信封或对应错误。
- 前端:`getCurrentEnvironment()` 非 local 时,三个页面渲染 Local-only 提示横幅并禁用操作;版本探测不发起。
- 理由:`GrokPlatform` 直操作宿主机文件,而 `system.rs` 版本探测/`detect_cli_status` 走 active environment(`system.rs:785` 缓存键含 env_id),不门控会出现「远端版本 + 本地配置」错配。

## 10. 兼容与回滚

- 纯增量功能:不改既有平台代码路径(system.rs/local.rs 只加分支;settings_raw/profile_lifecycle helper 改动保持 codex/claude 行为不变,新参数走默认值)。
- 核心层唯一改动 = D8.1 只读 API + spec 更新 + 测试;`just test` 全量回归护航。
- 回滚 = 移除 grok 路由与命令注册条目 + 再生 inventory/bindings;磁盘配置无迁移。
