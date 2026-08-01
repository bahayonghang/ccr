# 执行计划:Grok 可视化 Settings 页面

修订记录:2026-08-01 依据 Codex 审阅修订(set/unset patch、conflict UX、layers、生产 CSP 验证、串行与占位所有权)。

## 前置

- [ ] 依赖确认:grok-tauri-commands **契约冻结**(settings patch/raw/layers 命令 + custom_models 字段)、grok-ui-home 已交付;**grok-ui-profiles 已完成**(串行,共享文件)
- [ ] 读本任务 design.md、父 design D4/D5/D8/D9、spec `raw-config-editor-contracts.md`;打开 `CodexSettingsView.vue` 作基准
- [ ] 核对 `ConfigSourcePanel` 备份提示文案实现,确定 grok 覆盖方式(文案 key / 最小 prop)

## 步骤(按序)

1. [ ] `src/api/domains/grok.ts` 追加 settings 分区五函数;i18n 追加 `grok.settings.*`(zh/en 同步,`cd ccr-ui && node scripts/check-i18n.mjs`)
2. [ ] `GrokSettingsView.vue` 骨架:tabs + baseline/form/dirtyKeys + `buildPatch`(set/unset)+ 保存三分支(saved/conflict/托管拒绝)
3. [ ] Tab「模型」:default/default_reasoning_effort + 托管锁提示条 + custom_models 只读摘要卡
4. [ ] Tab「会话与界面」与「CLI」:全部控件 + 范围/枚举校验 + 未知现值保护
5. [ ] Source tab:ConfigSourcePanel 接入 + grok 备份文案覆盖 + layers 四层面板 + 策略覆盖提示 + 离开确认
6. [ ] 替换 grok-ui-home 留下的 settings 路由 import(**不删占位文件**,归父任务集成评审);页脚注释丢失说明

## 验证

- [ ] `just frontend-check-quick` 全绿;`just tauri-bindings-check`
- [ ] 手工冒烟(对照 prd 验收清单):patch 请求抓包只含 dirty key;未知表/键保留 diff;并发冲突(另进程写文件/apply)→ conflict 条 + 重载;托管锁定/解锁;source tab conflict/invalid/无备份;layers(临时构造 managed 文件);config 不存在创建流
- [ ] 生产构建 `just tauri-build`(或既有产物流程)验证 source tab 编辑器渲染(CSP nonce)
- [ ] 桌面 + 窄视口 × 明暗主题走查表单控件

## 评审门

- 步骤 3 后:托管提示条与 custom_models 摘要交互确认
- 完成后:交 `trellis-check`(重点 cross-layer:dirtyKeys → patch → CAS merge → 磁盘 diff 全链路)

## 回滚点

步骤 1(数据层)/ 2-5(视图)/ 6(路由替换)分 commit。
