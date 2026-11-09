# 本地提交与部分归档预检

审查日期：2026-09-30。审查者：`/root/independent_check`。活动任务：`09-29-evergreen-harness-audit`。审查基线仍为 `fe9d65972376dbb4c332fde9860c1e42ff2f2005`。机器证据见 [delivery-preflight-review.json](delivery-preflight-review.json)。

## 结论

当前 32 个交付路径与 `final-delivery-validation.json` 完全一致；32 份当前文件的 SHA-256 也与 `final-ci-retry1-source-after.json` 一致。两个待批补丁均未应用。可以保留当前批准差异形成阶段性本地提交，提交与归档状态必须分开记录。

T02、T04 的本地验收已满足。T01 在提交后完成下述三文件源码夹具验证，即可与 T02、T04 一起归档。T03、T05、T06、T07、T08 和父任务继续保持未完成。T08 的实施清单仍显式保留最终完整 CI，最终 LF 输入也没有完整前端执行回执；本次不取消该保留项。

主会话传达的用户请求“请提交并归档”授权已批准工作成果的本地提交和已完成子任务归档。该请求没有批准两个新增修复范围，也没有授权 push。归档动作不能替代未完成验收。

## Findings (fixed)

无产品或任务状态修复。本轮只新增本报告和对应 JSON。未运行产品测试、安装依赖、修改源码、暂存、提交、归档或推送。

## Findings (not fixed)

### P1：T01 的已提交源码交付边界待验证

`T01/research/implementation-results.md` 与 `evidence/verification.json` 明确只验证了未提交交付候选：源选择命令包含 `git ls-files --cached --others --exclude-standard`。既有 14/14 测试和独立构建通过；这些结果没有证明新提交已经包含 OMP runtime。当前 `.omp/extensions/trellis/index.ts` 仍为未跟踪交付候选。

最小补证在本地提交后进行，不需要新产品修复或安装 Trellis。主会话负责执行、保存新提交 SHA 与日志，再关闭 T01-AC3 的交付条件。

### P1：T08 的最终验收仍有显式未完成项

T08-AC1、AC2 的最后一次本地安全证据通过。AC3 的完整类型、lint、904 项 smoke、构建及覆盖率检查使用 CRLF 锁 SHA `f4d25c2659543ffbc3c439a92813f0d67d9c44c2bc3d881d7e3710bb52adbf1e`。最终 LF 锁 SHA 为 `c4301a8d1909ac7db2f92930e5d91172195d3ad51c84b4e163c16909a2528025`，仅冻结安装、audit 和 diff 检查有实际通过回执。

本轮重新验证两份锁只有换行差异。该证明不构成最终 LF 输入的完整前端重新执行记录。`T08/implement.md` 的追加实施部分仍保留 `[ ] 主会话独立复核与最终完整 CI`；[最终 UI 审查](new-advisories-final-ui-review.md)也保留同一验证边界。独立审查已经完成，完整聚合仍失败。按现有验收工件，T08 继续 `in_progress`，不得直接勾选并归档。

该判断保留 T08 自己声明的验收条件。扩展新公告继续归属 T05，Rust manager 失败继续归属 T06/T07。

### P1：其余未完成项不能随部分归档关闭

- T03：真实 Dependabot 运行仍未验证，T03-AC3 未完成。
- T05：原批准两节点补丁曾通过，但最后扩展审计失败；两个新的兼容目标仍待范围批准。
- T06：真实完整 `just ci` 两次失败；stub 成功与 2509 个源码哈希不变不能替代完整执行成功。
- T07：macOS 进程、新客户端 hook/trust、最终提交托管结果仍缺失；manager 隔离缺口待批准修复；九条 Tauri warning 保留。
- 父任务：AC4 未完成，父任务不能标记 completed 或归档。

### P2：当前活动状态文字需要追加本次授权及部分归档结果

