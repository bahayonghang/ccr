# 硬编码豁免登记（父任务 AC6 / 视图门）

`ccr-ui/src/**/*.{ts,tsx}` 内硬编码 `Npx` 与 `rgba()` / `rgb()` 必须为 0。
不可映射的残留逐条登记；计数权威是 smoke：

`ccr-ui/tests/hardcode-px-rgba.smoke.test.ts`

口径：`\d+px` 与 `rgba?(`，排除 token 形态 `rgb(var(--*-rgb) …)`。
Profiles 密集元信息 `0.75rem` 是字号契约例外，不是 px，不在本清单。

## 计数

| 种类 | 数量 |
| --- | --- |
| px | 22 |
| rgb / rgba | 9 |
| 合计 | 31 |

## 按原因

| 原因 | 文件 | 形态 |
| --- | --- | --- |
| CodeMirror theme | `src/features/editor/editorTheme.ts` | `13px` / `14px` / `1px` |
| ApexCharts canvas | `src/views/usage/usageChartOptions.ts` | `11px`×3 / `15px` / `10px`；`rgb(29 29 31 / 8%)` / `12%` |
| startup fatal HTML | `src/utils/startupRecovery.ts` | 内联 HTML 的 px 与 rgba/hex 回退页 |
| 6px drag | `src/ui/base-modal.tsx` | 注释中的 `6px`（实现常量 `DRAG_THRESHOLD = 6`） |
| themeBootstrap rgb() writer | `src/utils/themeBootstrap.ts` | `rgb(${toTriplet…})` 写入第 1 层 accent 变量 |
| viewport breakpoint | `src/shell/hooks/useMainLayoutShell.ts` | `matchMedia('(max-width: 1023px)')` |

新增字面量：先改 rem / 既有 token；不能改则在 smoke `EXEMPTIONS` 加一条（file + snippet + reason），再更新本表计数。
