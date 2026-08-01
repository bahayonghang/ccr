# Grok Profiles 管理页面

> 父任务:`08-01-grok-ui-platform`。
> 依赖:`08-01-grok-tauri-commands`(契约冻结)、`08-01-grok-ui-home`(路由占位/API domain 骨架/i18n 命名空间/平台色)。**与 grok-ui-settings 串行,本任务先行**(共享 `domains/grok.ts`/i18n/router,见父 implement.md)。
> 修订记录:2026-08-01 依据 Codex 审阅修订(删 raw 编辑器;DTO patch/credential_action;status 信封;rename 部分失败 UX;Local-only)。

## Goal

新建 `GrokProfilesView.vue`,复用共享 profiles 组件库(`components/profiles/*` + `profiles-page.css`),提供 Grok profile 的列表、切换、创建、编辑(patch 语义)、启停、删除(状态信封驱动)、off,交互与 Codex Profiles 页对齐。

## Requirements

### 列表与状态

1. 骨架复用共享组件:Header / StatStrip / QuickRail(钉选+最近)/ Toolbar(搜索/状态/标签/排序/卡片-列表切换)/ 启用与停用分组 / Inspector / CommandPalette(⌘K)/ 快捷键(`/`、⌘1-9、Esc)。**不含 ProfilesRawEditorPanel**(评审裁剪,Header 不渲染源码编辑按钮)。
2. 行/卡片展示 Grok 特有字段:auth_mode 徽章三态、`base_url_display`、model、api_backend、reasoning_effort、context_window;当前激活 profile 高亮;**activation 为 drifted/unsafe 时页头显示警示条**(文案引导 off/手工恢复)。
3. StatStrip 平台特色槽:当前 auth_mode;health 槽:profiles 总数/启用数。
4. **Local-only**:非 local 环境渲染提示横幅并禁用全部操作(父 design D9)。

### 操作

5. 切换(apply):确认框含当前→目标 diff(model / base_url_display / auth_mode / reasoning_effort);成功后整体重载 + toast。
6. Off:独立操作(有激活或 drifted 时可用),确认文案说明「恢复切换前的 Grok 原生配置」。
7. 创建/编辑弹窗(`GrokProfileEditorModal`):
   - 类型选择:官方 / 第三方,**以 DTO 的 `profile_kind` 为准回填,前端不做推断**。
   - 第三方:base_url(**只写不读**:输入框空白,placeholder 显示 `base_url_display` +「留空保持不变」;填写即整体替换)、model 必填、凭据区 `credential_action` 四态(preserve 默认/replace_api_key/replace_env_key/clear)、api_backend 下拉(chat_completions|responses|messages)、context_window 数字、supports_backend_search 开关、reasoning_effort 下拉 7 级含"未设置"。
   - 官方:仅 model(可空)/reasoning_effort/描述/标签,凭据与 base_url 区隐藏。
   - 提交为 **patch 请求**:未触碰的字段不出现在请求里;显式清除动作发 `null`(或 credential_action=clear)。
   - 后端校验错误全文呈现在弹窗错误条,不吞错误。
8. 删除:按后端 `{status}` 信封分支——`deleted` → 重载+toast;`blocked(active|drifted)` → danger 二次确认「off 并删除」→ `force:true` 重发;`blocked(unsafe_missing_entry_state)` → 仅提示手工恢复指引,**不提供 force**。禁止错误文案字符串匹配。
9. 改名:按信封分支——`renamed` 正常;`rename_apply_failed` / `rename_cleanup_failed` 显示 message 与明确的后续操作按钮(重试切换 / 重试删除旧名),列表重载呈现真实中间态。

## Acceptance Criteria

- [ ] 全流程手工冒烟(临时 `GROK_HOME`):创建第三方(api_key)→ 切换(diff 正确)→ 编辑仅改 reasoning_effort(**验证请求不含未触碰字段,base_url/凭据保持**)→ **改名激活中的 profile**(渐进验证 renamed 结局)→ 创建官方 → 切换 → off → 删除;每步 UI 状态、toast、重载正确
- [ ] 页面任何网络响应/DOM 中无 api_key 明文;编辑已配置凭据的 profile 时输入框空白、状态徽章正确
- [ ] env_key 型 profile 徽章与编辑回显(变量名可见,值不存在)正确
- [ ] drift 场景(apply 后手改 config.toml):警示条出现,删除走 blocked(drifted) 引导,off 后恢复正常
- [ ] ⌘K、钉选/最近、搜索过滤、启停分组与 Codex 页行为一致;非 local 环境横幅生效
- [ ] `just frontend-check-quick` 全绿;`profiles-page-contracts.md` 契约自查通过(raw 相关条款标注 N/A)
- [ ] 桌面 + 窄视口 × 明暗主题截图走查通过

## Out of scope

- profiles.toml 源码编辑(评审裁剪,后续单独立项)
- 共享 profiles 组件的功能增强(如需改共享组件,回规划层评审)
- profile 模板/目录(providers catalog)接入