父 PRD、implement、task.json 与子任务记录仍包含“提交、归档未授权”“全部保持 in_progress”等之前的当前状态文字。主会话应追加本次本地提交/归档授权，并更新活动状态及实际提交 SHA。带日期的历史审查、原始失败日志、候选 patch 和校验和凭据保留原字节。

归档完成后，活动父任务应明确列出已归档子任务和仍在进行的子任务。按 `task.py archive` 实际返回的新路径修正活动 Markdown 引用、JSONL 和任务关联，再验证上下文。不得用“全部完成”或“完整 CI PASS”描述本次部分交付。

## 子任务逐项验收与归档判定

| 任务 | PRD 验收项 | 当前证据 | 归档判定 |
| --- | --- | --- | --- |
| T01 | AC1 | 角色公开文档、JSONL 隔离及缓存刷新通过 | 提交后补三文件夹具 |
| T01 | AC2 | 路径、符号链接、trustedRoots、UTF-8、预算和缺文件负例通过 | 同上 |
| T01 | AC3 | 未提交交付候选 14/14、构建通过；已提交来源待验证 | 条件满足后可归档 |
| T02 | AC1 | 五工具和 Kimi 规则、生成步骤及原生未验证边界已回写 | 可归档；在 T01 提交夹具后处理依赖顺序 |
| T02 | AC2 | 并行、绑定、授权及修复副作用规则一致 | 同上 |
| T02 | AC3 | HC005 自修后 21/21，所需负例齐全 | 同上 |
| T02 | AC4 | 回写限于批准共享文档和已跟踪 skill | 同上 |
| T03 | AC1、AC2 | Bun/npm 映射、治理、冻结安装和文档的本地检查通过 | 保留进行中 |
| T03 | AC3 | 真实 Dependabot 未验证 | 不归档 |
| T04 | AC1 | LF/CRLF、BOM 等价解析和字节保持通过 | 可在本地提交后归档 |
| T04 | AC2 | 缺字段、空值、格式、未闭合及术语负例通过 | 同上 |
| T04 | AC3 | core.autocrlf=true 本机及 18 文件临时 Git 夹具通过，无 .claude 依赖 | 同上 |
| T05 | AC1–AC3 | VSIX 允许范围、合成负例、51 项既有测试和包检查通过 | 保留进行中 |
| T05 | AC4 | 原补丁验证通过；最后 audit 失败，新范围待批准 | 不归档 |
| T06 | AC1 | 两次真实聚合的源码字节保持 | 保留进行中 |
| T06 | AC2、AC3 | 三 OS stub 顺序/传播、路径相关性及 required contexts 检查通过 | 同上 |
| T06 | AC4 | 完整聚合 FAIL；额外正式检查独立记录 | 不归档 |
| T07 | AC1 | 旧 SHA、步骤及根因未知状态可追溯 | 保留进行中 |
| T07 | AC2 | 已记录完整覆盖率门槛通过，阈值未变 | 同上 |
| T07 | AC3 | macOS、新客户端等必需环境证据缺失 | 不归档 |
| T07 | AC4 | 告警和失败保留，manager 新缺口仍未关闭 | 不归档 |
| T08 | AC1 | 最终 LF 时点 audit 0；例外 0/0；新增 UI 公告已处理 | 保留进行中 |
| T08 | AC2 | 七个累计节点、版本、范围、依赖路径和完整性记录齐全 | 同上 |
| T08 | AC3 | 完整前端/覆盖率在 CRLF 输入通过；最终 LF 实跑边界及最终 CI 保留 | 不归档 |

T04 的 PRD 要求本机与仅跟踪源文件夹具，既有临时 Git 索引夹具已满足该条。报告不能把该夹具扩写为已提交完整 clone 或原生客户端通过。T01 的原发现直接涉及未交付 runtime，故在本地提交后补提交来源证明。

## 父级验收

