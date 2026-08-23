# CodeMirror `@uiw/react-codemirror` 可表达性（AC4.1）

判定标准：`design.md` §3.3。能通过 `extensions` 数组表达的不算无法表达。

| 契约项 | `@uiw` 表达 | 判定 |
| --- | --- | --- |
| JSON / Markdown / TOML 语法高亮 | `extensions` 传入 `json()` / `markdown()` / `StreamLanguage.define(toml)` | 可表达 |
| lint 提示 | `lintGutter()` + `setDiagnostics`（`onCreateEditor` / `ref.view`） | 可表达 |
| 搜索替换 | `search()` + `searchKeymap` | 可表达 |
| 撤销重做 | `history()` + `historyKeymap` | 可表达 |
| 快捷键（含 Mod-s 保存、Tab 缩进） | `keymap.of([...])` | 可表达 |
| 受控值同步 | `value` + `onChange` | 可表达 |
| `Compartment` 动态重配置 | 语言/只读变化时重建 `extensions`；`@uiw` 会重配 | 可表达 |
| CSP nonce | `EditorView.cspNonce.of(pageNonce)` 放入 `extensions` | 可表达 |
| 主题 | `EditorView.theme(...)`，`theme="none"` 关闭 `@uiw` 默认主题 | 可表达 |

无法表达项数：**0**（≤ 3，继续使用 `@uiw/react-codemirror` 4.25.11）。

9 个 `@codemirror/*` 包保留。`basicSetup={false}`，避免第二套默认扩展。
