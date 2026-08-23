# `codex-auth-shared.css` 落位清单（AC5）

源文件：`ccr-ui/src/styles/components/codex-auth-shared.css`（全局层，`index.css` 首屏导入）。

判定：规则服务 Codex Auth 多个组件（主视图、Accounts/Providers tab、Add/Rename/Save 弹窗），按 `08-22-design-system` 归 `styles/components/`。本任务不改该文件（写权限不含 `src/styles`），React 组件继续消费 `codex-auth-view__*` 类名。

| 选择器族 | 新位置 |
| --- | --- |
| `.codex-auth-view` / `__main` / `__title-icon*` / `__status-*` / `__segment*` | 保留 `styles/components/codex-auth-shared.css` |
| `.codex-auth-view__session-*` / `__filters-*` / `__guard*` | 同上 |
| `.codex-auth-view__providers-*` / `__provider-*` / `__input-*` / `__textarea` | 同上 |
| `.codex-auth-view__composer-*` / `__oauth-*` / `__warning-panel` / `__inline-*` | 同上 |
| `.codex-auth-view__save-*` / `__meta-pill` | 同上 |
| Auth off 横幅 | 本任务用 token 工具类写在 `CodexAuthView.tsx`（`border-accent-warning` / `bg-bg-elevated`） |

无未归类规则：该 CSS 仍整文件服务 Codex Auth 多组件，不拆到单路由 module.css。
