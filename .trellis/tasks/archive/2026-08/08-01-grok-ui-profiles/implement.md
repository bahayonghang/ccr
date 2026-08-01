# 执行计划:Grok Profiles 管理页面

修订记录:2026-08-01 依据 Codex 审阅修订(删 raw 步骤;patch/信封;drift 场景;截图矩阵;占位所有权)。

## 前置

- [ ] 依赖确认:grok-tauri-commands **契约冻结**、grok-ui-home 已交付(路由占位/`grokApi` 骨架/i18n `grok:` 在);**settings 子任务尚未开始或已协调串行**(共享文件冲突,见父 implement.md)
- [ ] 读本任务 design.md、父 design D3/D8、spec `profiles-page-contracts.md`;打开 `CodexProfilesView.vue`/`CodexProfileEditorModal.vue`/`utils/codexProfiles.ts` 作基准

## 步骤(按序)

1. [ ] `src/api/domains/grok.ts` 追加 profiles 分区七函数(无 raw)+ 运行时校验;i18n 追加 `grok.profiles.*`(zh/en 同步,`cd ccr-ui && node scripts/check-i18n.mjs`)
2. [ ] `utils/grokProfiles.ts`(descriptor 三件套,含 drift insights)+ `utils/grokProfileEditor.ts`(**dirtyFields 驱动的 buildGrokPatch**)+ `useGrokProfilesFilter.ts`
3. [ ] `GrokProfileCard.vue` + `GrokProfileEditorModal.vue`(profile_kind 回填 + credentialAction 四态 + base_url 只写不读)
4. [ ] `GrokProfilesView.vue`:复制 Codex 骨架 → 移除 raw 面板/按钮 → 替换 API/descriptor/弹窗/文案 → 接 off / delete 信封分支 / rename 恢复弹窗 / drift 警示条 / Local-only 横幅
5. [ ] 替换 grok-ui-home 留下的 profiles 路由 import(**不删占位文件**,归父任务集成评审)

## 验证

- [ ] `just frontend-check-quick` 全绿;`just tauri-bindings-check`
- [ ] 手工冒烟(临时 `GROK_HOME`,对照 prd 验收清单):patch 请求抓包验证未触碰字段缺席;激活改名三结局(正常 + 模拟 apply 失败/删除失败的恢复按钮);delete 三分支;drift 场景;env_key 回显
- [ ] DevTools 网络面板:所有 grok 响应无 `api_key`/`auth_token` 字段;base_url 仅 display 形态
- [ ] ⌘K/钉选/搜索与 Codex 行为对齐抽查;非 local 横幅
- [ ] 桌面 + 窄视口 × 明暗主题截图走查

## 评审门

- 步骤 3 后:编辑弹窗(profile_kind 回填、credentialAction 四态、base_url placeholder)截图确认
- 完成后:交 `trellis-check`;UI 走查用 `frontend-quality-reviewer`

## 回滚点

步骤 1-2(数据层)/ 3-4(视图层)/ 5(路由替换)分 commit;回退 5 即恢复占位。