| 验收 | 当前判定 |
| --- | --- |
| AC1 | 审查、原失败、当前复验分别记录；通过证据可追溯 |
| AC2 | 五工具静态规则、权限和知识回写通过；原生加载仍独立 |
| AC3 | 八个子任务的 PRD、设计、实施及上下文齐全 |
| AC4 | 未完成；安全和正式检查阻断保留 |
| AC5 | 当前提交/归档授权需由主会话追加记录；新补丁及外部操作未获授权 |

## T01：提交后的最小夹具

最小提交源文件精确为：

1. `.gitignore`
2. `.omp/extensions/trellis/index.ts`
3. `scripts/trellis/omp-context.test.ts`

在新提交产生后固定 `<delivery-commit>`。先运行以下 Git 命令，并断言清单恰为上述 3 个文件：

```text
git ls-tree -r --name-only <delivery-commit> -- .gitignore .omp/extensions/trellis/index.ts scripts/trellis/omp-context.test.ts
git archive --format=tar <delivery-commit> -- .gitignore .omp/extensions/trellis/index.ts scripts/trellis/omp-context.test.ts
```

将 archive 的原始字节提取到主会话拥有的空临时目录。可以用 Python `subprocess.check_output`、`io.BytesIO`、`tarfile` 和 `TemporaryDirectory`，避免 PowerShell 文本重定向改变 tar 字节。提取前校验成员清单；提取后记录三个文件 SHA-256。不得从当前工作区、ignored 文件、node_modules 或个人 Trellis 安装补齐文件。

在该目录执行：

```text
bun test scripts/trellis/omp-context.test.ts
bun build .omp/extensions/trellis/index.ts --target=bun --outfile <fixture>/extension.js
```

通过条件：14 passed、0 failed、0 skipped；独立构建退出 0；运行时产物不依赖类型专用的 `@oh-my-pi/pi-coding-agent` import。记录新提交 SHA、实际 Bun 版本、源清单、命令、退出码、输出和日志哈希。已有 verification.json 与首次失败凭据保持原样，补建提交后记录。原生 OMP 加载与 API 类型检查继续属于未验证边界。

## 待批补丁与当前文件核验

- manager 两个文件与 HEAD 无差异；SHA-256 分别为 `general.rs = 7ce7164c69ca883cf612ea440c51f2b40207ac4220694c68de42c3ab6a6ca4b2`、`legacy_registry.rs = 685cb2972188d005e7f314ecd38e14515b76b3f5f10bebffa1f94b1122163d67`。候选六行未应用。
- 扩展锁 SHA 为 `846b84a475e48c9e7f335a87121a1894dc64548f41449d2dd54a5b932174fdd4`；brace-expansion 仍 5.0.9、fast-uri 仍 3.1.7、undici 为已批准的 7.29.1。两个新的目标仍未应用。
- 32 路径清单没有新增或遗漏，并全部匹配上次聚合后的当前源码字节。任务证据与 journal 属于独立交付记录，不计入产品路径数量。
- `context-governance.md` 引用的规范 SHA 与当前规范一致，没有规范快照漂移。

## Verification

- Lint：本轮未重跑。已核对最终集成记录；actionlint、Python compile、静态契约门槛通过记录保留。
- TypeCheck：本轮未重跑；T08 完整前端类型检查属于记录的 CRLF 输入。
- Tests：本轮运行 0 次；T01/T04 的 19 份既有日志及最终集成 7 份日志哈希核验通过。T02 使用自修后的 21/21 记录。
- 源码：32/32 当前交付哈希匹配；T01 的 3 个、T04 的 4 个局部源哈希与各自验收凭据一致；最终集成 10 个输入哈希与当前文件一致。
- 两次实际完整 CI 的失败凭据保留；本报告没有将局部通过换成完整验收通过。

本轮建议的最终状态为：补证后归档 T01/T02/T04；父任务和 T03/T05/T06/T07/T08 保留进行中。新的验收证据可以改变后续判定，提交或移动目录本身不能改变判定。
