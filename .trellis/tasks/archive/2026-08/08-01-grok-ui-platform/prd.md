# Grok UI 平台支持总控(首页/Profiles/可视化 Settings)

## Goal

在 ccr-ui 桌面端为 Grok Build 平台提供与 Claude Code / Codex 对等的管理界面,共三个页面:平台首页、Profiles 管理页、可视化 Settings 页。CLI 侧 Grok profile 运行时(`GrokPlatform`,spec `grok-profile-runtime.md`)已完整实现并有测试基线;本任务树只做 Tauri 命令桥接 + 前端页面。核心 crate 改动限定为**一个只读、无副作用的 activation inspection API**(2026-08-01 评审确认必需,详见父 design.md D8),其余核心 grok 领域逻辑不新增/不修改。

父任务持有需求集、任务地图与跨子任务验收标准;实现工作全部在子任务中进行。

## 源需求(用户 2026-08-01)

基于现有 Claude Code 与 Codex 的页面和功能,为 Grok 设计并实现:

1. **Profile 管理页**:profile 列表、切换、增删改、off(退出 profile 模式)。
2. **配置页(可视化修改)**:`$GROK_HOME/config.toml` 的表单化编辑 + 原文(source)编辑双模式。
3. **平台首页**:安装/版本状态、当前 profile、快捷入口。

## 任务地图

| 子任务 | 交付物 | 依赖 |
|---|---|---|
| `08-01-grok-tauri-commands` | src-tauri Grok 命令集(profiles CRUD/apply/off/raw、settings 读写/raw、版本探测与平台清单接入)+ 命令清单再生 | 无(最先做) |
| `08-01-grok-ui-home` | `GrokView.vue` 首页 + 全部前端接线(路由/侧边栏/子导航/i18n/类型/API domain 骨架/平台色) | grok-tauri-commands |
| `08-01-grok-ui-profiles` | `GrokProfilesView.vue` + grok 专属编辑弹窗/descriptor/过滤器 | grok-tauri-commands、grok-ui-home(接线与 API domain 骨架) |
| `08-01-grok-ui-settings` | `GrokSettingsView.vue` 可视化表单 + source tab | grok-tauri-commands、grok-ui-home(同上) |

依赖顺序写入各子任务 prd/implement;父子结构本身不是依赖系统。

## 范围边界

**In scope**:上述三页 + 支撑它们的 Tauri 命令、导航接线、i18n(中英双语)、类型与 API domain。

**明确的范围裁剪(2026-08-01 评审后)**:

- **Grok UI 整体 Local-only**:所有 grok 后端命令统一在命令入口做本地环境门控(非 local 返回 `unsupported_environment` 状态),前端在非 local 环境显示提示横幅。理由:`GrokPlatform` 直读写宿主机文件,而版本探测等走 active environment,混用会产生「远端版本 + 本地配置」的错配。
- **不做 profiles.toml 源码编辑器**(claude/codex 有,但非本次源需求):削减明文暴露面,并规避 drift 状态下 activation 守卫被绕过的风险(见「凭据边界」)。后续如需可单独立项。

**Out of scope**(后续单独立项):

- Grok Auth 页(`auth.json` 按 spec 永不读写,无认证管理面)
- MCP / Sessions / Slash Commands / Agents 页
- 用量分析(grok 无 usage 后端服务;首页不放 `PlatformUsageInsightPanel`)
- 托盘集成、`command_exec.rs` 白名单(UI 走原生 IPC)
- 把 grok 逻辑抽成独立域 crate(如 `ccr-grok`)的重构
- WSL/SSH 环境下的 Grok 管理

## 跨子任务验收标准

### 凭据边界(全部子任务一票否决项)

- [x] **结构化响应零明文**:所有结构化 DTO/IPC 响应(list/get/overview/settings typed)不含 `api_key` / `auth_token` 明文;凭据字段"只写不读"(编辑表单不预填明文,区别于 claude/codex 现状)
- [x] **唯一受控例外**:Settings 页 source tab 的 config.toml 原文读取,遵守 `raw-config-editor-contracts.md` 全部约束——Local-only 后端强制门控、进入前明文警告确认、内容不得进入 Pinia store/日志/监控字段/localStorage/路由 state
- [x] base_url 展示一律经 `safe_base_url_for_display`;**脱敏值永不作为写回值**(patch 语义:未提交 = 保留原值)
- [x] runtime `config.toml` 写入 `BackupPolicy::None`(不新增明文凭据副本)
- [x] `auth.json` / `mcp_credentials.json` 永不读写
- [x] 错误消息不回显凭据 TOML 原文
- [x] force delete 仅在 blocked-active 状态时才 off + 重删

