# `overrides` 复核记录（段 2 后，AC9）

> 任务：`08-22-dep-upgrade`。本表为第 2 次复核（段 1 后第 1 次、段 2 Tailwind v4 落地后第 2 次），日期 2026-08-23。
> 判定方法（design.md §5）：`bun pm why <pkg>` 看引入链与所需范围 → 移除 override 后 `bun install` 观察自然解析版本 → `bun run audit:dependencies` 确认无告警。
> 本次复核结论：**9 项全部移除**。移除后 `bun install` 成功、audit 通过（0 reported advisories, 0/0 active exceptions）、`bun run build` / `lint:style` / `test:smoke` 全绿（293/293）。

## 判定表

| 包名 | pin 版本 | 原因 | 升级后传递依赖现状 | 判定 | 依据 |
| --- | --- | --- | --- | --- | --- |
| fast-uri | 3.1.5 | 2026-07-27 安全整改（`bb46226b`）：收敛 ajv→fast-uri 链路的高危版本 | ajv@8.20.0 要求 `^3.0.1`，移除后自然解析仍为 fast-uri@3.1.5 | 移除 | 自然解析与 pin 完全一致，pin 已无约束作用 |
| flatted | 3.4.2 | 同上安全整改：eslint→flat-cache 链路收敛 | flat-cache@4.0.1 要求 `^3.2.9`，移除后自然解析仍为 flatted@3.4.2 | 移除 | 自然解析与 pin 完全一致 |
| js-yaml | 4.3.1 | 同上安全整改：js-yaml 旧版 load() 原型污染面收敛 | @eslint/eslintrc@3.3.6 要求 `^4.3.0`，移除后自然解析仍为 js-yaml@4.3.1 | 移除 | 范围下限已高于等于安全版本，pin 冗余 |
| nanoid | 3.3.18 | 同上安全整改：nanoid <3.3.8 可预测值问题收敛 | postcss@8.5.x 要求 `^3.3.16`，树中仅此一处引入，移除后自然解析仍为 nanoid@3.3.18 | 移除 | 自然解析与 pin 一致；且全局强制 3.3.18 反而会破坏未来引入 nanoid v4/v5 的包 |
| picomatch | 4.0.4 | 同上安全整改：多版本 picomatch 收敛 | 移除后自然解析为 picomatch@4.0.5（原 pin 目标），micromatch 恢复其声明范围 `^2.3.1` 内的 2.3.2 并存 | 移除 | pin 曾把 4.0.4 强制安装进 micromatch 的 `^2.3.1` 范围外；恢复按范围解析后 audit 无告警 |
| postcss | `$postcss` | 单一实例收敛：避免 stylelint/vite/tailwind 各带一份 postcss | v4 经 @tailwindcss/postcss@4.3.3 引入（要求 `^8.5.16`），与直接 devDep `^8.5.23` 合流，移除后树中仅 postcss@8.5.26 一个实例 | 移除 | 合流由范围重叠自然达成，无需自引用 pin；vite 8 底层 rolldown 不再自带独立 postcss 分叉 |
| rollup | 4.61.0 | vite 5–7 时代 vite 自带 rollup 的版本对齐 | vite 8.2.2 底层为 rolldown，依赖树中已不存在 rollup（`bun pm why rollup` 报 No packages matching） | 移除 | pin 指向的对象已从依赖树消失，属死条目 |
| esbuild | 0.28.1 | vite 7 时代 esbuild 版本对齐 | vite 8.2.2 不再依赖 esbuild（`bun pm why esbuild` 报 No packages matching） | 移除 | 同上，死条目 |
| ws | 8.21.0 | 同上安全整改：ws DoS 高危版本收敛 | jsdom@26.1.0 要求 `^8.18.0`，移除后自然解析仍为 ws@8.21.0 | 移除 | 范围下限已覆盖安全修复，pin 冗余 |

## 验证记录

```text
$ bun install
（成功，重排 lockfile）

$ bun pm ls --all | grep -cE "postcss@[0-9]"   → 1 个实例（8.5.26）
$ bun run audit:dependencies
frontend dependency audit passed: 0 reported advisories, 0/0 active exceptions

$ bun run build      → ✓ built in 13.29s
$ bun run lint:style → exit 0
$ bun run test:smoke → Test Files 59 passed, Tests 293 passed
```

## 备注

- 若后续段 3/段 4 或其他子任务重新引入 rollup/esbuild 系工具链（如 vitest 工作区浏览器模式），需在当次变更里重做对应项的传递依赖核对，不回填本表。
- 安全类 pin 的原始动机以提交 `bb46226b`（2026-07-27「收紧依赖审计与兼容补丁治理」）为准；当时审计门禁 fail-closed 所需的例外清单现已清空。
