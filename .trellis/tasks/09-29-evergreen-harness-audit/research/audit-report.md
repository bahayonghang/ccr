# CCR 常青项目审查报告

> 历史快照：本文保留实施批准前的审查状态与失败记录。批准后的改动和验证见 [实施与验证记录](implementation-results.md)。

- 日期：2026-09-29（America/Chicago）；UTC 日志可能显示 2026-09-30。
- 基线：dev / fe9d65972376dbb4c332fde9860c1e42ff2f2005，版本 7.4.0，初始工作区干净。
- 结论：主要产品测试通过，但五套 harness 尚未完全对齐；安全审计、OMP 契约和跨平台检查存在当前失败。
- 状态：审查与规划完成，实施待批准。本轮只新增本父任务和 8 个子任务，没有修改生产源码、共享说明或 skill。
- 范围：关键结构、测试、检查脚本、打包、五工具入口和近期失败 workflow；不等于逐行审查所有产品源码。

## 1. 项目结构与关键归属

| 区域 | 已核对关键文件 | 责任边界 |
|---|---|---|
| Rust workspace | Cargo.toml；crates/ccr/src/main.rs；crates/ccr-cli/src/lib.rs | CLI/TUI 入口委托领域 crate；测试通过 fixture 隔离进程环境。 |
| 使用统计 | crates/ccr-usage/src/lib.rs；ccr-ui/src-tauri/src/llmusage_adapter/mod.rs | ccr-usage 拥有只读、schema-gated SQLite 投影；Tauri adapter 保留 CLI sync、事件及 DTO，不链接上游 llmusage crate。 |
| Tauri 后端 | ccr-ui/src-tauri/src/process/gateway.rs；commands/handler_registry；.cargo/tauri-ci.toml | 子进程受限执行、生命周期与类型化命令；CI 使用受控 frontendDist 夹具。 |
| React 前端 | ccr-ui/package.json；vitest.smoke.config.ts；tests/setup | React 19、类型化 IPC、jsdom smoke 与行覆盖率门禁。 |
| VS Code 扩展 | ccr-vscode/package.json；justfile；.vscodeignore | TypeScript 构建、51 项 Node 测试和 VSIX 打包。 |
| 文档与治理 | docs/package.json；scripts/ci；scripts/quality；justfile | VitePress、双语链接检查、工作流相关性、版本/依赖/安全治理。 |
| Agent 入口 | AGENTS.md；CLAUDE.md；docs/agents/harnesses.md；工具 dot 目录 | 共享规则、平台 schema、Trellis 注入与执行权限分层。 |

根 CONTEXT.md 不存在。本轮使用 code_map.md 和现有领域规范，没有新建 CONTEXT.md。

## 2. 验证摘要

完整命令、退出码、耗时和日志哈希见 [检查记录](check-results.md)。

- Rust：just version-check、fmt-check、lint-strict、check-workspace、test 通过。工作区累计 1867 passed、16 ignored、0 failed；保持默认并行。
- Tauri：just tauri-ci 通过；主要后端二进制 407 passed、1 ignored、197 export 过滤，独立 guard 2 passed；后续绑定生成检查 197 passed，额外契约测试与 inventory 检查通过。
- 前端：i18n 24/24；169 个 smoke 文件、904 项测试通过；循环依赖检查覆盖 738 文件；类型、lint 和边界负例通过。
- 前端覆盖率：Lines 75.21%（13083/17395），现有门槛 70%。未调整分母或阈值。
- 前端构建：CCR_SKIP_ICON_GENERATION=1 just frontend-build 通过；使用已提交图标，未重新生成品牌资源。存在 638.13 kB chunk 警告，未作为本轮必改项。
- 扩展：just vscode-ci 通过，51/51 测试；打包命令成功，但包内容存在 F07。
- 文档：just docs-check 通过。CI 治理、actionlint 通过。
- Rust audit：两个 lockfile 分别扫描，退出码均为 0；Tauri 仍有 9 条非阻断 warning，详见 O01。
- 当前失败：just frontend-audit；bun test scripts/trellis/omp-context.test.ts；just copilot-check。仅跟踪文件构成的 OMP 导入夹具也失败。

未整条运行原 just ci，因为该命令先执行 version-sync、fmt。组成项通过不等于完整 just ci 通过。未运行新鲜五客户端会话、hook/trust 原生校验、GUI、Linux/macOS、MSRV 1.95、Rust/Tauri coverage、VS Code coverage、release 构建或当前 HEAD 托管 CI。

## 3. 当前发现与改造责任

### F01 · P1 · 前端安全门禁报告 18 条未批准公告

证据：research/evidence/frontend-audit.json，退出 1；frontend-audit-raw.json。共 9 high、6 moderate、3 low。报告是当次 registry 审计结果。

