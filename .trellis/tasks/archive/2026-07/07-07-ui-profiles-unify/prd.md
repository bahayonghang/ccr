# Claude/Codex Profiles 交互与视觉统一

## Goal

把 ClaudeCodeProfilesView 与 CodexProfilesView 收敛到**同一套交互模型与视觉语言**:统一确认对话框、快捷键语义、命令面板/快速切换能力、accent 色轴与卡片栅格;移除假数据;优化编辑 modal 的引导层级。两页共享的 profiles/* 组件族是落点,平台差异只允许存在于 descriptor 策略层(现有模式,保持)。

## 现状问题清单(代码证据)

| #   | 问题                                                                               | 位置                                                                                 |
| --- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| P1  | Claude 页 apply/delete/rename 用原生 `confirm()/alert()`,Codex 页用 ConfirmModal   | ClaudeCodeProfilesView.vue:1112/1145/1157/1165/1177 vs CodexProfilesView.vue:286-294 |
| P2  | ⌘K 语义冲突:Codex=命令面板,Claude=聚焦搜索                                         | CodexProfilesView.vue:895 vs ClaudeCodeProfilesView.vue:987                          |
| P3  | Codex 有 CommandPalette + ProfilesQuickRail + ⌘1-9,Claude 全无                     | CodexProfilesView.vue:62-68/247-256/900-908                                          |
| P4  | Codex StatStrip 硬编码假 sparkline `[3,5,4,6,7,8,7]` / `[2,4,3,5,4,6,5]`           | CodexProfilesView.vue:58-59                                                          |
| P5  | accent 轴不一致:Claude 用 accent-secondary(沙),Codex 用 accent-primary(陶土)       | 两页 scoped `--cp-accent` 定义                                                       |
| P6  | 卡片视图强制单列,宽屏空间浪费                                                      | 两页 `.cp-grid` 单列                                                                 |
| P7  | lastWrite 显示的是列表加载时间,非真实写入时间                                      | 两页 loadProfiles 内 `new Date().toLocaleTimeString()`                               |
| P8  | Claude 编辑 modal 分区滚动同步 @scroll 未节流;20+ 字段无渐进披露                   | ClaudeCodeProfilesView.vue:351/998-1014                                              |
| P9  | Codex 卡片把只读 base_url/model/认证渲染成输入框样式,假可编辑 affordance           | 截图3 ProfileCard 卡片区                                                             |
| P10 | 页头命名不一致:"Claude Profiles 管理" vs "配置管理"                                | 两页 ProfilesHeader labels                                                           |
| P11 | 每行重复大号"应用此 Profile"按钮(19 行阵列);当前 profile 未在列表置顶,只在右栏可见 | 截图2                                                                                |
| P12 | 全宽卡片仅 3 个稀疏字段横向铺开,但 base_url 仍被截断(https://api.78c...)           | 截图2 卡片字段区                                                                     |
| P13 | 右栏 AUTH 分布渲染 3 条 0 值条目(OpenAI Chat 0 / API Key 0 / Provider 环境 0)      | 截图3 分布洞察                                                                       |

## Requirements

- R1 确认交互统一:两页全部走 ConfirmModal(或升级后的 GlobalConfirmDialog),类型语义一致(apply=warning、delete=danger、rename=warning);错误提示统一走 uiStore toast,禁用原生 alert。
- R2 快捷键统一:两页 `⌘K`=命令面板、`/`=聚焦搜索、`⌘1-9`=切换启用 profile、`Esc`=关闭浮层;CommandPalette 与 QuickRail 从 codex/ 提升为平台无关组件(descriptor 注入平台行为),Claude 页接入。
- R3 数据真实性:P4 假 sparkline 移除——ProfilesStatStrip 的 spark 参数改为可选,无真实数据就不渲染趋势线(禁止占位假曲线);P7 lastWrite 改为仅在真实写操作(save/apply/delete)成功后更新,读操作不更新。
- R4 视觉统一:两页 `--cp-accent` 统一取 `--color-accent-primary`(跟随用户 data-accent 选择),平台识别用 `--color-platform-claude/codex` 仅出现在图标/徽章;卡片栅格 ≥1280px 双列、≥1680px 可三列(卡片内容密度同步收紧);列表视图两页列结构一致。
- R5 编辑 modal 体验:分区滚动同步节流(rAF 或 IntersectionObserver);"基础"分区始终展开,高级分区(模型映射/超时等)默认折叠(渐进披露);Provider 模板选择器保持首位作为引导入口。
- R6 材质落地:modal 用 floating 档玻璃;页面内卡片/统计条用不透明表面;QuickRail 若做 sticky 悬浮可用 inline 档(计入玻璃预算)。
- R7 所有改动过 `useClaudeProfilesFilter`/`useCodexProfilesFilter`/descriptor 的既有单测与 smoke(如有),i18n 中英齐全。
- R8 卡片信息设计(截图复核):只读数据一律键值文本行,禁用 input 样式假 affordance(P9);当前 profile 固定置顶并强化"当前"标识(P11);"应用"动作仅对非当前 profile 呈现,行内按钮降为紧凑尺寸(hover/focus 强化),消除 N 行大按钮阵列;base_url 中段省略 + title/tooltip 全文(P12);分布洞察隐藏 0 值条目(P13);两页页头命名统一为"<平台> Profiles 管理"(P10)。

## Out of Scope

- 后端 IPC 与 profile 数据结构;provider-template-contracts 的模板数据流(仅消费)。
- OpenCode/Gemini/Droid 的 profile 类页面(清扫任务顺带对齐)。

## Acceptance Criteria

- [ ] `rg "\\b(confirm|alert)\\(" ccr-ui/src/views/ClaudeCodeProfilesView.vue` 零命中。
- [ ] 两页 ⌘K/⌘1-9/斜杠/Esc 行为一致(手测记录);Claude 页可用命令面板完成 apply/add/export/reload。
- [ ] ProfilesStatStrip 无假 spark 入参;lastWrite 仅写操作后变化。
- [ ] 1920px 宽下卡片视图 ≥2 列;两页 accent 一致且跟随 data-accent 切换。
- [ ] Codex 卡片无 input 样式只读字段;当前 profile 列表首位且有当前标识;非当前行按钮为紧凑尺寸。
- [ ] 右栏分布无 0 值条目;两页页头命名一致;base_url 悬浮可见全文。
- [ ] 编辑 modal:高级分区默认折叠;滚动分区高亮不再逐帧计算(rAF 节流或 IO)。
- [ ] `bun run type-check && bun run lint` + `bun run test:smoke -- tests/provider-templates.smoke.test.ts` 通过。
- [ ] 亮/暗截图:两页并排对比,视觉语言一致。

## Dependencies

- 依赖 07-07-ui-glass-tokens(floating/inline 材质令牌)。
