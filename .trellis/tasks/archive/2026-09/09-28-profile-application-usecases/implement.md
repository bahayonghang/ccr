# T02 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：T01 / 09-28-config-repository-consistency；T05 / 09-28-safe-persistence-backups
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：在 runtime、profiles、registry 每个写点注入故障；比对旧状态字节、版本和指针；加入并发外部改动使 rollback 必须拒绝覆盖的测试。
- [ ] 1.1 在明确 owner 内实现机制：复用 ccr-cli::application 建立 prepare → execute → outcome 用例，平台 operation lock 覆盖整个复合流程。off 的必要清理规则纳入同一计划，禁止 TUI 先提交独立 off。跨文件失败以版本保护回滚；不能安全回滚时返回 partial/recovery 信息，禁止覆盖外部新版本。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：共用 contract harness 调 CLI application、TUI backend adapter、Tauri service adapter；成功、重复提交、计数写失败、history DB 不可用逐项断言。
- [ ] 2.1 在明确 owner 内实现机制：保留现有 ConfigService 禁用项不能激活的规则。runtime/指针确认后才提交成功副作用；用操作标识及可重放记录避免对已提交结果的重试重复计数。返回不含终端输出的结构化 outcome。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：结果序列化和三端呈现测试覆盖成功、附属失败、部分提交；secret sentinel 不进入日志和 DTO。
- [ ] 3.1 在明确 owner 内实现机制：区分 unchanged、applied、applied_with_warning、recovery_required 等结果语义，最终命名与既有 error freeze 保持兼容；不新增 CcrError 公共变体作为捷径。适配器负责 CLI 文本/退出策略、TUI 消息和 IPC DTO。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。
- [ ] 4. 固化 R4/AC4 的旧失败反例：依赖/调用边界 guard 加行为适配测试，保留 CLI help、公开路径和生成 IPC 漂移检查。
- [ ] 4.1 在明确 owner 内实现机制：先在现有 application 模块形成可测试接口，再迁移调用者；不在本任务新建全能 ccr-app crate。CLI command 只解析、调用、呈现，平台低层 apply 限定为内部机制。
- [ ] 4.2 运行行为断言并验证 AC4，保留兼容成功路径。
- [ ] 5. 固化 R5/AC5 的旧失败反例：三阶段 fault-injection 加 current/default、旧名/新名存在性和 unknown fields fixture。
- [ ] 5.1 在明确 owner 内实现机制：将 Claude/Codex handler 中的 load/patch/save/delete/apply 编排迁入同一 application lifecycle；沿用 T01 mutation、操作锁和版本保护补偿；不对自由 JSON 内容强制封闭 schema。
- [ ] 5.2 运行行为断言并验证 AC5，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cargo test -p ccr-cli profile
cargo test -p ccr-tui apply
cargo test -p ccr-codex profile
cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml profile
just lint-strict
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

按一个平台的三端适配器为迁移批次，不能长期混用旧 off+apply 与新事务；以受保护前镜像和 operation record 恢复，禁止无条件覆盖用户的新文件。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
