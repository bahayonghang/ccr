# ccr-ui-frontend-vue 引用核查（AC2）

执行时间：2026-08-25
命令：`rg -n 'ccr-ui-frontend-vue' --glob '!.git' .`

| 命中 | 处置 |
|---|---|
| `ccr-ui/package.json` `"name"` | 需要同步改 → `ccr-ui-frontend` |
| `ccr-ui/bun.lock` workspace `name` | 历史记录不动（lockfile；AC1 排除） |
| `.trellis/tasks/08-25-arch-drift-docs/prd.md` | 历史记录不动（规划原文描述旧名） |
| `.trellis/tasks/08-25-react-home-style-redesign/prd.md` | 历史记录不动（父任务规划原文） |

脚本、CI、workspace、justfile 均无按该包名引用。根 `AGENTS.md` 的 “Vue 3 + Tauri” 属本任务范围外（仅改 `code_map.md`、`ccr-ui/package.json`、`ccr-ui/CLAUDE.md`）。
