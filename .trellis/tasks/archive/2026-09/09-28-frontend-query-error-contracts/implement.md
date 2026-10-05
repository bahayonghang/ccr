# T09 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [x] 核对前置任务：T03 / 09-28-tauri-config-adapter；T08 / 09-28-settings-lossless-capabilities
- [x] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [x] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [x] 1. 固化 R1/AC1 的旧失败反例：真实 BaseAuth 配合 mock domain，覆盖初次两类失败、旧缓存刷新失败、重试、unsupported、off 拒绝和重复点击。
- [x] 1.1 在明确 owner 内实现机制：在现有 BaseAuth 实现明确 view model；先处理 probe error，再处理 load error，旧成功数据标记 stale。off 使用受控 mutation pending 与 catch 反馈；不强制迁移 Claude/Codex 特殊认证页。
- [x] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [x] 2. 固化 R2/AC2 的旧失败反例：控制 Query promises 先后顺序，断言输入值、baseline、禁用提交和确认行为；保存失败保留草稿，成功重建基线。
- [x] 2.1 在明确 owner 内实现机制：依赖 T08 typed snapshot，引入编辑基线版本与 draft dirty set；query key 含环境身份。dirty 时收到新服务端版本显示冲突/刷新提示。环境变化冻结旧会话写入并保留其草稿，提供返回原环境或明确放弃后重载的动作；敏感文本不进入持久 storage。
- [x] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [x] 3. 固化 R3/AC3 的旧失败反例：同一页面实例语言切换行为测试；禁止通过 unmount/remount 满足断言，保留 layering/cycles checks。
- [x] 3.1 在明确 owner 内实现机制：使用既有 useAppT/useResolvedT 并让 memo 依赖正确的翻译身份；只修本次配置表面，不清理全仓文案或重做视觉。
- [x] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cd ccr-ui && bun run test:smoke -- tests/platforms tests/configs tests/shell
cd ccr-ui && bun run type-check
cd ccr-ui && bun run check:cycles
cd ccr-ui && bun run check:arch-boundaries
git diff --check
```

## 交付与集成

- [x] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

按 domain hook 与其消费者成组回退，query key 变更需清除本次作用域旧缓存。首页 Insights 的既有任务优先协调，禁止同时修改同一 hook/事件矩阵。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。


## 2026-09-28 R2 implementation clarification

Independent source review found that Claude typed settings reads the active ExecutionEnvironment repeatedly during one asynchronous update. Frontend identity checks cannot prevent the check-to-IPC race. The approved R2 requires a backend environment boundary for the affected read/update. This is necessary completion of R2, not a new platform.

- `claude_get_settings` and `claude_update_settings` accept optional `expected_environment_id`. The Settings page always sends the acknowledged session id. A mismatch at entry rejects before file access. A single captured environment Arc owns the rest of the read or update. Existing callers without the argument retain entry-time environment selection. Local updates retain the current atomic SettingsManager owner.
- Capture once in the shared Claude update helper so an environment switch during read cannot redirect the later write. Test the mismatch and the switch-during-read with two controlled environments and a barrier.
- Codex/OpenCode typed Settings use fixed local paths in the current backend. Their descriptors must expose existing local-only capability and a clear unsupported state in remote environments; no remote support is added.
- Frontend owner: Settings context, descriptors, domain wrappers, editor/Query tests and locale. Backend owner: Claude commands/helper, scoped registry schemas/generated files, Rust regressions. Independent check covers both.
- Include actual IPC payload tests. Do not infer backend safety from mocked frontend success or from identity verification before and after an unbound write.

## 2026-09-28 前端实施交接

- 源码和测试已冻结；实施及逐项验证见 `implementation-report.md`，当前文件 hash 见 `research/implementation-files.json`。
- 前置 T03/T08 行为保留并纳入最终 66 文件、277/277 前端回归；任务激活由 root 完成，本 owner 未改任务状态。
- type-check、cycles、架构边界、i18n、style、生产 build 通过；正式 lint 仍因两份原有受保护临时文件的 5 个错误失败。
- 独立检查、父任务 ledger、T10 集成及原生验证保持未勾选；相关结论由 root/checker 按实际证据更新。
