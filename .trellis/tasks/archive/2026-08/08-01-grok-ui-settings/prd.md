# Grok 可视化 Settings 页面

> 父任务:`08-01-grok-ui-platform`。
> 依赖:`08-01-grok-tauri-commands`(契约冻结)、`08-01-grok-ui-home`(路由占位/API domain/i18n/平台色)。**与 grok-ui-profiles 串行,本任务在后**(共享文件,见父 implement.md)。
> 修订记录:2026-08-01 依据 Codex 审阅修订(字段级 set/unset patch;typed 保存 conflict UX;managed/requirements 层;Local-only;生产 CSP 验证)。

## Goal

新建 `GrokSettingsView.vue`,对 `$GROK_HOME/config.toml` 提供「可视化表单 tabs + source 原文 tab」双模式编辑,与 CodexSettingsView 同构;数据完整性由后端字段级 patch + CAS 保证,前端正确呈现托管锁与冲突状态。

## Requirements

### 可视化表单(父 design D4 白名单字段)

1. Tab「模型」:`models.default`、`models.default_reasoning_effort`(7 级下拉+未设置);`managed_keys_locked` 为 true 时(activation ∈ active/drifted/unsafe)这两项只读并显示托管提示条(「当前由 CCR profile 托管,请到 Profiles 页切换/off」+ RouterLink);`custom_models`(来自 get_settings 响应,后端已脱敏)以只读摘要卡展示,编辑指引到 source tab。
2. Tab「会话与界面」:`ui.theme`、`session.auto_compact_threshold_percent`(0-100 校验)、`session.load_envrc`(开关)。
3. Tab「CLI」:`cli.auto_update`、`cli.channel`(stable|alpha)、`cli.show_tips`、`hints.new_session_worktree_mode` 与 `hints.fork_worktree_mode`(ask|always|never)。
4. **保存请求 = `{set, unset}` 字段级 patch,只含被用户改动的白名单 dotted key**;「未设置」选项对应 unset。前端不发送任何 section 整体、不构造整文件 payload。
5. 保存结果分支:`saved` → toast + 重新拉取;`conflict`(后端 CAS 3 次重试后仍冲突)→ 冲突提示条 + 「重载最新值」动作,不静默覆盖;托管键拒绝错误 → 呈现引导文案。
6. 每字段带简短说明(docs.x.ai/build/settings 语义);页脚提示「更多配置节请使用源码模式」+「可视化保存会规范化被改动文档的排版(注释不保留),需保留注释请用源码模式」。
7. config.toml 不存在:表单默认态渲染,保存即创建。
8. **Local-only**:非 local 环境整页横幅 + 禁用(父 design D9)。

### Source tab

9. 复用 `ConfigSourcePanel`(language="toml"),接 `getGrokConfigRaw`/`saveGrokConfigRaw`/`listGrokConfigLayers`;明文警告、脏检查、离开确认、conflict/invalid 状态呈现全套继承。
10. 备份文案:grok 不承诺备份——涉及"已自动备份"的提示对 grok 替换为「此文件不做自动备份」(父 design D5)。
11. 配置分层面板:展示 user(可编辑)/ project / **managed / requirements**(均只读)各层存在性;managed 或 requirements 存在时提示「用户设置可能被组织策略覆盖」。

## Acceptance Criteria

- [ ] 手工冒烟(临时 `GROK_HOME`,config.toml 含 `[mcp_servers]`/`[permission]`/section 内未知键):改 theme/auto_compact/channel → 保存 → 文件 diff **只含被改 key,未知表与 section 内未知键原样保留**
- [x] 并发场景:保存前另开进程改文件(或 apply profile)→ 后端重试收敛或返回 conflict → 前端冲突条 + 重载可用,无数据丢失
- [x] 托管锁:激活 profile 时模型 tab 锁定 + 提示条;off 后恢复可编辑;绕过前端直发托管键(手工构造)被后端拒绝且前端呈现该错误
- [x] source tab:双开窗口模拟并发 → conflict 横幅与重载;语法错误 → 行/列 errorMarker;保存成功后 `~/.ccr/backups/grok/` 无 config 备份新增;备份相关文案正确
- [x] layers 面板:managed/requirements 文件存在时(临时构造)显示与策略提示正确
- [ ] config.toml 不存在 → 默认态 → 保存创建成功
- [x] `just frontend-check-quick` 全绿;`raw-config-editor-contracts.md` 契约自查通过
- [ ] **生产构建验证**:`just tauri-build`(或产物)下 source tab 编辑器渲染正常(CodeMirror CSP nonce,契约已知 WebView2 风险)
- [x] 桌面 + 窄视口 × 明暗主题截图走查通过

> 2026-08-01 验证边界:并发/托管/raw/layers 条目由 Tauri + frontend smoke 与 Web 预览覆盖;真实临时 `GROK_HOME` 写入、缺失文件首次创建、production WebView2 运行时 CSP 仍待手工验收。`just tauri-build` 与 CSP nonce smoke 已通过。

## Out of scope

- `[mcp_servers]`/`[permission]`/`[sandbox]`/`[tools]` 等节的表单化(source tab 覆盖)
- project/managed/requirements 层的编辑(只读列出)
