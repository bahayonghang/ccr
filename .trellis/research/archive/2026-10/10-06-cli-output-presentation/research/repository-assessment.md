# 仓库调查与问题归属

## 基线

日期 2026-10-06，dev，HEAD `9fafc1116bc7d54004ab3b224b33c2f08cae6fcb`。初始 `git status --short --branch -uall` 为干净工作树。根 GLOSSARY.md/GLOSSARY-MAP.md 不存在，不创建缺失 glossary。

已读 code_map、crates/AGENTS.md、相关 backend indexes、CLI backend guidelines、core logging contracts、CLI diagnostics、env fixture 规则及 docs/AGENTS.md。事实采用当前源码。

## 问题与证据

| ID | 当前事实 | 影响与归属 |
| --- | --- | --- |
| F1 | `crates/ccr-core/src/core/logging.rs:31-49` 固定输出五种括号标签；success/warning/error/step 对整个 msg 加色。 | 标签重复、颜色范围过大；C1。 |
| F2 | `crates/ccr-cli/src/commands/codex/auth/save.rs:57-73` 将成功、描述、邮箱、提示标题交给状态函数，再直接打印两条固定建议。 | 字段无层次，已知账号仍用占位符；C2。 |
| F3 | `save.rs:67` 使用 service.mask_email；ColorOutput::mask_sensitive 委托既有算法。 | 保持脱敏与读取边界，不复制算法；C2。 |
| F4 | `crates/ccr-cli/src/commands/codex/auth/current.rs:154-190` 已保存、API Key、Provider Key 各有提示分支。 | 可复用现有状态，不增加认证探测；C2。 |
| F5 | `crates/ccr-cli/src/commands/codex/auth/list.rs:187-203` 计数使用 success，结尾固定建议 switch/current/delete。 | 统计采用中性表达，精简建议；C2。 |
| F6 | `crates/ccr-cli/src/commands/codex/auth/export.rs:183-199` 分散打印结果、数量、加密说明和导入建议。 | 归组展示，保留加密和密码事实；C2。 |
| F7 | `crates/ccr-cli/src/commands/claude/auth/save.rs:13-38` 有相同标签/建议；email 直接取 account.email。 | 保持平台原有数据规则，不能声称原来所有邮箱都已掩码；C2。 |
| F8 | `crates/ccr-cli/src/commands/grok/auth.rs:49-77` 先处理 JSON/取消，再打印 mutation；Codex `rename.rs:53-60` 先序列化 DTO。 | 不将取消标为成功，不污染 JSON；C2。 |
| F9 | `crates/ccr-cli/src/services/doctor_service.rs:37-44` 有独立四种标签；`commands/doctor_cmd.rs:54-75` 在 JSON 后调用 label。 | shared helper 不能自动覆盖；C3 在 command 渲染层迁移。 |
| F10 | `crates/ccr-cli/src/commands/common/table.rs:9-26` 已有表格 helper，截断符固定 `...`。 | 不重建表格；保留 columns、borders 与截断。 |
| F11 | `crates/ccr-core/src/core/logging.rs:421-429` 的测试只调用输出，无文本、层次、流断言。 | C1 补格式契约，C2/C3 补进程输出证据。 |
| F12 | ColorOutput 从 core lib 与 root facade 导出，Codex 和 Sync 也调用。 | 保留公共 API，共享入口外影响须复核。 |

## 数量与边界

全 crates 有 66 个文件包含 ColorOutput 调用；CLI 的 63 个文件合计 912 次，其中状态 784：success 138、info 422、warning 134、error 36、step 54。Auth info 127 次。统计是源码 regex，包含测试/可能注释，不代表命令数或运行覆盖率。

语义清单有 452 个匹配入口、62 个 CLI 源文件，收录 info、旧标签、提示标题和部分重复 emoji。清单只提供复核入口，实施须继续搜索 raw println 和其他状态调用。

共享入口外生产调用：`crates/ccr-codex/src/platforms/codex.rs` warning 1；`crates/ccr-sync/src/sync/content_selector.rs` title 1、warning 2、info 2、error 2。业务逻辑不在编辑范围。

## 环境与兼容性

- 根 Cli 只有既有全局 yes；JSON/verbose 位于具体命令定义，无全局输出样式配置。任务不增加配置项。
- Cargo.lock 与两个 crate Cargo.toml 使用 colored 3.1.1，CLI 已有 unicode-width；任务不增加依赖。
- 安装的 colored 3.1.1 `control.rs:91-118` 与公开 tag 一致：颜色默认根据 stdout TTY，CLICOLOR_FORCE 优先于 NO_COLOR。逐条消息不使用全局 override。
- 当前根契约与 fixture 要求默认并行；CLI backend 仍有 serial 示例。执行采用根契约，不降低门槛。
- 活动旧标签文档命中 `docs/reference/commands/clean.md`、`doctor.md`，对应英文镜像均存在。

## 范围外发现

1. Codex save 不支持存储、未登录和保存失败后返回 Ok（save.rs:32-50、75-88）；Claude save 也在失败后返回 Ok（claude/auth/save.rs:34-43）。属于退出码/错误传播问题。当前运行影响依据源码推断，未操作真实账号；展示任务保留基线退出码。
2. `logging.rs:315-325` 的控制台 tracing layer 使用 with_ansi(true)。没有证明具体机器命令受污染；不在 ColorOutput 任务内修改 logger。
3. 标题采用 chars().count 生成下划线，中文宽度可能不同；通用标题/表格重设计留在范围外。

## 验证状态

源码调查与公开资料读取已执行。没有运行产品测试、真实认证动作或终端截图。用户报告的输出无需重复操作真实账号。产品验收均 NOT_RUN。
