# 父任务执行计划(集成编排,非实现清单)

父任务不直接承载实现;本文件固化子任务串行顺序、共享文件所有权、集成评审与回滚点。实现细节见各子任务 implement.md。

## 串行顺序(硬依赖,不并行共享文件)

1. `08-01-grok-tauri-commands` — 后端契约冻结(DTO/status 信封/activation inspection/生成物)。**契约冻结前任何前端子任务不得 start。**
2. `08-01-grok-ui-home` — 唯一动公共配置的任务(router/mainLayoutShell/moduleSubnav/tailwind/i18n 骨架/`domains/grok.ts` 骨架/占位视图)。
3. `08-01-grok-ui-profiles` 与 `08-01-grok-ui-settings` — **串行执行,profiles 先行**(两者都追加 `domains/grok.ts`、i18n `grok:` 命名空间与 router import;并行会在共享文件上冲突)。

## 共享文件所有权

| 文件                                    | 建立者                             | 追加者                                                   | 约束                                           |
| --------------------------------------- | ---------------------------------- | -------------------------------------------------------- | ---------------------------------------------- |
| `src/api/domains/grok.ts`               | ui-home(骨架+分区注释)             | profiles、settings 各自追加自己分区                      | 不改他人分区                                   |
| i18n `grok: {}`                         | ui-home(overview/dashboard/states) | profiles(`grok.profiles.*`)、settings(`grok.settings.*`) | key 不跨区复用时先提到 `common.*`              |
| `router/index.ts` grok 三条             | ui-home(含占位 import)             | profiles/settings 各自替换**自己那条** import            | 路由结构不再变                                 |
| `views/grok/GrokPlaceholderView.vue`    | ui-home 创建                       | —                                                        | **删除动作归父任务集成评审**(确认零引用后删除) |
| generated(inventory/bindings/TS client) | tauri-commands 再生                | 后续子任务只消费                                         | 手改禁止                                       |

## 集成评审清单(全部子任务归档前,父任务执行)

- [x] 删除 `GrokPlaceholderView.vue`,`GrokPlaceholderView` / `grok.placeholder` 无残留引用
- [ ] 跑父 prd「集成验收」全部条目(含改名激活 profile、drift 场景、非 local 环境、五个 just 门禁)
- [ ] 对照父 prd「凭据边界」逐条终审(重点:DevTools 网络面板全量 grok 响应无明文;source tab 内容未进入 store/日志)
- [x] 3.3 spec 更新核验:`grok-profile-runtime.md`(inspection API)已更新;前端已补 `environment-scoped-dashboard-contracts.md`、`profiles-page-contracts.md` 与 `grok-settings-contracts.md`
- [ ] 桌面 + 窄视口截图走查三个页面(明暗主题)

集成记录(2026-08-01):六项父 PRD 门禁与最终 `just ci` 全绿;Tauri inventory、bindings、路由/导航、i18n、结构化脱敏、Local-only、CAS/rename/delete 契约均由静态检查与自动化覆盖。真实 DevTools 全响应检查与临时 `GROK_HOME` 全链路未执行,故上面两项集成/凭据终审保持未勾选;Home/Profiles 四象限与 production WebView2 CodeMirror 运行时也保持 UNVERIFIED。

## 回滚点

- 每个子任务独立成串 commit,单独 revert 不影响其他子任务(home 回滚会连带 profiles/settings 路由失效,故回滚顺序与实施顺序相反)。
- 核心层 inspection API(tauri-commands 内)单独 commit,可独立 revert(UI 未合入时无消费方)。
