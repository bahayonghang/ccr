# CCR UI 视觉世界替换：首页/设置/配色体系重构

## Goal

用户判定当前首页设计、整体配色与设置页存在系统性问题，决定**替换视觉世界**（非现有 clay 编辑体系内精修）。在保持产品真相（冷静、精确、可信赖的运维工作台；高密度；保护本地配置安全）的前提下，用一套新视觉系统重构 Overview 首页、全局设置页、左下角设置坞，并重写配色 token 体系；同时修复图表比例失控、信息可信度、中英文切换失效三类实质缺陷。

## 用户已确认的决策

1. **视觉方向：替换视觉世界** —— 旧外观作为反参照，需重写 `ccr-ui/DESIGN.md`。
2. **深色配色：中性深色转暖** —— 中性深色族（现冷灰 `#131316`）向暖棕微调，与暖强调色同源；消除两个底色族在深色下的割裂。
3. **语言策略：全局中文化** —— zh-CN 模式下全部约 29 处英文 eyebrow（`Shell Preferences`、`Operations Monitor` 等）及零散英文标签翻译为中文；修复空依赖 memo 导致的切换失效。
4. **新视觉方向：行情终端（Operator Terminal）** —— impeccable 方向轮（concept-seed `19fe1fa0`，mode=operate）骰子指派「运转图台」，**用户最终点选「行情终端」（kind=pick，用户决定优先于骰子）**。暖黑磷光终端：底 `#100f0c`、面板 `#1e1c16`、数据白 `#e6e1d5`、琥珀 `#f0a32b` 仅命令/激活/焦点、绿 `#5fa05a` 红 `#c0503c` 仅状态；等宽表格数字、发丝分隔线、功能键命令条；平台线色仅作身份 tick。方向契约：`ccr-ui/.impeccable/surfaces/ui-src-features-usage-dashboard-dashboardview-tsx.md`；决策全程：`research/direction-round.md`。三条落选挑战者纪律作为全场约束：颜色只住状态发线、禁用态一眼可辨、图表维度诚实。

## Scope（Task Map）

| 子任务 | 交付物 | 依赖 |
|---|---|---|
| `09-03-theme-token-world` | 新配色/token 体系落地 `tokens.css`，深色暖化，平台线色确权（含 Antigravity），启动 Loader 修色，DESIGN.md 重写 | 无（地基，先行） |
| `09-03-overview-home-restructure` | 首页重构：图表固定尺度、信息层级、Sessions 未索引诚实态、死键清理 | 依赖 theme-token-world 的 token |
| `09-03-settings-i18n-restructure` | 设置页布局/排版重构 + 全局中文化 + stale memo 修复 + 回归测试 | 依赖 theme-token-world 的 token |
| `09-03-settings-dock-polish` | 设置坞排版与样式（窄侧栏换行、层级、点击态） | 依赖 theme-token-world 的 token |

子任务间依赖已写入各 child PRD；每个子任务可独立验证、独立归档。

## 跨子任务验收标准

- [x] 新视觉世界在首页、设置页、设置坞三处一致呈现，浅色/深色 × 中性/暖陶四组合均成立（finish reviewer 像素级复核 combo/settings 截图矩阵通过）
- [x] 首页图表有确定高度，任何窗口尺寸下不再被右栏撑成巨柱；峰值柱不超出图表区（`clamp(10rem,26vh,16rem)`，四组合截图验证）
- [x] 首页 Sessions 未索引时显示诚实状态而非静默 0；Antigravity 使用自己的平台色（新增 `sessionsUnindexed/sessionsIndexing/sessionsUnindexedHint` 三键，消费既有 `needs_session_index`）
- [x] zh-CN 模式下设置页无任何英文残留；全局英文 eyebrow 中文化（实际 52+ 处 zh 值）；切换语言后所有文案实时更新（stale memo 修复 + locale-switch 回归测试）
- [x] 设置坞在 200px 最窄侧栏下排版整齐，无凌乱换行（双行布局 + 版本号入 meta 行，en 标题不再裁切）
- [x] 中性深色族完成暖化，与 clay 深色并列时气质一致（reviewer 实测两族可分且协调）
- [x] `ccr-ui/DESIGN.md` 按建成后的新世界重写（impeccable：规则书写于建成之后；design.json 边车与 AGENTS.md 指向同步，PRODUCT.md 旧世界语言清除）
- [x] 新增 i18n 回归测试：zh 值中文断言（`tests/i18n/zh-cn-cjk.smoke.test.ts`）+ 设置页 live 切换（`tests/configs/app-settings-locale-switch.smoke.test.tsx`）
- [x] `just ui-check`（type-check、lint、test、build、tauri:check）各阶段全绿；唯一例外为 2 个 pre-existing `/agent-sessions` 路由失败（clean HEAD 可复现，与本任务无关，用户已确认作为残留保留）

## 约束

- 遵守 `ccr-ui/PRODUCT.md` 品牌真相：冷静、精确、可信赖；高密度优先；不得滑向通用 SaaS 后台、紫蓝渐变、装饰性玻璃、guofeng/neko/anime。
- 遵守 `ccr-ui/AGENTS.md` 的验证路径：`just check` / `bun run type-check|test|build`，Tauri 侧 `bun run tauri:check|tauri:test`。
- 配色只经 `tokens.css` 单一权威变更；不改 `theme.css` 别名桥以外的遗留面。
- 三个固定不变量：产品功能与内容不变；Tauri IPC 契约不变（除 Sessions 诚实态需要的读取补充）；`data-theme/data-flavor/data-accent` 启动序列保持字节级 key 兼容（`ccr-theme`/`ccr-flavor`/`ccr-accent`）。

## Notes

- 深度分析：`research/overview-page-analysis.md`、`research/settings-i18n-analysis.md`、`research/theme-shell-analysis.md`
- 视觉方向决策过程（impeccable new-work）：候选推导、concept-seed 指派、挑战者裁决，见本目录 `research/direction-round.md`（方向锁定后写入）
- 相关 spec：`.trellis/spec/ccr-ui/frontend/{index,theme-token-contracts,usage-chart-stability-contracts,dashboard-presentation-contracts,react-rerender-discipline}.md`
