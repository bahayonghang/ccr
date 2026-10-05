# T11 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：T05 / 09-28-safe-persistence-backups；T06 / 09-28-usage-job-lifecycle
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：执行矩阵 C01-C04，分别阻塞 command/install 的共享执行路径、usage 同模块同步导入、OAuth 同模块 token exchange，逐个控制 ID 验证在 barrier 释放前到达 owner；get/status/recent/port probe 逐个保留行为测试。C05 对四类后台 start 验证 handler 返回后 admission 仍占用、第二 start 无并行、终态与清理后释放；C06 验证辅助窗口 ACL、用户确认、错误 ID 和外部进程权限。用 owner acknowledgement 与虚拟时钟/有界看门狗断言，不靠 sleep 或只检查 manifest。
- [ ] 1.1 在明确 owner 内实现机制：registry 按 control-command-matrix.md 的真实命令 ID 显式区分风险、资源和操作类别；control 不继承执行任务的全局或模块长 permit，直接交给按 job/login/attempt ID 校验的 owner，以短临界区或消息控制。start 的 admission 在发布 ID 前取得并转移给真实 job owner，持到完成与清理；foreground permit 继续绑定实际完成。保留 completion-aware future、ACL/confirmation 与现有幂等规则，不把所有命令改为 Parallel。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：fake listener/storage 及本地 loopback 隔离 fixture；覆盖 bind race、磁盘失败、state mismatch 和过期恢复。
- [ ] 2.1 在明确 owner 内实现机制：将 probe-close-rebind 改为持有 listener，先验证/保存再发布内存状态；service 注入 clock/storage/http/listener，Tauri 仅做 DTO/events。使用 T05 secret pending writer 和既有端点白名单。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：可控 socket/server 与时钟 fixture，不调用真实 OAuth；后续原生 smoke 单独证明 OS 资源释放。
- [ ] 3.1 在明确 owner 内实现机制：同一 login controller 持 cancel handle，socket read、request/body 共享有界执行机制；取消等待受控清理完成后终态；typed内部结果区分取消、超时和失败，映射现有 IPC，不全量改写 String error macro。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。
- [ ] 4. 固化 R4/AC4 的旧失败反例：manifest/bindings、旧payload、ACL 与 sentinel tests。
- [ ] 4.1 在明确 owner 内实现机制：只为变动的 profile/usage/auth操作使用 named result DTO；自由配置保留 OpenJson，旧未知消费者采用兼容 envelope，公共 CcrError freeze 保留。
- [ ] 4.2 运行行为断言并验证 AC4，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml runtime_policy
cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml codex_auth
just tauri-command-inventory-check
just tauri-bindings-check
just tauri-ci
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

registry override 与 job resource admission 同批回退，禁止只撤销 admission；OAuth storage与controller使用同一兼容pending格式，活动登录先结束再切换实现。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
