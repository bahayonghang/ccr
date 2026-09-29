# T03 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：React 用户动作测试加真实 Rust handler/service fixture；禁止仅 mock switch 为成功来证明链路修复。
- [ ] 1.1 在明确 owner 内实现机制：typed domain 请求携带明确平台，handler 只校验/映射到 T02 用例。启用持久策略与激活动作分别定义，由用例完成必要组合，避免 enableConfig 仅重命名 switch 调用。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：handler 参数反序列化、领域校验及持久化结果联合测试，复用 T01 并发 harness。
- [ ] 2.1 在明确 owner 内实现机制：移除 handler 自行持 config 锁的 RMW，采用共享 resource mutation；patch DTO 区分未提供与清空，禁止将非字符串默默转为 None。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：旧/新 payload contract、辅助窗口 ACL、确认元数据、manifest count 和 bindings drift 测试。
- [ ] 3.1 在明确 owner 内实现机制：在 registry 定义新输入并同步 generated client、domain wrapper 和调用者；命令 ID 保留兼容 adapter，禁止恢复旧全局隐式 switch。只迁移受影响命令，不扩张为全部 71 个 legacy 命令重写。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::config
just tauri-command-inventory-check
just tauri-bindings-check
cd ccr-ui && bun run type-check
cd ccr-ui && bun run test:smoke -- tests/configs tests/api
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

handler、registry、生成物和消费者同批回退；旧无平台调用继续明确拒绝，禁止回退到任意平台猜测。T01/T02 的领域修复独立保留。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
