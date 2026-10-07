# 实施与验收计划

## 授权门槛

- [x] 用户于 2026-10-06 批准“审阅完毕开始实施”；批准范围为父任务和三个子任务。
- [x] 启动前重读 HEAD、git status、子任务和受影响规范；保留现有改动。
- [x] 实现使用 trellis-implement，独立检查使用 trellis-check；主会话负责最终范围、规范和证据。dispatch 以当前实际 `Active task: <path>` 开头。
- [x] 父任务只有编排职责，不作为实现目标；规划阶段不修改产品源码。

## 顺序与规模

1. C1：共享状态、输出能力、建议排版接口与契约测试。
2. C2：Auth 消息及上下文建议；依赖 C1 验收。
3. C3：其余 CLI、Doctor 和活动文档；迁移依赖 C1，最终集成依赖 C2。
4. 父任务：核对 AC1–AC10、完整清单、当前全量门槛与未运行边界。

扫描范围超过 8 个文件：63 个 CLI 引用文件、额外 Doctor command/service、两个共享入口外调用和文档镜像。完整责任表见 research/scope-inventory.md。

- [x] 每个调用有修改、共享规则覆盖或保留理由。不要求编辑所有调查文件。931 基线调用与 452 基线语义入口已闭合，追加当前源码覆盖。
- [x] 不机械替换所有 println、表格或 service 日志，不改变目录及业务分层。
- [x] 实现检查完成后，由主会话新增 `.trellis/spec/ccr-cli/backend/cli-output-presentation.md` 并更新 backend index，记录最终可执行契约；规划阶段不改 live spec。

## 自动检查

使用 Cargo 默认并行度及 `--skip export_bindings`。CLI backend 中的旧 serial 命令与根契约冲突，采用根契约，不使用 --test-threads=1。

- [x] C1：`cargo test -p ccr-core core::logging -- --skip export_bindings`，11 项 PASS。
- [x] C1：`cargo test -p ccr-cli commands::common::feedback -- --skip export_bindings`，3 项 PASS。
- [x] C2/C3：`cargo test -p ccr --test commands -- output_presentation --skip export_bindings` 和各自既有命令回归。
- [x] C3：相关完整 crate 套件由正式 `cargo test --workspace --all-features -- --skip export_bindings` 覆盖；未重复 package 命令。
- [x] C3：完整 root commands 由同一 workspace/all-features Test 步覆盖，139 项通过；未重复相同套件。
- [x] 按变化运行 `just version-check`、`just fmt-check`、`just lint-strict`；验证与修复命令分开。最终当前源码各 exit 0；一次 fmt exit 1 与复测分别保留。
- [x] 文档变化：在 docs/ 依次执行 `bun run build`、`bun run audit`，各 exit 0。
- [x] 父任务最终跨 crate 门槛：`just ci`；focused PASS 不代替全量结果。

实际 subprocess 夹具捕获两个流，隔离 HOME、USERPROFILE、CCR_ROOT、CCR_LOCK_DIR、CODEX_HOME、CLAUDE_CONFIG_DIR、GROK_HOME 及适用的官方目录。覆盖或清除可绕过夹具的路径 override，尤其 CCR_CODEX_DIR、CCR_DATA_DIR、CCR_CONFIG_PATH。只使用合成账号和密钥。复用项目 env 夹具，纯格式测试不改全局 env。

## 终端检查

- [x] Windows 实际终端：40/80/120 列、明暗背景、普通/NO_COLOR/TERM=dumb，覆盖成功、警告、错误、缺字段、长字段和多行说明。
- [x] stdout 重定向、stderr 重定向、两流捕获，分别记录默认模式和 CLICOLOR_FORCE 例外。
- [x] 从终端复制建议至隔离夹具，验证命令文本，覆盖普通及前导连字符名称；不执行真实账号覆盖/删除。
- [x] JSON 可解析，DTO 与值保持；比较失败、取消和部分成功退出码。
- [x] 回执保存在子任务 checks/；截图或实际终端文本不能被编译结果代替。
- [x] Linux、macOS、hosted CI 无条件时标 NOT_RUN；真实账号验证不属于展示任务。

## 收尾与异常

- [x] 调用清单和语义清单每项有处置，重新扫描当前源码防止遗漏。当前 1091 引用与 961 语义入口另存；72 个源码 hash 一致。
- [x] 保留第一失败回执；修正后复测单独记录。既有退出码/logger 问题只报告。
- [x] 对批准范围执行 git diff --check，确认无凭据、环境文件、生成绑定或用户数据。已生成 Doctor 失败回执做敏感值布尔审计，结果为 false。
- [x] AC1–AC10 分别给出 PASS、FAIL、NOT_RUN 或 UNVERIFIED。必需验收全部满足后才报告实现完成。结果见 checks/verification.md；当前未报告实现完成。
- [x] 回退仅恢复批准文件；不 reset、clean、stash 或覆盖用户工作树。


2026-10-07 当前复测：路径独立安全 13+5+2、展示 binary 22、既有回归 8/10/14、原生 handler 18×9=162、屏幕复制 36、混合流 12 均通过。完整 workspace Test 已通过；完整 just ci 16/16通过。旧暂停状态仅属此前证据，见 checks/2026-10-07-pre-path-verification.md。父 AC7 保持 UNVERIFIED。

当前状态：展示、路径安全、binary 与原生矩阵已通过；完整 workspace Test 2101 passed，完整 just ci 16/16通过。父 AC7 的 Unknown/完整 import 交互边界未关闭。父与三个子任务保持 in_progress；提交、归档未授权。

最终门槛已通过：just ci 16/16、exit0、20:36.985；sourcemanifest82与HEAD无变化。父AC7的Unknown/密码交互运行边界仍UNVERIFIED，交付与归档未授权。
