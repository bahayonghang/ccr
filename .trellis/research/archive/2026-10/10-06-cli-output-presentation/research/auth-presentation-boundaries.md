# Auth 展示任务的 domain 边界

当前依据为 `.trellis/spec/ccr-codex/backend/backend-guidelines.md` 的 Scope、Filesystem And Security、Error Handling、Logging、Testing（第 5–97 行）及当前 CLI/Auth 专用规范。

该指南为 49,036 字节，超过 native context 单文件 32,768 字节上限。上下文使用较小的 backend/index.md 与 test-fixtures.md，并提供本摘要。执行前直接读取指南相关章节，不依赖自动截断内容；不修改注入上限或 live spec。

- Codex/OpenCode 的 auth、runtime、quota 等归 ccr-codex；CLI 调用现有服务，不直接读取真实 home 凭据来补充提示。
- 使用既有 CodexPaths 和 override。夹具必须控制 CCR_CODEX_DIR、CCR_DATA_DIR、CCR_LOCK_DIR，并隔离官方 runtime 目录。
- 保持账号快照、auth 文件的脱敏、权限、备份及修复/同步规则；展示层不改变写入或读入次数。
- OAuth pending store、desktop listener/controller 不在范围；不将 verifier、state、URL、密码或 Token 放入消息、建议或回执。
- 错误类型及 Result 传播保持当前接口，展示不修改冻结的 CcrError，不新增错误分类。
- 诊断日志用 tracing；access/refresh Token、provider keys、OAuth payload 和 raw auth JSON 不得输出。
- env 测试复用 TestCodexEnv；root subprocess 通过 Command.env 隔离父环境，不改变默认测试并行。

本摘要是任务范围说明，不替代 live spec。若实现需要修改 auth 业务、export/import 协议、runtime/snapshot sync 或存储，已经超出授权展示范围，应停止并报告，不沿用该摘要扩大修改。