| 锁定版本 | 公告数 | 已确认依赖路径 |
|---|---:|---|
| colord 2.9.3 | 1 | stylelint 开发依赖 |
| fast-uri 3.1.5 | 5 | stylelint → table → ajv |
| js-yaml 4.3.1 | 1 | eslint/eslintrc 和 stylelint/cosmiconfig |
| undici 8.10.0 | 11 | jsdom / Vitest 开发测试依赖 |

定位：ccr-ui/bun.lock:644、778、978、1340；ccr-ui/scripts/audit-dependencies.mjs 的 validateAuditReport 将未批准 GHSA 阻断。bun pm why 确认上述开发工具链路径。该证据没有证明发行版存在可利用攻击路径。

根因：lockfile 仍包含公告受影响版本，现有 policy 无对应例外；脚本按契约拒绝。T08 负责兼容范围内的定向升级和完整前端回归，保留 maxActiveExceptions=0。不降低级别或忽略公告。

### F02 · P1 · OMP 实际注入遗漏设计和实施文档

证据：本机 OMP 测试 3 pass、2 fail。scripts/trellis/omp-context.test.ts:210–214 检查复杂任务三类文档，首先在 Design 标记失败。

根因：.omp/extensions/trellis/index.ts:646–657 仅 materialize prd.md 和 info.md；没有读取 design.md、implement.md。与 .trellis/workflow.md:163–167 的复杂任务契约及 docs/agents/harnesses.md:31 不符。存在低成本执行 agent 收不到批准方案的风险。

另一个失败位于 scripts/trellis/omp-context.test.ts:241：测试要求旧的 ## PRD 标题，而实现输出带路径和 inline 状态的标题。这一项是展示断言漂移；不能通过放宽所有标记断言来掩盖实际漏文档。T01 分别处理内容契约和展示断言。

### F03 · P1 · 已跟踪测试依赖 Git 未交付的 OMP 源码

scripts/trellis/omp-context.test.ts:5 直接导入 ../../.omp/extensions/trellis/index.ts。git ls-files 不含该源码；.gitignore:79 忽略 .omp。

只复制 HEAD 已跟踪测试到自有临时夹具后运行 Bun，得到 Cannot find module，退出 1；该实验证明导入缺口，未冒充完整 clean clone。五工具 agents/hooks 当前均存在于本机，但相关目录整体被 .gitignore:60–81 排除。共享 docs 未充分说明本机安装和 Git 交付的差异。

T01 精确交付 OMP 测试运行源；T02 明确其他本地生成组件的状态和引导。不要取消整个工具目录的忽略或纳入 settings.local/账户/缓存。

### F04 · P1 · 五工具说明存在可执行规则冲突

- CLAUDE.md:22 和 .codex/skills/ccr-gate-recovery/SKILL.md:10、32 要求直接 cargo test 使用 --test-threads=1；justfile:397–401 和 test-fixtures.md 明确默认并行、测试内锁隔离。直接示例还遗漏 --skip export_bindings。
- docs/agents/harnesses.md:30 及英文第 30 行称 Kimi 未安装项目 agents，仅分派内置 coder；本机 .kimi-code/agents/ 已有 implement/check/research 三个定义，且其内容要求按名称分派。
- .agents/skills/trellis-meta/references/platform-files/platform-map.md:27 仍写 Kimi Agents 为 None，与本机文件及官方能力不一致。文档提到的镜像属于 ignored 生成层，不能当作团队唯一来源。
- docs/agents/harnesses.md:31 声明 OMP 三类文档自动注入，实际 F02 不成立。

已经正确的部分：CLAUDE.md:5 是真实 @AGENTS.md import；docs 区分只读 reviewer 与可写 trellis-check；docs 已说明 Grok/Kimi 官方具备自定义 agents/hooks；Codex effort 没有被定义为五工具通用参数。

T02 保留正确规则，修复实际冲突，增加基于已跟踪事实的静态一致性检查。官方支持不等于本机启用或用户授权。

### F05 · P1 · Dependabot 生态与 Bun lockfile 不匹配

最新查询到的失败是 Dependabot run 36370961840，开始于 2026-09-28T02:44:17Z，SHA 7ab923ecfb302ba041128de852d90d6a5aca8c86。日志第 131 行明确报告 npm_and_yarn 无法更新 /docs/bun.lock，要求 Bun 生态。

当前 .github/dependabot.yml:18–19 和 28–29 仍把 /ccr-ui、/docs 设为 npm；两个 package.json 均声明 bun@1.4.0。/ccr-vscode 的 npm 配置应保留。较早的 proxy 解析提示不是本次最终失败原因。T03 仅修复两处映射并增加治理测试。

### F06 · P2 · Copilot frontmatter 检查误报 CRLF 文件

just copilot-check 退出 1，9 份文件共 18 个必填字段被报告缺失。实际字段存在。

scripts/quality/check-copilot-assets.mjs:163、167 硬编码 ---\n 与 \n---\n。示例 .github/instructions/rust.instructions.md 的工作区字节有 13 个 CRLF，HEAD blob 有 0 个 CRLF；core.autocrlf=true。仅在内存中规范化后，分隔符可识别。

