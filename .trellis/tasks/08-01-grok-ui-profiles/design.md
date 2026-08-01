# 技术设计:Grok Profiles 管理页面

前置阅读:父任务 design.md(D2/D3/D8/D9)、`research/frontend-platform-patterns.md` §3、spec `profiles-page-contracts.md`。骨架基准:`CodexProfilesView.vue`。
修订记录:2026-08-01 依据 Codex 审阅重写 §2/§3(patch 表单模型、信封分支),删除 raw 编辑器。

## 1. 新增文件

| 文件 | 内容 |
|---|---|
| `src/views/GrokProfilesView.vue` | 复制 CodexProfilesView 骨架改造;`<ModuleSubnav module="grok" />`;`--cp-icon-color` 用 `platform.grok` 色;**去掉 raw 编辑器面板与入口按钮**;新增 drift 警示条与 Local-only 横幅 |
| `src/components/grok/GrokProfileEditorModal.vue` | 编辑弹窗(patch 表单模型,见 §2) |
| `src/components/grok/GrokProfileCard.vue` | 卡片视图(仿 codex ProfileCard) |
| `src/utils/grokProfiles.ts` | `createGrokRowDescriptor(t)` / `createGrokInspectorDescriptor(t)`(insights:第三方缺 base_url、未设 reasoning_effort、drifted 警示)/ `createGrokDiffFields(t)` |
| `src/utils/grokProfileEditor.ts` | `createEmptyGrokForm()` / `fillGrokForm(dto)` / `buildGrokPatch(form, dirtyFields)`(**只序列化被触碰字段**) |
| `src/composables/useGrokProfilesFilter.ts` | 仿 useCodexProfilesFilter |

`src/api/domains/grok.ts` 追加 profiles 分区:`listGrokProfiles / getGrokProfile / addGrokProfile / updateGrokProfile(name, patch) / deleteGrokProfile(name, {force}) / applyGrokProfile / grokProfileOff`(**无 raw 两函数**)。i18n 追加 `grok.profiles.{toolbar,statStrip,inspector,commandPalette,groups,fields,messages,authModes,editor,driftBanner,renameRecovery}`。

## 2. 编辑弹窗:patch 表单模型

```
form.profileKind:来自 DTO.profile_kind(编辑态只读展示;创建态可选)——前端零推断
dirtyFields: Set<string>            // 表单控件 @change 记入;buildGrokPatch 只输出 dirty 项
base_url:输入框永远空白起步;placeholder = base_url_display + t('留空保持不变')
  - 未触碰 → patch 缺席(后端保留原值,含 query/userinfo)
  - 填写   → patch 带新值(整体替换)
  - 显式「清除 base_url」动作(仅当切到官方型时隐式发生,由后端 validate 把关)
凭据区 credentialAction: 'preserve'(默认,徽章显示当前形态)| 'replace_api_key' | 'replace_env_key' | 'clear'
reasoning_effort:下拉 8 项(未设置 + 7 级);「未设置」在编辑态且原值存在时 → patch 发 null(清除)
提交:创建 = 全量请求;编辑 = buildGrokPatch(form, dirtyFields) + credential_action
```

- 前端只做轻校验(必填非空、context_window 正整数、0-100 类范围),语义校验交后端 `validate_profile`,错误全文入弹窗错误条。
- **`base_url_display` 只进 placeholder/展示,任何路径不写回**(丢 query/userinfo 的根源,评审 P0)。

## 3. 操作编排(View 层,信封分支)

- apply/off/delete/rename 均走 `useConfirmAction`;apply 用 `buildProfileDiff(current, target, createGrokDiffFields(t))`。
- **delete**:`deleteGrokProfile(name)` → 按 `status` 分支:
  - `deleted` → `markWrite()` + 重载 + toast;
  - `blocked` + `reason: active|drifted` → danger 确认「off 并删除」→ `deleteGrokProfile(name, {force:true})`;
  - `blocked` + `reason: unsafe_missing_entry_state` → 信息弹窗(手工恢复指引,含 config.toml 路径),无 force 按钮。
  - 命令 `Err`(意外错误)→ `getErrorMessage` toast。**全程无错误文案匹配。**
- **rename**(编辑弹窗改名提交后):`renamed` → 正常收尾;`rename_apply_failed` → 警示弹窗 + 「重试切换到新名」按钮(调 applyGrokProfile(new));`rename_cleanup_failed` → 警示弹窗 + 「重试删除旧名」按钮(调 deleteGrokProfile(old));两种情况都强制整体重载呈现真实中间态。
- **off**:activation ∈ {active, drifted} 时可用(来自 list 响应的 activation 字段);drifted 场景文案改为「恢复入口配置并清除漂移状态」。
- `pendingAction` / `rowsDisabled` / `loadProfiles({preserveData:true})` / uiStore toast 全套沿用;QuickSwitch `useProfilesQuickSwitch({platform:'grok'})`。
- Local-only:`getCurrentEnvironment()` 非 local → 横幅 + `rowsDisabled` 常开。

## 4. 凭据呈现红线(复查清单)

- 前端类型(生成物)本身无 api_key/auth_token 字段;Inspector/diff/卡片只用 auth_mode + has_inline_credential + env_key + base_url_display。
- 编辑弹窗不存在"取回明文"的调用路径;`getGrokProfile` 与 list 同形。
- DevTools 网络面板抽查纳入验收。

## 5. 回滚

页面/组件/utils/composable 均为新文件;api domain 与 i18n 为追加块;router 仅改自己那条 import(占位文件不删,归父任务)。整任务一个 revert 干净退出。
