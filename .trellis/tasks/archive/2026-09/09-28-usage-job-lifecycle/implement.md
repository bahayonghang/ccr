# T06 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：无子任务前置；仍须用户批准规划后才可实施。
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：受控 admission/start barrier 和 fake spawner，覆盖重复 start、早期 cancel、running cancel。
- [ ] 1.1 在明确 owner 内实现机制：建立同一 registry record 持有 snapshot、token 和执行所有权，admission 原子化；runner 在 spawn 前检查 token；cancel 仅切换 cancel-requested 并发信号。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：状态转移表、乱序事件、重复终态和 cleanup failure 测试，核验 active 释放时点。
- [ ] 2.1 在明确 owner 内实现机制：在 lifecycle service 内封闭状态转换；runner 以 typed execution result 唯一提交终态，保留 cancelled/timed_out/cleanup_failed 分类。事件只投影已提交状态，前端同步完整枚举。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：可控子进程 fixture 配合单调时钟，允许调度容差并记录期限；Windows/Linux/macOS tree cleanup matrix，不用前端 Promise timeout 代替。
- [ ] 3.1 在明确 owner 内实现机制：在现有 ProcessGateway/stream owner 中统一 deadline/cancel/terminate/reap 和 bounded reader joins；测试可注入毫秒期限，生产保留现有一小时策略；清理失败优先于伪造 cancelled。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。
- [ ] 4. 固化 R4/AC4 的旧失败反例：现有 bounded stream 测试与 SQL ownership guard，加后端和 React 终态消费测试。
- [ ] 4.1 在明确 owner 内实现机制：复用现有 bounded readers、ManagedProcess 和 DTO mapper，仅修生命周期；不链接上游 llmusage Rust crate、不重建 usage parser。
- [ ] 4.2 运行行为断言并验证 AC4，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml usage
cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml process::gateway
just tauri-process-smoke
cd ccr-ui && bun run test:smoke -- tests/usage
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

lifecycle result、状态 DTO 和前端映射同批回退；升级/回退不在有活动子进程时替换 registry。保留终态诊断，禁止将 cleanup_failed 降级为成功取消。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
