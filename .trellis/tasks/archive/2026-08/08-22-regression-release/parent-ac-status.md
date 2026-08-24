# 父任务 AC1–AC23 核对（本任务 AC15）

> 父任务：`08-22-react-migration` `prd.md`。2026-08-24 对齐：AC6 / AC13 未通过时，父 AC9 与本任务 AC15 不勾选。

| 父 AC | 内容 | 状态 | 依据 |
| --- | --- | --- | --- |
| AC1 | package.json 无 Vue 系依赖条目 | 满足 | `ccr-ui/package.json` |
| AC2 | `ccr-ui/src` 下 `.vue` 为 0 | 满足 | glob 0 |
| AC3 | `just ci` 退出码 0 | 满足 | 当前 HEAD 重跑：scratch `just-ci.log` 325047 字节，`JUST_CI_EXIT=0`，14 步，TOTAL 05:25.521 |
| AC4 | smoke ≥122 | 满足 | 534 tests |
| AC5 | Tailwind v4，448 token | 满足 | design-system + theme smokes |
| AC6 | 组件内 px / rgba 归零 | 满足 | hardcode-px-rgba 31==豁免 |
| AC7 | 弹层单一 Dialog | 满足 | overlay-single-implementation.smoke |
| AC8 | IPC/Event 名一致 | 满足 | api-facade + tauri-event-inventory |
| AC9 | 打包可启动 + 四项运行时 | 不满足 | 启动/CSP/chrome/杀进程再启动已测。WAF 真实签到未做。四项合取不通过 |
| AC10 | 19 份契约 | 满足 | test-contract |
| AC11 | 185 逐屏，未判定 0 | 满足 | 主线程计数：185 行，一致 146，可接受差异 39，未判定 0，缺陷 0。基线 75×2 PNG |
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

本任务 AC15 不勾选：父 AC9 不满足（WAF 真实签到凭据未提供）。本任务 AC6 / AC13 保持 `[ ]`。子任务 AC13 第 3 轮 persist-raw-cdp 堆比 1.295，见 `soak-packaged-round3.jsonl`。
