# 父任务 AC1–AC23 核对（本任务 AC15）

> 父任务：`08-22-react-migration` `prd.md`。本任务 AC15 要求全部满足。下列「本会话」列为本次能核对的结果，不是把父任务 prd 勾选掉。

| 父 AC | 内容 | 本会话 | 依据 |
| --- | --- | --- | --- |
| AC1 | package.json 无 Vue 系依赖条目 | 满足 | `ccr-ui/package.json` 无 `vue` / `vue-router` / `pinia` / `vue-i18n` 等；包名仍为 `ccr-ui-frontend-vue` |
| AC2 | `ccr-ui/src` 下 `.vue` 为 0 | 满足 | 递归枚举 0 个 `.vue` |
| AC3 | `just ci` 退出码 0，recipe 与 justfile 一致 | 不满足 | 未跑 `just ci`。justfile `_ci-timed-*` 现为 14 步（含 `frontend-coverage`） |
| AC4 | smoke ≥122 且覆盖不降 | 部分满足 | 本会话 `bun run test`：108 文件 / 511 测试通过。覆盖比对表在 `08-22-test-contract-rebuild/coverage-comparison.md` |
| AC5 | Tailwind v4，448 token 两集合 | 部分满足 | `tailwindcss` 4.3.3；token 契约由 design-system 落盘。本会话未重数 448 名 |
| AC6 | 组件内 px / rgba 归零 | 未核 | 视图门条款；本会话未扫 `.tsx` |
| AC7 | 弹层单一 Dialog 原语 | 满足 | `overlay-single-implementation.smoke.test.ts` 3/3 |
| AC8 | IPC 命令与 Event 名一致 | 未核 | test-contract AC6 在其 prd 仍未勾选 |
| AC9 | 打包可启动 + 四项运行时 | 不满足 | 未打包；AC5–AC7 本任务未勾 |
| AC10 | 19 份契约无 Vue/SFC 残留 | 满足（子任务已勾） | test-contract AC3 |
| AC11 | 185 逐屏记录，未判定 0 | 满足 | 本任务 `screen-comparison.md` |
| AC12 | audit 无新增高危 | 不满足 | 未跑 audit |
| AC13 | Cargo 升级清单 + ts-rs 204 diff | 子任务已交付 | `08-22-workspace-cargo-upgrade` implement 已勾；其 prd 勾选未改 |
| AC14 | 分层/门面/循环 error 级 | 满足 | `lint:ci`、`check:cycles`、`check:arch-boundaries` 本会话通过 |
| AC15 | 行数/复杂度/嵌套/样式行数上限 | 部分满足 | `check:style-lines` PASS；复杂度上限未单独重跑全仓报表 |
| AC16 | react-hooks error 无豁免残留 | 部分满足 | `lint:ci` 通过；未单独扫豁免注释 |
| AC17 | 覆盖率门 | 未核 | 未跑 `just frontend-coverage` |
| AC18 | 五性能场景前后数据 | 不满足 | 仅有 Vue 基线；React 侧未测 |
| AC19 | 启动/FCP/bundle 不高于基线或有重设 | 不满足 | 构建失败，无 React 产物体积 |
| AC20 | 路由分割与三层 CSS | 子任务已交付 | `code-splitting.md`；本会话未复测首屏模块集合 |
| AC21 | 差异矩阵 + 20 文件处理 + 行数对比 | 满足（子任务已勾） | platform-unify AC1–AC3 |
| AC22 | base 无平台名分支；薄壳 ≤100 行 | 满足（子任务已勾） | platform-unify AC4/AC7 |
| AC23 | 平台 × 功能面矩阵 + 跨平台用例 | 满足（子任务已勾） | platform-unify AC6/AC10 |

AC15（本任务）不勾选：父 AC3、AC9、AC12、AC18、AC19 本会话不满足。
