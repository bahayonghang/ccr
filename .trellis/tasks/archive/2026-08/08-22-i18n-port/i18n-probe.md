# i18n 前置核对

日期：2026-08-24。词条文件未改。

## 花括号

`zh-CN.ts` 叶子 4,164。插值变量均为 `{identifier}`（`count` / `name` / `error` 等）。

11 处非标识符花括号，全部是字面量写法 `{'{'}` / `{'}'}` / `{'@'}` / `{'{"k":"v"}'}`：

- `checkin.oauthWizard.credentialsPlaceholder`
- `mcp.argsPlaceholder`
- `plugins.form.configPlaceholder`
- `sync.account.usernamePlaceholder`
- `codex.mcp.placeholders.args`
- `codex.profiles.placeholders.account`
- `codex.profiles.placeholders.extraJson`
- `codex.plugins.configPlaceholder`
- `gemini.plugins.configPlaceholder`

处理：加载词条时解开这些字面量，locale 文件 git diff 为空。i18next `interpolation.prefix='{'`、`suffix='}'`。

## formatMessage.ts

自定义兜底插值，不是 `$t` 薄包装。保留；运行时 `t` 改为 i18next。

## 语言偏好

键：`localStorage['ccr-ui-locale']`。读写在 `src/i18n/index.ts` 的 `readStoredLocale` / `setLocale`。未引入 `i18next-browser-languagedetector`。

## 动态 key

存在。形态：`t(item.labelKey)`、`t(\`${prefix}.suffix\`)`、`t(\`mcp.manager.scopes.${scope}\`)`。

静态检查：字面量 `t()` / `tf()` 做缺失 key；模板前缀/后缀与引号中的点分串做未使用 key。白名单见 `ccr-ui/scripts/i18n-key-whitelist.json`。
