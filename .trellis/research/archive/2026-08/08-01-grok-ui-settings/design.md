# 技术设计:Grok 可视化 Settings 页面

前置阅读:父 design D4/D5/D8/D9、`research/frontend-platform-patterns.md` §4、spec `raw-config-editor-contracts.md`。骨架基准:`CodexSettingsView.vue`(双模式范本)。
修订记录:2026-08-01 依据 Codex 审阅重写 §2(dirtyKeys → set/unset patch,废弃 section payload)。

## 1. 新增/修改文件

| 文件 | 内容 |
|---|---|
| `src/views/GrokSettingsView.vue`(新建) | tabs:`model` / `sessionUi` / `cli` / `source`;`<ModuleSubnav module="grok" />`;Local-only 横幅 |
| `src/api/domains/grok.ts`(追加 settings 分区) | `getGrokSettings / updateGrokSettings(patch) / getGrokConfigRaw / saveGrokConfigRaw / listGrokConfigLayers` |
| i18n zh/en(追加) | `grok.settings.{tabs,model,sessionUi,cli,managedBanner,customModels,layers,conflict,messages,sourceNotes}` |
| router | 仅替换 settings 一条 import(占位文件不删,归父任务) |

## 2. 表单状态与 patch 组装

```ts
const baseline = ref<GrokSettingsResponse>()   // getGrokSettings 快照(含 custom_models/activation/managed_keys_locked)
const form = reactive<GrokSettingsForm>({...}) // 白名单字段的编辑值;'__unset__' 语义由下拉「未设置」项承载
const dirtyKeys = new Set<DottedKey>()          // 控件 @change 记入
function buildPatch(): { set: Record<string, unknown>, unset: string[] } {
  // 只遍历 dirtyKeys:值为「未设置」→ unset;否则 → set
}
```

- `handleSave`:`updateGrokSettings(buildPatch())` → 按响应分支:`saved` → toast + `reload()`(重拉 baseline,清 dirtyKeys);`conflict` → 冲突提示条(常驻,含「重载最新值」按钮,重载后 dirtyKeys 清空需用户重新改动——不做自动合并);托管键拒绝 → 错误条 + Profiles 页链接。
- 托管锁定:`managed_keys_locked` → 模型 tab 两个控件 disabled + 提示条;dirtyKeys 逻辑天然不会产出被禁用控件的 key(双保险:后端拒绝)。
- `custom_models` 只读摘要卡列(id/name/model/base_url_display)+ 「编辑请用源码模式」切 tab 链接。
- 数值/枚举校验:auto_compact 0-100、channel/hints 枚举下拉;文件中的未知现值(如未收录 theme)渲染为「当前值」选项防隐性改写。

## 3. Source tab

```vue
<ConfigSourcePanel v-if="activeTab==='source'" language="toml"
  :get-raw="getGrokConfigRaw" :save-raw="saveGrokConfigRaw" :list-layers="listGrokConfigLayers"
  @saved="reloadSettings" @close="activeTab='model'" @dirty-change="sourceDirty = $event" />
```

- 备份文案:核对 ConfigSourcePanel 现有 i18n key,若含"已备份"承诺则为 grok 提供覆盖(最小 prop 或按平台文案 key,默认行为不变,codex/claude 零影响);grok 文案 = 「此文件不做自动备份」。
- layers 面板渲染 user/project/managed/requirements 四层(exists + editable 标记);managed/requirements 任一存在 → 策略覆盖提示。
- `changeTab` 离开 source 丢弃确认与 `onBeforeRouteLeave` 守卫照搬 Codex 页;挂载明文警告由 Panel 内建。

## 4. 已知取舍(UI 明示)

- 注释丢失限于 typed 保存(后端 toml::Value 全文档重序列化);页脚常驻说明;source tab 原文写入无此问题。
- typed 保存的 conflict 不做三方合并 UI(重载后重改),与 raw tab 的 conflict 行为语义对齐。
- project/managed/requirements 层只读(MVP)。

## 5. 回滚

新文件 + api/i18n 追加块 + 一行路由 import;单 revert 退出,不影响 profiles/home。