### 状态机与数据完整性(2026-08-01 评审新增)

- [x] active/drift 判定统一走核心层新增的**只读 activation inspection**(`inactive | active | drifted | unsafe_missing_entry_state`),禁止用 `get_current_profile()`(有清指针副作用且 drift 时返回 None)作为 active intent 依据
- [x] Settings typed 保存为**字段级 patch + read/merge/CAS 重试(≤3 次,每次重新检查托管锁)**,不得整 section 覆盖;并发 apply/off 与外部编辑不丢失未知键/未知表
- [x] 改名(rename)采用「存新名 → apply 新名(若原为激活)→ 删旧名」顺序,部分失败返回结构化状态并有明确恢复路径
- [x] 跨层错误传递用结构化 status envelope(如 `deleted | blocked_active`),前端禁止错误文案字符串匹配

### 一致性

- [x] 遵守 `.trellis/spec/ccr/backend/tauri-handler-registry.md`、`.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`、`profiles-page-contracts.md`、`raw-config-editor-contracts.md`
- [x] i18n 中英双语同步(check-i18n 通过),文案聚合在顶层 `grok: {}` 命名空间
- [x] 交互模式与 Codex 页对齐:确认优先(apply/delete 弹 ConfirmModal + diff)、成功后整体重载、toast 走 uiStore

### 集成验收(父任务收尾评审)

- [x] 三个页面在侧边栏/子导航/面包屑中完整可达,与 Claude Code / Codex 入口并列;占位视图文件已删除且无引用残留
- [ ] 全链路手工冒烟:创建第三方 profile → 切换(检查 `~/.grok/config.toml` 写入)→ 首页显示当前 profile → **改名激活中的 profile**(验证存新→apply→删旧顺序)→ settings 可视化改 `[ui]`/`[session]` 字段并保存(验证未知表保留)→ source tab 冲突提示可复现 → 制造 drift(手改 config.toml)验证各页呈现与删除引导 → off → 删除
- [x] 非 local 环境(如 WSL)下三个页面显示 Local-only 提示且命令被后端拒绝
- [x] 门禁:`just fmt-check`、`just lint-strict`、`just test`、`just tauri-command-inventory-check`、`just tauri-bindings-check`、`just frontend-check-quick` 全绿
- [x] `docs/reference/tauri-command-inventory.md`(中英)含 grok 命令条目(由再生流程产出)
- [x] `grok-profile-runtime.md` spec 已补 activation inspection 签名与契约(3.3 spec update 完成)

## 集成验收记录(2026-08-01)

- 自动门禁通过:`just fmt-check`、`just lint-strict`、`just test`、`just tauri-command-inventory-check`、`just tauri-bindings-check`、`just frontend-check-quick`(117 files / 586 tests)与最终 `just ci`。
- 静态与自动化安全核验通过:结构化 Grok 响应 DTO 不含 `api_key`/`auth_token`;profile 序列化脱敏、Local-only 入口门控、raw 内容不进入 store/storage/log、CAS/托管锁/未知键保留/rename/delete 状态机均有聚焦回归。
- 路由与导航 smoke 覆盖 `/grok`、`/grok/profiles`、`/grok/settings`;`GrokPlaceholderView.vue` 已删除,`GrokPlaceholderView`/`grok.placeholder` 无引用。
- UNVERIFIED:真实桌面 DevTools 全量响应检查、临时 `GROK_HOME` 端到端写入/首次创建/drift 流程、Home/Profiles 的桌面与窄视口明暗主题走查、production WebView2 下 CodeMirror `style.sheet`/gutter 对齐。Settings 已完成 Web 预览四象限走查与 production build,但不能替代这些真实 Tauri 验收。

## 参考资料

- research/:`grok-backend-capabilities.md`、`tauri-command-layer.md`、`frontend-platform-patterns.md`
- spec:`.trellis/spec/ccr-cli/backend/grok-profile-runtime.md`(Grok 运行时契约,本任务树的行为权威)
- Grok Build 官方配置面:docs.x.ai/build/settings(settings 可视化字段范围依据)
