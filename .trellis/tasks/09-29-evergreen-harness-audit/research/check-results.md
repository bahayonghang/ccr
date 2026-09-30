# 本轮检查记录

> 历史快照：本文保留实施批准前的审查状态与失败记录。批准后的改动和验证见 [实施与验证记录](implementation-results.md)。

日期：2026-09-29（America/Chicago）。基线 fe9d65972376dbb4c332fde9860c1e42ff2f2005。
所有结果均为原始基线检查，没有先修复再报告成功。原始 .log 受仓库日志忽略规则控制；命令、退出码、摘要与 SHA-256 留在任务。完整元数据见 check-results.json。

| 命令 | 退出码 | 秒 | 日志 |
|---|---:|---:|---|
| `actionlint -color` | 0 | 0.24 | `actionlint.log` |
| `cargo audit` | 0 | 3.72 | `cargo-audit-root.log` |
| `cargo audit --file ccr-ui/src-tauri/Cargo.lock` | 0 | 3.91 | `cargo-audit-tauri.log` |
| `just check-workspace` | 0 | 24.63 | `check-workspace.log` |
| `just ci-governance-check` | 0 | 38.87 | `ci-governance-check.log` |
| `just copilot-check` | 1 | 4.97 | `copilot-check.log` |
| `just docs-check` | 0 | 11.39 | `docs-check.log` |
| `just fmt-check` | 0 | 6.48 | `fmt-check.log` |
| `just frontend-audit` | 1 | 4.54 | `frontend-audit.log` |
| `CCR_SKIP_ICON_GENERATION=1 just frontend-build` | 0 | 11.14 | `frontend-build.log` |
| `just frontend-check-arch-boundaries` | 0 | 21.18 | `frontend-check-arch-boundaries.log` |
| `just frontend-check-cycles` | 0 | 5.63 | `frontend-check-cycles.log` |
| `just frontend-coverage` | 0 | 136.72 | `frontend-coverage.log` |
| `just frontend-lint` | 0 | 40.77 | `frontend-lint.log` |
| `just frontend-test` | 0 | 116.38 | `frontend-test.log` |
| `just frontend-typecheck` | 0 | 18.12 | `frontend-typecheck.log` |
| `just lint-strict` | 0 | 62.8 | `lint-strict.log` |
| `bun test scripts/trellis/omp-context.test.ts` | 1 | 1.07 | `omp-context.log` |
| `bun test scripts/trellis/omp-context.test.ts (only tracked test file in owned temporary fixture)` | 1 | 0.15 | `omp-tracked-fixture.log` |
| `just tauri-ci` | 0 | 343.92 | `tauri-ci.log` |
| `just test` | 0 | 252.8 | `test.log` |
| `just version-check` | 0 | 2.24 | `version-check.log` |
| `just vscode-ci` | 0 | 25.97 | `vscode-ci.log` |

## 通过计数

- Rust 工作区：1867 passed、16 ignored、0 failed；41 个结果段，含空 doctest 段。
- React：i18n 24/24；169 文件、904 smoke 测试；Lines 75.21%（13083/17395），阈值 70%。
- Tauri 主后端：407 passed、1 ignored、197 export 过滤；guard 2 passed；后续绑定生成 197 passed，inventory 1 passed，另有聚焦共享 crate 契约测试。生成绑定检查恢复原工作区字节。
- VS Code：51 passed、0 failed；VSIX 包含不应交付的 .serena 和 source map，不能凭打包退出 0 判定内容合格。

## 失败摘要

- frontend-audit：18 条未批准公告（9 high、6 moderate、3 low）；4 个已定位传递开发依赖，原始 JSON 已保留。
- omp-context：3 pass / 2 fail，一项实际遗漏 design/implement，一项旧标题断言。
- copilot-check：18 个字段误报缺失，根因为 LF 独占解析遇到 CRLF。
- omp-tracked-fixture：缺失 ignored .omp 导入，0 pass / 1 fail / 1 error；仅为导入夹具，不是完整 clean clone。

## 未运行或不能替代的检查

- 原始整条 just ci：含 version-sync/fmt 修复，未运行；组成项没有合并宣称整条 PASS。
- Rust/Tauri llvm-cov、VS Code coverage、Linux/macOS、MSRV 1.95、release、原生 GUI、新鲜五客户端加载/trust 和当前 HEAD hosted CI：UNVERIFIED。
- frontend-build 设置 CCR_SKIP_ICON_GENERATION=1：已提交图标检查通过；未验证图标重生成。
- cargo audit 两个锁文件均退出 0，Tauri 9 条 warning 仍开放。
- Bun 实际 1.4.2，项目声明 1.4.0；Node 26.7.0；Rust 1.98.0；just 1.58.0；cargo-audit 0.22.2。

## Hosted 原始记录

- Dependabot 36370961840：failure，SHA 7ab923ecfb302ba041128de852d90d6a5aca8c86。
- Root CI 32684239641：failure，SHA fac5611f5587dcbe842ef8c133072a26f7ef7be5。
- Tauri CI 32684151306 / Frontend CI 32684151228：failure，SHA 8fb8f20ac551f2133e4e9c5190fe6f82ac9d8455。
上述 gh run view 的 jobs JSON 及失败日志保存在 evidence/；旧 SHA 不代表当前 HEAD。
