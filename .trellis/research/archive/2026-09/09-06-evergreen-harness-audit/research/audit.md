# CCR 常青维护审查报告

日期：2026-09-06。基线：dev / 77058135；开始时工作区干净。访问方式：本地 PowerShell/CLI、gh 只读、官方文档；未操作 GUI，未执行其他 harness 会话或计费模型调用。审查使用当前主模型和两个继承主模型的独立审查代理；没有把规划交给便宜模型。

## 结论

当前不是需要大规模重构的状态。Rust/Tauri/扩展的现有测试通过；实际阻断是前端 route smoke 夹具失真。另有两个能让 CI 错报绿灯的缺口，以及五工具规则和 OMP 上下文不一致。建议先批准四个 P1 子任务，P2 历史证据复核可独立批准。

## 项目结构与关键边界

| 入口/模块 | 已读关键文件 | 实际职责与约束 |
|---|---|---|
| 项目导航 | code_map.md:5、Cargo.toml:1 | 根为 13 个成员的 Rust workspace，ccr-ui/src-tauri 是独立 workspace；不能只跑根 cargo test 就声称桌面已通过 |
| CLI/TUI | crates/ccr/src/main.rs:1、crates/ccr/src/lib.rs:33 | main 经 CommandDispatcher 到 ccr-cli；ccr 为 facade，领域在各 crate；现有 deprecated re-export 是明确 7.x 契约，本次不擅自清理 |
| 核心/配置/数据 | crates/ccr-core、ccr-config、ccr-codex、ccr-db、ccr-store | 配置、锁、敏感持久化、诊断、SQLite 的实际所有者；仅围绕失败链深入检查 |
| Usage | crates/ccr-usage/Cargo.toml:7、ccr-ui/src-tauri/Cargo.toml:29、.trellis/spec/ccr/backend/llmusage-provider-adapter.md:10 | 外部 llmusage CLI 同步 + SQLite 只读查询；共享投影属于本地 ccr-usage，禁止链接上游 crate |
| UI/IPC | ccr-ui/package.json、ccr-ui/src/api/generated/agentSessions.ts:15、ccr-ui/src/features/agent-sessions/AgentSessionsView.tsx:216 | React 19 / Query / Tauri typed IPC；生成类型来自 Rust，测试必须遵守真实 DTO |
| 扩展/文档 | ccr-vscode/package.json、ccr-vscode/src、docs/package.json | Node/TypeScript 扩展与 VitePress；有各自 lint/test/build |
| 验证与规则 | justfile:395、.github/workflows、scripts/ci/ci_surface_policy.py:25、.trellis/workflow.md | 区分检查、自动改写/安装、副作用与 hosted 证据；三件套计划不等于实施批准 |

本报告是结构、现有测试和失败工作流驱动的审查，不声称穷尽所有业务/安全缺陷。

## 当前确认的发现

### F1 · P1 · Agent Sessions 路由 smoke 使用错误返回形状

证据：ccr-ui/tests/shell/route-view-mount.smoke.test.tsx:70 以名称包含 session 匹配，第 76 行把函数 stub 返回 []；同文件第 81–86 行的 wrapValue 会递归替换 agentSessionsApi 中的函数。agentSessionsStartRefresh 实际返回 StartSessionIndexJobResponse（ccr-ui/src/api/generated/agentSessions.ts:15），含 job_id + snapshot（ccr-ui/src/types/generated/usage/StartSessionIndexJobResponse.ts:4）。真实后端在 ccr-ui/src-tauri/src/commands/agent_sessions.rs:104 返回完整对象。

因果链：路由 bootstrap refresh → 名称型 mock [] → refreshMutation.data 存在但 snapshot 不存在 → AgentSessionsView.tsx:216 读取 snapshot.status → ErrorBoundary → 两种语言的路由测试失败。全套和单文件原参数均复现。当前证据指向 fixture 契约问题，尚未做修复实验；不能称真实桌面存在相同错误。最小方案：该域显式 typed fixture，保持路由失败断言，不给产品代码补无依据的可选链。

### F2 · P1 · VS Code coverage 管道可能吞失败

.github/workflows/vscode-ci.yml:49 使用 just vscode-coverage | tee vscode-coverage.txt，未显式 shell。默认 Linux bash -e 不带 pipefail；显式 shell: bash 带 pipefail。[GitHub 官方 shell 语义](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstepsshell)

实证：bash -e -c 'false | tee /dev/null' 返回 0；bash --noprofile --norc -eo pipefail -c 'false | tee /dev/null' 返回 1。仅需给该步骤加 shell: bash，验证失败与成功分支；不新增 shell 配置框架。

### F3 · P1 · CI 漏判 Cargo 配置输入

scripts/ci/ci_surface_policy.py:25 的路径集合不包含 .cargo/tauri-ci.toml、.cargo/config.toml、.cargo/audit.toml。调用 is_relevant 对每个文件单独测试，root/frontend/tauri/vscode 全 False。.cargo/tauri-ci.toml:2 设置 TAURI_CONFIG，被 justfile:1213 和 ccr-ui/justfile:1545 等读取。只改该输入时相关验证会 skip，required gate 可成功。

