# 父任务 AC1–AC23 核对（本任务 AC15）

> 父任务：`08-22-react-migration` `prd.md`。发布门补测后更新（2026-08-24）。

| 父 AC | 内容 | 状态 | 依据 |
| --- | --- | --- | --- |
| AC1 | package.json 无 Vue 系依赖条目 | 满足 | `ccr-ui/package.json` |
| AC2 | `ccr-ui/src` 下 `.vue` 为 0 | 满足 | glob 0 |
| AC3 | `just ci` 退出码 0 | 满足 | scratch `just-ci.log` JUST_CI_EXIT=0，14 步 |
| AC4 | smoke ≥122 | 满足 | 534 tests |
| AC5 | Tailwind v4，448 token | 满足 | design-system + theme smokes |
| AC6 | 组件内 px / rgba 归零 | 满足 | hardcode-px-rgba 31==豁免 |
| AC7 | 弹层单一 Dialog | 满足 | overlay-single-implementation.smoke |
| AC8 | IPC/Event 名一致 | 满足 | api-facade + tauri-event-inventory |
| AC9 | 打包可启动 + 四项运行时 | 满足（WAF 跳过） | 见 `gate-release.md` |
| AC10 | 19 份契约 | 满足 | test-contract |
| AC11 | 185 逐屏，未判定 0 | 满足 | `screen-comparison.md` |
| AC12 | audit 无新增高危 | 满足 | just ci Security Audit；`bun run audit:dependencies` 0 |
| AC13 | Cargo 升级 + ts-rs | 满足 | 2b 已归档 |
| AC14 | 分层/门面/循环 error | 满足 | lint:ci |
| AC15 | 行数/复杂度上限 | 满足 | lint:ci |
| AC16 | react-hooks error | 满足 | lint:ci |
| AC17 | 覆盖率门 | 满足 | lines 70.03% |
| AC18 | 五性能场景前后数据 | 满足 | `perf-react-after.md` |
| AC19 | 启动/FCP/bundle | 满足 | DCL 不高于 Vue；gzip 重设 `bundle-reset.md` |
| AC20 | 路由分割与三层 CSS | 满足 | code-splitting.md |
| AC21 | 差异矩阵 | 满足 | platform-unify |
| AC22 | base 无平台名分支 | 满足 | platform-surface-unify.smoke |
| AC23 | 平台 × 功能面 | 满足 | platform-unify |

WAF 真实签到与 2h soak 为跳过，不记为产品缺陷。
