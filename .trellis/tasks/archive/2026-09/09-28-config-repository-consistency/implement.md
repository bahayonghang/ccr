# T01 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：无子任务前置；仍须用户批准规划后才可实施。
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：双进程 barrier fixture 覆盖 platform/service 与 desktop/service 组合，重复运行并断言磁盘最终值；不得靠测试串行化通过。
- [ ] 1.1 在明确 owner 内实现机制：在 ccr-config 提供按规范化平台配置路径定位的 mutation 入口，锁在读取之前获取；接收变更闭包或有类型 patch，不接受先前读出的整份 sections。规定 operation lock → resource lock → guarded leaf lock 顺序，避免重复获取非重入锁。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：空目录、缺文件、损坏文件、只读目录和不同 registry 顺序 fixture；比较前后文件路径、字节和修改时间。
- [ ] 2.1 在明确 owner 内实现机制：分离 open/read 与 ensure_initialized/reconcile。保留现有各平台 current 优先级，用纯 resolver 返回值、冲突诊断和 repair suggestion；修改只在显式 reconcile 用例中执行。未指定平台的旧 adapter 不可猜测首个 enabled 平台。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：CRUD table tests、CAS stale token、unknown field 和 secret sentinel；回归原 current/default 行为。
- [ ] 3.1 在明确 owner 内实现机制：领域 patch 显式区分保留、设置、删除；在锁内检查存在性、重名、字段类型及平台 auth-mode 校验。仍使用既有 guarded secret writer，不改变文件格式。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cargo test -p ccr-config
cargo test -p ccr-cli profile
just fmt-check
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

不迁移磁盘 schema；每个调用者迁移前后使用同一 fixture 对比。回退统一 API 和调用者为同一批次，保留旧文件读取与备份恢复能力。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
