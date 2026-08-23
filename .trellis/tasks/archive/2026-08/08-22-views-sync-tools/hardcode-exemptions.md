# 硬编码豁免（AC11）

CodeMirror 主题内联样式允许 px，登记如下：

| 文件 | 字面量 | 原因 |
| --- | --- | --- |
| `ccr-ui/src/features/editor/editorTheme.ts` | `fontSize: '13px'`、`padding: '14px 0'` | CodeMirror `EditorView.theme` 运行时 stylesheet，AC11 豁免 |
| `ccr-ui/src/features/tray/tray-format.ts` | `TRAY_PANEL_MANUAL_MOVE_THRESHOLD_PX = 12` | 拖动阈值常量，非样式 |

其余新组件样式使用 rem / token / `var(--space-px)`。
