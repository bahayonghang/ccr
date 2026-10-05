# T05 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：无子任务前置；仍须用户批准规划后才可实施。
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：注入固定时钟与不同内容；碰撞重试、备份失败、轮换和同秒多进程测试。
- [ ] 1.1 在明确 owner 内实现机制：备份名保留既有时间/前缀并增加唯一后缀；通过 create-new 语义保证不覆盖。排序采用稳定 tie-break；复合 off 备份目录采用同样操作身份。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：旧命名 fixture、恢复后内容、Windows DACL 与 Unix mode 的平台测试。
- [ ] 2.1 在明确 owner 内实现机制：复用统一 backup parser/selector 或现有兼容规则；明确新的唯一后缀 contract，不通过批量重命名用户备份迁移。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：以合成 verifier/state sentinel 分别覆盖创建、替换、取消、过期清理、权限拒绝；比较目标目录和配置的备份目录前后清单并扫描所有非当前目标文件，断言无新增凭据副本。fake permission adapter 注入拒绝，断言首次写入无新目标、替换失败旧字节不变；原生 Windows/Unix 分别验 ACL，不以源码检查替代。
- [ ] 3.1 在明确 owner 内实现机制：将普通 AtomicWriter 后 chmod/icacls 的顺序迁入现有 secret/guarded writer；权限在临时文件发布前建立。CLI/Tauri 复用 pending 存储/清理 owner，桌面仅负责 listener 和 UI 事件。 OAuth pending 显式使用 secret:true 和 BackupPolicy::None，禁止备份 code_verifier/state，旧权限失败不得吞没。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。
- [ ] 4. 固化 R4/AC4 的旧失败反例：捕获 tracing、IPC 和 CLI 错误进行 sentinel 断言；不使用真实账户数据。
- [ ] 4.1 在明确 owner 内实现机制：保持 Secret 类型与 redacted DTO，清理错误只携带分类和非敏感路径；扩展相关持久化守卫覆盖实际 owner。
- [ ] 4.2 运行行为断言并验证 AC4，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cargo test -p ccr-core guarded_write
cargo test -p ccr-cli profile_off
cargo test -p ccr-codex oauth
cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::codex::auth::tests -- --test-threads=1
python scripts/quality/check_secret_writes.py
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

新备份保留旧前缀和可识别后缀，回退前验证旧 reader 的恢复路径；任何回退均保留已生成备份。OAuth writer 与调用者同批发布，禁止恢复权限错误吞没。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
