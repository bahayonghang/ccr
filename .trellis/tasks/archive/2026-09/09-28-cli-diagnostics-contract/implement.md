# T04 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：assert_cmd 在隔离 HOME/CCR_ROOT 执行实际 binary，断言 exit code 与分类，而非只匹配输出文字。
- [ ] 1.1 在明确 owner 内实现机制：共享诊断用例返回 typed report；终端层渲染。保留既有错误退出码映射，warning-only 为零，错误非零；不让可嵌入服务调用 process::exit。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：平台 × auth-mode × 6 类状态表驱动测试；读取错误不可进入未配置分支。
- [ ] 2.1 在明确 owner 内实现机制：调用平台 validator，拆开 Option 未配置与 Result 读取失败，删除 .ok()/None 吞错支路；应用入口保留 T02 enabled policy。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：文件清单/字节测试和 capability matrix guard；核对 help、docs 与生成入口，旧受支持行为不被静默移除。
- [ ] 3.1 在明确 owner 内实现机制：使用 T01 纯查询。能力矩阵从共享平台类型派生；Gemini/Droid legacy writer 仅盘点和标记兼容风险，禁止新增调用者或扩张支持范围，不在无可达证据时删除公共 API。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cargo test -p ccr validate
cargo test -p ccr-cli validate
cargo test -p ccr-config validator
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

共享 report 与 terminal adapter 同批回退；不回退 T01 的纯读取保护。公开退出码变化以 bugfix 记录，保持错误码数值的既有映射。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