最小方案：按真实消费者补精确输入集合与现有路径单测，不增加全仓任意变更跑全部 CI 的规则。tauri-ci.toml→tauri；config.toml→root+tauri；audit.toml→root。根 .github/workflows/ci.yml:148 消费 cargo audit；当前 Tauri workflow/justfile 没有该消费者。

### F4 · P1 · 项目说明冲突且 Claude 共享入口无效

AGENTS.md:4、:27 仍写 Vue 3/.vue 与 pinned 上游 llmusage crate；CLAUDE.md:53 和当前契约要求 CLI+SQLite，code_map.md:8 写 React 19。CLAUDE.md:43 的 scope imports 及 :45 的 @AGENTS.md 都置于行内代码中，且后者限定 OpenSpec，不能作为实际共享加载桥。[Claude 官方 memory/import 说明](https://code.claude.com/docs/en/memory)

先纠正 AGENTS 再建立真实 import；共用项目事实只维护一处。读取全局规则的要求保留在工具加载契约中，不把用户全局规则复制五份进仓库。

目录级入口也未同步：ccr-ui/code_map.md:17、:20、:28 仍指 Vue/Pinia/旧路由，:45 仍为 pinned llmusage；ccr-ui/CLAUDE.md:109、:138 为 Anthropic 编辑式方向，与 ccr-ui/AGENTS.md:35 指向 DESIGN.md 的行情终端相冲突。crates/ccr/src/CLAUDE.md:40 的旧 src 服务/命令树与实际 facade 不符。crates/code_map.md 的 crate 清单遗漏当前 ccr-usage。规则子任务将这些已存在的入口一起纠正，避免只修根文件后下层覆盖回旧事实。ccr-vscode 的 AGENTS/CLAUDE 与已读实现未发现同等级冲突；docs/AGENTS 要求中英镜像，新 harness 说明须同时交付英文页。

### F5 · P1 · OMP 注入漏复杂任务设计和执行计划

.omp/extensions/trellis/index.ts:274 的 buildTaskContext 只读 prd.md/info.md 和按角色分派的 jsonl；不读 design.md/implement.md。.omp/agents/trellis-implement.md:23 和 trellis-check.md:24 也没有补读要求。与 .trellis/workflow.md:161 的三件套契约不一致。源码缺口确认；真实 OMP 会话影响仍 UNVERIFIED。

最小方案：补加载 design/implement（存在时），保持现有角色 manifest 选择与路径信任边界；补默认 extension 入口的行为测试及三角色手工拉取说明。无需新增通用 context 系统。

### F6 · P2 · harness 能力说明、角色权限与 skill 路由漂移

.kimi-code/skills/trellis-implement/SKILL.md:49 与 check 同行称 Kimi 无项目自定义代理；平台地图称 Grok/Kimi 无 hooks；当前官方均已提供相应能力。应描述“本仓库使用手工拉取”，不能写成平台上限。Trellis check 在 .codex/agents/trellis-check.toml:25 等是可自行修改的执行角色，不能用于审批前只读审查。

.codex/skills/ccr-ui-visual-workflow/SKILL.md:25 仍有 Vue；gate-recovery 仅描述 Codex且示例将测试串行化。统一路由要明确五工具适用性、保持原并行测试、不将 UI 工具可用视为 UI 操作授权。CLAUDE.md:16 的 xhigh 也不能作为五工具通用参数。

## 本地检查账本

命令均使用 rtk 支持的代理/透传；表内保留有效底层命令。Rust 为避免锁漂移和下载，加 --locked --offline。现有依赖已可用，未安装。

| cwd | 命令 | 结果 |
|---|---|---|
| 根 | just version-check | PASS；版本 7.3.0、文档锁策略及 dependency drift |
| 根 | just fmt-check | PASS；5 个 JSON 格式测试、11 文件，根/Tauri fmt |
| 根 | just dependency-governance-check | PASS；6 tests，19 重复依赖，1 明示 toml 例外 |
| 根 | just ci-governance-check | PASS；22 workflow tests、45 immutable actions、6 dependency tests、1 Tauri inventory test |
| 根 | cargo test --workspace --all-features --locked --offline | PASS；1739 passed / 0 failed / 15 ignored，含 doctests，共 40 result groups |
| 根 | cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings -D clippy::unwrap_used | PASS |
| 根 | python -X utf8 scripts/quality/check_secret_writes.py | PASS |
| 根 | cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked --offline | PASS；515 单测 + 2 no-crate guards |
| ccr-ui | bun run type-check | PASS |
| ccr-ui | bun run lint:ci | PASS；ESLint/stylelint/style-lines |
| ccr-ui | bun run check:cycles | PASS；711 文件无循环 |
| ccr-ui | bun run check:arch-boundaries | PASS；4 种违规夹具被正确拒绝 |
| ccr-ui | bun run test | FAIL；test:i18n PASS；smoke 151 文件中 150 pass/1 fail，722 tests 中 720 pass/2 fail，106.44s |
| ccr-ui | bun run test:smoke -- tests/shell/route-view-mount.smoke.test.tsx | FAIL；2 pass/2 fail，26.42s，重复同一 status 错误 |
| ccr-vscode | npm run lint | PASS |
| ccr-vscode | npm test | PASS；51 tests / 12 suites；pretest build PASS |
| ccr-vscode | node --experimental-test-coverage --test-coverage-lines=70 --test-coverage-functions=70 --import tsx --test 'src/**/*.test.ts' | PASS；line 91.86%、branch 81.08%、function 91.50%，仅加载文件 |
| docs | bun run audit | PASS |
| docs | bun run build | PASS；VitePress 1.6.4，4.74s |

UI 原始失败摘要：

~~~text
FAIL tests/shell/route-view-mount.smoke.test.tsx
zh-CN :333 / en-US :353
/agent-sessions: 页面渲染失败Cannot read properties of undefined (reading 'status')
AgentSessionsView.tsx:216:82
Test Files 1 failed | 150 passed (151)
Tests 2 failed | 720 passed (722)
~~~

工具环境：cargo 1.98.0，Bun 1.4.2；仓库/hosted 固定 Bun 1.4.0，本机结果不代替 pinned CI。原始本地日志保存在 OS temp 的 ccr-evergreen-{rust-tests,tauri-tests,ui-tests,ui-focused,ui-lint,rust-clippy}.log；持久化证据以本报告中的命令、摘要和源码锚点为准，临时日志可能被清理。

## 历史工作流与当前证据分离

| 工作流 | 首个失败点 | 根因深度与当前结果 |
|---|---|---|
| [Root 32684239641](https://github.com/bahayonghang/ccr/actions/runs/32684239641)，2026-08-24，fac5611f5587dcbe842ef8c133072a26f7ef7be5 | coverage 内 ccr-cli 332 pass/1 fail；non_dry_run_doctor_persists_sanitized_report：report should be persisted | 当前 crates/ccr-cli/src/commands/codex/fix.rs:1158；save_report :679 用 .ok() 丢失 IO 错误，:571 仍给 failed:false。能确认报告缺失和证据丢失，底层 OS 失败原因 UNVERIFIED；当前根测试全通过 |
| [Tauri 32684151306](https://github.com/bahayonghang/ccr/actions/runs/32684151306)，2026-08-24，8fb8f20ac551f2133e4e9c5190fe6f82ac9d8455 | Windows process smoke 8 pass/2 fail/480 filtered，5.04s；stdin success 与 flood stdout_truncated 断言失败 | 当前 ccr-ui/src-tauri/src/process/gateway.rs:909、:953 为 powershell.exe + 5s；冷启动/并发资源竞争是高可信假设，非已证实根因；本机 Tauri 全通过 |
| [Frontend 32684151228](https://github.com/bahayonghang/ccr/actions/runs/32684151228)，2026-08-24，同 SHA | 626 tests pass 但 1 unhandled EnvironmentTeardownError，Closing rpc while onUserConsoleLog was pending | 历史 usage.store.smoke.test.ts 当前已删除，现为 React；不得修已不存在 Vue 测试。当前 route smoke 是独立新失败 |
| [Release 29002291872](https://github.com/bahayonghang/ccr/actions/runs/29002291872)，2026-07-09 | 成功 | 旧 SHA 268154df6edcaff7fc13e52112b934f7840312c1，不证明当前 HEAD 发布可用 |

历史日志仅在 OS temp：ccr-audit-root-32684239641.log（1915–1927 行）、ccr-audit-tauri-32684151306.log（1495–1519 行）、ccr-audit-frontend-32684151228.log（2164–2179、2537–2539 行）。通过 gh run list/view --log-failed 读取，无重跑/取消/远程写入。

## 运行边界与未验证项

- 未跑聚合 just ci：justfile:569 先 version-sync/fmt，audit 可能安装工具；不适合作为本次原始树只读审查入口。已拆分现有安全检查，不报告 full CI green。
- 未跑 just vscode-ci（npm ci）、Tauri bindings-check（会清理/重建目录）、release 打包、UI production build、Root/Tauri llvm-cov 或前端 coverage。前端基础测试已失败，先修首故障。未安装缺少的覆盖工具。
- Tauri export_bindings 测试生成了 6 个 tracked TS 文件的 8 处尾随空格变化；已记录 diff 并仅恢复本次生成变化，git diff 回到空。已有 normalize-generated-bindings.mjs 和专用 bindings-check，不新增重复修复任务。
- 原有测试忽略项、真实 Windows WebView/桌面体验、Linux/macOS、本提交 hosted CI、provider/账户联通、五套 harness 实際模型解析和 hooks 生效均 UNVERIFIED。
- 未触碰用户全局配置或知识库。批准后回写选择项目说明和现有技能，不需要同时复制到多个知识库。

## 任务与交付

父任务及五个子任务全部 planning；见父任务 design.md/implement.md。本次完成的是审查与可批准规划，子任务的产品验收尚未完成。
