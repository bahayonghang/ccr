# Fix - 系统提示词编辑器正文不可见

> 日期: 2026-07-17。影响 Claude Code 与 Codex 的系统提示词页面；修复位于共享 `CodeSourceEditor`，因此同样保护其他 Markdown 编辑入口。

## 现象与证据

- Codex 卡片显示 `AGENTS.md` 已存在、3727 bytes，编辑区显示 14 个当前视口逻辑行。
- Claude 卡片显示 `CLAUDE.md` 已存在、2366 bytes，编辑区显示 22 个当前视口逻辑行。
- DOM 中存在完整正文，计算出的正文颜色也正确；故障不是后端返回空内容，也不是前景色透明。
- `just tdev` 实际先执行 `just tbuild`，再启动 `src-tauri/target/release/ccr-desktop.exe`，因此必须在 production WebView 中复现，Vite 浏览器结果不能替代该证据。
- 故障现场中 CodeMirror 生成了约 12 KiB 的运行时 `<style>`，但它没有 Tauri CSP nonce，`style.sheet === null`。基础布局规则被 WebView 拒绝后，`.cm-scroller` 不再是 flex：gutter 宽约 389.6px、高约 2220.6px，正文 top 约 2372.9px，实际内容被排到可视区下方。

## 根因

CodeMirror 通过 `style-mod` 在运行时注入基础布局和主题 CSS。Tauri production WebView 的 CSP 只允许带本次页面 nonce 的内联样式，而 CodeMirror 默认创建的 `<style>` 没有 nonce，整张运行时样式表被拒绝。行号与正文仍存在于 DOM，但缺少 `.cm-scroller { display: flex }` 等基础规则后被纵向排布，正文落到巨大 gutter 之后，所以视口中只看到行号。

修复从页面已有 `style[nonce]` / `script[nonce]` 读取 Tauri nonce，并通过 CodeMirror 官方 `EditorView.cspNonce` facet 传入。这样 CodeMirror 生成的运行时 `<style>` 带相同 nonce，基础布局与主题规则都能通过 CSP。共享主题仍对 `.cm-content` 和 `.cm-line` 显式设置 `color: var(--text-primary)`，作为明暗主题正文颜色契约。

## 被否定的竞态假设

早期草案认为动态导入 CodeMirror 时 `v-model` 被清空，并提出忽略第一次 `docChanged`、挂载后强制重同步及空内容日志。该假设与证据不符:

- 空文档只会有第 1 行；截图与浏览器探针均显示多行正文和换行布局。
- `EditorState.create({ doc: props.modelValue })` 在动态导入完成后读取当前 prop，不需要额外回写。
- CodeMirror 构造时没有需要过滤的伪 `docChanged`；聚焦测试证明过滤第一次事件会吞掉用户第一次真实编辑，使父级仍保留旧内容。
- 后端空内容日志不对应本次故障，还会给合法的空文件制造噪声。

因此最终实现删除上述防御性改动，仅保留 CSP nonce 接入与显式正文颜色。

## 自动化回归

`tests/code-source-editor.smoke.test.ts` 新增三项契约:

1. 页面存在 CSP nonce 时，CodeMirror 生成的运行时 `<style>` 必须继承该 nonce。
2. CodeMirror 生成的 `.cm-content` 与 `.cm-line` 规则必须直接包含 `color: var(--text-primary)`。
3. 第一次真实文档变更必须立即触发 `update:modelValue`。

nonce 回归在修复前稳定收到空字符串，接入 `EditorView.cspNonce` 后通过。第三项测试在早期 `firstChangeSuppressed` 实现下稳定失败，恢复正常 update listener 后通过。

## 验证

- 聚焦 smoke: `bunx vitest run --config vitest.smoke.config.ts tests/code-source-editor.smoke.test.ts`
- 前端: `bun run type-check`, `bun run lint`, `bun run test`
- Tauri: `bun run tauri:check`
- production 构建: `just tbuild`。
- release WebView2: Claude/Codex 在浅色和深色下正文均可见；每次页面加载生成的 CodeMirror `<style>` 均带非空 nonce、`style.sheet` 可读、约 147 条规则，`.cm-scroller` 为 flex，gutter 与正文 top 完全相同且 gutter 宽 38.5px。

## 回滚

回滚 `CodeSourceEditor.vue` 的 `EditorView.cspNonce` 接入、两条显式颜色规则及对应 smoke 断言即可。不要恢复首次事件抑制逻辑。
