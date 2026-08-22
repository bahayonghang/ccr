# 分布测量记录（批次 1，AC4 输入）

> 任务：`08-22-arch-quality-perf` 批次 1。测量方法与排除清单见 `design.md` §3.1；测量脚本 `ccr-ui/scripts/measure-distribution.mjs`（可重复执行，输出 JSON）。
>
> - 测量日期：2026-08-23
> - 分支：`react-migration/react-foundation`（React 基座已合入，185 个 `.vue` 仍在树但已整体退出 lint 管线）

## 关键适配：排除清单在活文件集上为空操作

`design.md` §3.1 第 2 步要求排除「将被统一层接管的 20 个文件」。经 `path-mapping.md` 偏差登记修正，该清单为 **18 个收敛文件 + 3 个 `views/generic/` base 本体 = 21 个**，且全部是 `.vue`。基座批次 3 起 `**/*.vue` 已整体加入 eslint ignores，因此这 21 个文件**本就不在测量集合内**——排除步骤在活文件集（`.ts`/`.tsx`）上是空操作，暂定分布 = 全量 `.ts`/`.tsx` 分布。脚本仍保留排除逻辑，供批次 3b（阶段 4 后冻结）在统一层 `.tsx` 落位后复用。

## 活文件集分布（`src/**/*.{ts,tsx}`，排除 `src/types/generated`，排除 21 个统一层文件）

217 个文件。复杂度/深度/参数以「文件内最差函数」计，经临时 ESLint 配置（`complexity` / `max-depth` / `max-params`，max=0，warning 级，不提交）取数；行数按物理行计。

| 指标 | P50 | P75 | P90 | P95 | max | 均值 |
| --- | --- | --- | --- | --- | --- | --- |
| 行数 | 84 | 216 | 414 | 625 | 6,058 | 232.6 |
| 圈复杂度 | 5 | 9 | 16 | 20 | 51 | 7.1 |
| 最大嵌套深度 | 1 | 2 | 3 | 3 | 4 | 1.6 |
| 最大参数个数 | 2 | 3 | 4 | 4 | 7 | 2.2 |

排除前（仍不含 generated）与排除后分布完全一致（见上节适配说明），数据同表。

### 行数 Top 8（活文件集）

| 文件 | 行数 | 备注 |
| --- | --- | --- |
| `src/api/generated/commandCapabilities.ts` | 6,058 | 生成数据表，零逻辑 |
| `src/i18n/locales/en-US.ts` | 5,457 | 翻译数据表 |
| `src/i18n/locales/zh-CN.ts` | 5,301 | 翻译数据表 |
| `src/i18n/bootMessages.ts` | 1,204 | 文案数据表 |
| `src/stores/usage.ts` | 992 | 复杂度 27，归 `08-22-state-logic-port` |
| `src/api/domains/codex.ts` | 953 | |
| `src/api/tauri.ts` | 737 | 冻结门面，只读 |
| `src/types/checkin.ts` | 668 | 类型数据表 |

行数 max 被纯数据表（生成物、i18n、类型表）占据，这类文件拆分无收益，处置见 `thresholds.md` 超限清单（逐文件登记豁免或拆分批次）。

## `.vue` 历史分布（上下文参考，不参与阈值取值）

185 个文件，已整体退出 lint 管线，阶段 4–5 迁移后离开树。

| 指标 | P50 | P75 | P90 | P95 | max | 均值 |
| --- | --- | --- | --- | --- | --- | --- |
| 文件行数 | 389 | 605 | 1,024 | 1,174 | 1,745 | 462.1 |

## 组件内样式行数分布（历史上下文）

139 个 `.vue` 组件带 `<style>` 局部样式，合计 24,573 行（父任务测量口径 24,434，本表为当前树重测值，含批次间漂移）。

| 指标 | P50 | P75 | P90 | max |
| --- | --- | --- | --- | --- |
| 单组件样式行数 | 131 | 238 | 412 | 806 |

活文件集（`.tsx`）当前无组件级局部样式文件，样式行数约束（单组件局部样式 ≤ 其 JSX 行数，父任务 `design.md` §6）以检查脚本对 `.tsx` + 配对 `.module.css` 生效，当前零违例；基线在首个带样式的组件落地后于批次 3b 补测。

## 复测方式

```bash
cd ccr-ui && bun scripts/measure-distribution.mjs
```

输出 JSON 含 `liveTsTs` / `fullTsTsBeforeExclusion` / `vueHistorical` / `top20ByLines` 四段。批次 3b 冻结时以同一脚本重跑，替换排除清单为统一层实际文件集合。