另有可移植性问题：脚本第 106–109 行强制存在 ignored .claude/skills。T04 使用跨平台解析和受控负例修复，避免批量改换行或削弱必填字段规则。Copilot 是本仓库额外检查对象，不扩展本轮五套 harness 的范围。

### F07 · P1 · 成功构建的 VSIX 包含本地工具配置

just vscode-ci 产物 ZIP 共 23 条目，含 extension/.serena/project.local.yml、project.yml、.gitignore，以及 extension/dist/extension.js.map、AGENTS.md、CLAUDE.md、code_map.md。本轮仅检查路径与清单，未读取本地配置内容，也未发布包。

根因：ccr-vscode/.vscodeignore 采用遗漏式排除，未排除 .serena；!dist/** 与 *.map 的组合没有阻止实际 source map 入包。ccr-vscode/justfile 的 ci 验证编译、测试、打包退出码，没有最终内容约束。

T05 建立运行文件允许范围及合成负例。当前证据确认本地文件入包，未证明文件包含密钥或已有外部泄露。

### F08 · P2 · 聚合 CI 带修复副作用且检查覆盖不完整

justfile:570–583 的 Windows 步骤以及 Linux/macOS 对应数组先运行 version-sync、fmt，可能把待审差异改写。audit 默认只审 root lockfile。当前聚合列表缺少已存在的 frontend-audit、copilot-check 和 OMP 契约测试，而托管 frontend-ci.yml:48–49 明确执行 frontend-audit。

因此既有本地“完整 CI”与托管、安全、harness 证据边界不同。T06 在现有 just 和工作流上分离修复与检查并接入遗漏项；保留 required context 名称、并行和 coverage 门槛。

## 4. 历史托管失败追踪

这些失败属于旧提交，不能用于断言当前 HEAD 同样失败。本轮未远程 rerun。

| 编号 | Run / SHA | 直接失败点 | 根因判断与当前状态 |
|---|---|---|---|
| H01 | 32684239641 / fac5611f5587dcbe842ef8c133072a26f7ef7be5 | Root Coverage 的 non_dry_run_doctor_persists_sanitized_report；report_path=None，覆盖率文件未生成 | 原测试失败先于 coverage 汇总。底层根因未查明；save_report 及 capture 分支均可能导致 None，旧日志缺少区分。当前 Windows just test 通过，未运行 Linux llvm-cov。 |
| H02 | 32684151306 / 8fb8f20ac551f2133e4e9c5190fe6f82ac9d8455 | Windows process smoke 两项：stdin/status.success 与 output flood/stdout_truncated | 定位 gateway Windows PowerShell 夹具与受限执行边界。旧日志不足以区分启动失败、超时和输出回收，底层根因未查明。当前 Windows tauri-ci 对应测试通过。 |
| H03 | 32684151228 / 同上 | Frontend coverage：EnvironmentTeardownError，RPC 关闭时 onUserConsoleLog 未完成 | 直接原因是测试环境清理阶段仍有异步日志请求；无阈值未达证据。旧版本是 Vue，当前 React 测试和 coverage 均通过；不能直接证明旧缺陷修复。 |

T07 负责正式环境复验和有界诊断。不得改为串行、增加宽泛重试或降低 70%/85% 来关闭这些记录。

## 5. 其他观察与保留项

O01：Tauri lockfile audit 退出 0，但存在 7 条 unmaintained 和 2 条 unsound warning；其中 glib 0.18.5 / RUSTSEC-2024-0429、rand 0.7.3 / RUSTSEC-2026-0097 需按实际 target/feature 依赖图复核。不能据 lockfile 存在断言 Windows 产品路径可利用。T07 保存依赖链、责任与后续项，不加 ignore。

O02：本地 Bun 1.4.2，项目声明 1.4.0；Rust 1.98.0，Node 26.7.0，just 1.58.0，cargo-audit 0.22.2。本轮未安装/升级全局工具。验证时版本差异必须可见。

O03：前端 workflow 保留 Vue and Docs Required 是现有 branch-protection 兼容契约（dependency-governance.md:359–363），不是应顺手重命名的缺陷。

## 6. 优先级与审批入口

P1：T08 依赖安全、T05 包内容、T01 OMP 注入与可复现源、T03 Dependabot、T02 共享契约。P2：T04 CRLF、T06 聚合门禁、T07 正式证据。

具体文件白名单、依赖与必须通过的命令见 [实施总计划](../implement.md) 和各子任务 design.md / implement.md。五工具分工见 [能力矩阵](harness-matrix.md)。

只读审查阶段保留失败；用户批准后才修改产品/规则。批准项最终回写 AGENTS/CLAUDE、双语 harness 页、已跟踪 ccr-gate-recovery skill 或领域规范，并注明适用工具。不向个人知识库导入本轮原始日志。
