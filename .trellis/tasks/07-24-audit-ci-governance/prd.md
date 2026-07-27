# CI 与契约治理补齐

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P1-11、P2-03、P2-04、P2-15、P2-16、P2-17、P3-03 ｜ 报告 Epic E1-E3/E5-E7

## Goal

让 hosted CI 覆盖全部产品面并把设计规范变为 required checks，消除"local `just ci` 强于 hosted CI"的漂移。

## 背景 / 证据（已核实）

- 仓库仅 3 个 workflow：`ci.yml` / `frontend-ci.yml` / `release.yml`
- `ci.yml:6-12` — paths 不含 `ccr-ui/src-tauri/**`；Tauri Rust（315/323 高权限 commands）仅 tag release 时编译（P1-11）
- `release.yml:155-197` — VSIX 仅 tag 时 build，未跑已定义的 `lint`/`test`；无 PR workflow 覆盖 `ccr-vscode/**`（P2-03）
- `frontend-ci.yml:5` — PR branches 仅 `[main, develop]`，root `ci.yml:5` 已含 dev（分支不一致，P2-04）
- `ci.yml:130` — `cargo test --workspace --all-features -- --test-threads=1`；无 coverage job（P2-15）
- `ci.yml:26,40` `dtolnay/rust-toolchain@stable`、`frontend-ci.yml` Bun `latest`、actions mutable major tags（P2-16）
- MSRV 分裂：多数 crate `1.90`，`ccr-db`/`ccr-types`/`src-tauri` `1.88`；`scripts/check-dependency-drift.sh` 有 allowlist 但未进 hosted CI（P2-17）
- spec `typed-ipc-bindings.md:56` 写 312/320，`handler_registry.rs:502,505` 测试断言 323/315（P3-03）

## Requirements

### 新增 hosted 门禁
- [x] 新增 `tauri-rust-ci.yml`：fmt/check/clippy/test/bindings，Linux 每 PR，Win/macOS smoke（P1-11, E1）
- [x] 新增 VSCode PR job：`npm ci && npm run lint && npm test && npm run build:package`（P2-03, E2）
- [x] `frontend-ci.yml` PR branches 加 `dev`，与 root/Tauri/VS Code PR branch policy 统一（P2-04, E3）
- [x] 四个 PR workflow 始终生成稳定 required context；集中 relevance policy 只在命中产品路径时运行重验证，避免 branch protection 因 context 缺失永久等待
- [x] 将 Tauri / VS Code checks 设为 required branch protection；`main`/`dev` 均已启用四个稳定 context、strict checks 与 admin enforcement

### 治理 quick wins
- [x] hosted 加 dependency drift job + bindings drift；allowlist 设 owner/rationale/expiry，当前 1 个、目标 ≤3（P2-17, E6）
- [x] 文档计数从 registry test/codegen 生成，修正 312/320→315/323 并加 docs drift job（P3-03, E5）
- [x] pin Rust 1.95.0/MSRV 1.95、Bun 1.3.10、Node 24.18.0；第三方 action pin commit SHA；接入 Dependabot（P2-16）

### 测试并行化与覆盖率
- [x] 移除全局 `--test-threads=1`：tests 用 temp `CCR_ROOT`/HOME/DB，共享进程状态使用显式 mutex fixture；默认并行（P2-15）
- [x] 加 coverage jobs/thresholds：root Rust workspace 总体行覆盖率 ≥70%，Vue/VS Code 行覆盖率 ≥70%，root/Tauri 关键 gateway ≥85%；serial-only annotation 当前/目标均为 0

## Acceptance Criteria

- [x] 改 Tauri/VSCode 的 PR 必触发对应 check 并设 required branch protection（hosted 产品面 2/4→4/4）
- [x] dev push/PR 触发 frontend CI（branch coverage 3/3）
- [x] 依赖/bindings/docs drift PR 必失败
- [x] 同一 commit 重跑工具版本一致（pin 生效）
- [x] `just ci` 与 hosted workflow 调同一脚本，结果等价

## Out of Scope

- 不以降低 coverage target、扩大 ignore 或继续全局单线程来制造绿灯
- 不在缺少 GitHub 仓库管理权限时宣称 required branch protection 已配置
- 不把 release 签名实现混入本子任务；这里只保证相应 workflow 门禁可被要求

## Notes

- CI 时间增加：Linux required、Win/macOS 起步可 nightly/smoke（报告阶段 0 回滚：drift job 可暂 non-blocking 一周）
- 本子任务 quick wins（frontend-ci 加 dev、pin 版本、修文档计数）可最先做，成本 <1d
- 测试并行化改动大，作为独立 PR，避免与门禁新增混在一起
- 2026-07-27 已使用 keyring 凭据配置并回读 `main`/`dev` branch protection：两者均 `protected=true`、`strict=true`、`enforce_admins=true`，四个 required contexts 绑定 GitHub Actions app `15368`

## Verification evidence (2026-07-27)

| Evidence | Result |
| --- | --- |
| `python -m unittest scripts/test_check_workflow_governance.py` | PASS：10/10，覆盖路径 relevance、稳定汇总 job、事件/YAML 解析、Tauri Bun/fresh-checkout fixture |
| `python scripts/check_workflow_governance.py` | PASS：52 immutable action refs；serial-only 0/0；四个稳定 required context；Tauri Linux 固定 Bun 1.3.10 |
| `just ci-governance-check` | PASS：19 repeated dependencies；1 active exception；315/323 registry doc |
| `just coverage-rust` | PASS：workspace lines 70.10%；gateway 93.20% |
| `just coverage-tauri` | PASS：full baseline 39.90% reported；gateway 95.57% enforced |
| `just frontend-coverage` | PASS：Vue lines 74.54% |
| `just vscode-coverage` | PASS：lines 91.79%；functions 86.87% |
| `just tauri-ci` / `just vscode-ci` | PASS；Tauri 293 + 2 tests；bindings/inventory 同步 |
| `just frontend-check` | PASS：103 files / 460 smoke tests；docs audit/build PASS |
| `just frontend-audit` | PASS：初始 1 critical + 9 high 已整改；仅余 `GHSA-mh99-v99m-4gvg` 的版本数据库命中，1.1.16/2.1.2 已通过 Bun patch 委托至安全 5.0.8，结构化 exception 为 1/1 且 2026-08-31 到期 |
| Workflow YAML parse | PASS |
| `actionlint v1.7.7` | PASS：Root/Frontend/Tauri/VS Code 四个 workflow |
| Hosted `dev` push | PASS：`Frontend CI` run `30242564309` 在 `50771c9e` 成功 |
| `main` / `dev` branch protection | PASS：两者均 protected；strict required checks + admin enforcement；四个 contexts 绑定 app `15368`；force-push/deletion disabled |
| Hosted PR matrix | PASS：PR #42 head `133842b3`；Root `30252249630`、Tauri `30252249641`、Frontend `30252249690`、VS Code `30252249627`；四个稳定 contexts 与 Linux/Windows/macOS/coverage 全部 SUCCESS |
| `just version-check` / final `just ci` | BLOCKED by excluded parallel 7.0.0 metadata: `ccr-ui/README.md` lacks `version-7.0.0` |
