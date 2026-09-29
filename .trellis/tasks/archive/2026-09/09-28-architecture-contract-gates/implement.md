# T10 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases；T03 / 09-28-tauri-config-adapter；T04 / 09-28-cli-diagnostics-contract；T05 / 09-28-safe-persistence-backups；T06 / 09-28-usage-job-lifecycle；T07 / 09-28-command-workbench-lifecycle；T08 / 09-28-settings-lossless-capabilities；T09 / 09-28-frontend-query-error-contracts；T11 / 09-28-desktop-control-oauth-lifecycle
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：recipe/workflow contract tests、故障 fixture 与最终 just ci/just tauri-ci 组合；仅在授权实施阶段、隔离工作树执行有改写的步骤。
- [ ] 1.1 在明确 owner 内实现机制：复用 just tauri-ci 并与 root aggregate 对齐，分清只读 check 与 format/regenerate。保留 hosted required lane 和跨平台 tree-cleanup matrix；不降低现有 coverage 阈值。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：审阅实际测试断言并运行聚合 contract suite；每个标记通过的目标至少执行一个匹配测试。
- [ ] 2.1 在明确 owner 内实现机制：复用 T01-T09 及 T11 fixture，父任务维护 operation × platform × entry 的验证矩阵；行为测试与静态边界检查互补。Rust/原生未执行必须保持未通过，禁止将检查子集改称完整 gate。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：文档路径检查、inventory/bindings drift、治理脚本，人工核对历史示例与当前契约的区分。
- [ ] 3.1 在明确 owner 内实现机制：以 registry/manifest 和 code map 为证据更新受影响规范，不批量替换历史案例；每个子任务随实现同步自己的契约，T10 只完成跨域收敛。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。
- [ ] 4. 固化 R4/AC4 的旧失败反例：基线/最终 git status 对照、门禁原始日志、任务状态和 requirement coverage 检查。
- [ ] 4.1 在明确 owner 内实现机制：最终在 clean worktree 或明确保留用户未提交内容的工作区运行正式 gate；不新增忽略规则绕过真实失败、不移动用户文件。归档只在各任务真实完成且符合用户授权后进行。
- [ ] 4.2 运行行为断言并验证 AC4，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
just version-check
just fmt-check
just lint-strict
just test
just frontend-check
just tauri-ci
just ci
git diff --check
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

门禁变化先以 fixture 证明仍 fail-closed；回退只撤销本任务规则，不降低既有检查。报告保留失败和未验证记录，不以修改基线定义完成验收。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
