# T07 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：T11 / 09-28-desktop-control-oauth-lifecycle
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：MemoryRouter mount/unmount/remount 行为测试，运行中回页与离页期间完成两类场景。
- [ ] 1.1 在明确 owner 内实现机制：使用现有 shell event bridge 与 stream store；发起后保存 job ID，页面挂载按 ID 调现有 status API 对账。事件带 seq 的增量幂等合并；不新增第二个页面私有任务真相源。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：保留重复事件、序号间隙、snapshot/事件倒序及过期 job 测试，并增加三个受控时序：①延迟 start Promise，先发送同 job 的终态，再 resolve 旧 queued snapshot，终态不回退；②新 job 生效后发送旧 job 的 event 和 start response，新 job 不变；③start Promise 未 resolve 时连续点击两次，仅调用一次 start。每项均断言界面状态、history 写入次数和启动/取消控件可用性，旧终态不重复记录。
- [ ] 2.1 在明确 owner 内实现机制：以 job ID 和终态版本去重，由同一 owner 记历史；snapshot 是后端权威，事件为增量加速。后端过期返回明确已过期，不伪造运行。 start 响应、cancel 响应和事件统一走 reducer，terminal 为同 job 的单调终点；提交 pending 独立于 job snapshot。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：deferred Promise 控制 listen 完成顺序，断言订阅/反订阅次数和卸载后无 state update。
- [ ] 3.1 在明确 owner 内实现机制：复用 eventBridge 已有 disposed guard，页面不再独立订阅全局 job stream；需要局部监听时采用相同生命周期 helper。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cd ccr-ui && bun run test:smoke -- tests/commands tests/shell
cd ccr-ui && bun run type-check
cd ccr-ui && bun run lint:ci
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

只替换命令页状态 owner，保留后端 command job API 和持久 history schema；store 和页面同批回退。进程重启后的任务恢复不在本任务范围。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。

## 2026-09-28 实施交接

- 产品/测试源码已冻结；实现结果见 implementation-report.md，检查原始输出和源码指纹见 research/implementation-evidence.json。
- 最终 commands/shell/API 回归 19 文件、98 项通过；type-check、生产构建、i18n、边界、循环、style 和相关 diff 检查通过。
- 正式 lint 仍为原有两个受保护临时脚本的 5 条 no-console；没有修改、删除或忽略这些文件。
- listener rejection 和 history persistence rejection 两项早期独立检查反馈已落实并补真实页面回归；最终独立检查由主代理协调 check_t07。
- 任务生命周期和父 ledger 由主代理更新；本实现者未提交、归档或修改原 Insights 任务状态。
