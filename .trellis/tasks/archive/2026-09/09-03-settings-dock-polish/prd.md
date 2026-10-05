# 设置坞（左下角）排版与样式优化

## Goal

优化侧栏底部设置坞（`设置 / 跟随系统 · 深色模式 · 中性 · 中文 / CCR UI v7.3.0`）的排版与样式，使其在新视觉世界下层级清晰、窄宽皆稳。

## Requirements

1. **排版重构**：当前 meta 行是 0.6875rem 点分隔长串（`shell/shell.css:240-246`），窄侧栏下换行凌乱。重排信息层级：标题、状态摘要、版本号分行/分组，或对 meta 做结构化缩写；200px 最窄侧栏（`shellPreferences.ts:55-57` 下限）下依然整齐。
2. **视觉样式**：按方向契约重做 `.settings-dock` 及 `__icon/__copy/__title/__meta/__version`（`shell/shell.css:199-251`）——surface、边框、hover、active（`/settings` 路由下 `--active`）态与邻近导航项的层级关系。
3. **行为保持**：仍是 `NavLink to="/settings"`（`MainLayoutChrome.tsx:74-97`，`data-testid="settings-dock-link"`），不改交互语义；meta 标签计算（`MainLayout.tsx:77-85`）保持响应式，中文化后显示中文值。
4. 评估 meta 串的信息取舍：theme/flavor/locale/version 四项是否全保留、缩写还是图标化，按新世界的状态表达纪律决定。

## Acceptance Criteria

- [ ] 侧栏 200px 与 480px 两个极端宽度下设置坞排版整齐、无凌乱换行
- [ ] zh-CN 下 meta 全中文；语言/主题/flavor 切换后坞内文案实时更新
- [ ] hover/active/focus 三态符合新方向契约；键盘焦点可见
- [ ] `bun run type-check|lint|test|build` 全绿

## Dependencies / Ordering

- 依赖 `09-03-theme-token-world` 的 token 落地后实施；体量小，建议最后做。

## Notes

- 分析：`../09-03-ui-visual-world-replacement/research/settings-i18n-analysis.md` 第 4 节、`research/theme-shell-analysis.md` 壳层节
- 关键文件：`shell/MainLayoutChrome.tsx:74-97`、`shell/MainLayout.tsx:77-85`、`shell/shell.css:199-251`、`config/appMeta.ts`
