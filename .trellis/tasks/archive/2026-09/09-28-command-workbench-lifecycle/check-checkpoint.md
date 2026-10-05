# T07 早期独立检查记录

- 日期：2026-09-28。
- 阶段：实现者尚未冻结；本轮只读检查产品、测试和规范。
- 范围：真实 CommandsView/MemoryRouter、command stream store/reducer、shell event bridge、现有 typed start/status/cancel 路径。
- 本轮仅写入本文件和 `research/check-checkpoint-evidence.json`。未运行构建、测试、lint、type-check 或生成命令；未修改产品、测试、规范、任务状态和保护文件。
- 使用技能：`.agents/skills/trellis-check/SKILL.md`。已读取保存的 hook 输出、任务 PRD/design/implement、check context、根/UI code map 和适用规范。

## Findings (fixed)

无。本轮为主代理指定的早期只读检查。

## Findings (not fixed)

### CP-T07-01 — P2 — 命令监听注册失败没有可见的对账恢复入口

- 证据等级：源码控制流确认；未执行故障注入测试。
- 文件：`ccr-ui/src/shell/eventBridge.ts:124-130`、`:156-170`；`ccr-ui/src/features/commands/commandJobStore.ts:104-118`、`:156-178`；`ccr-ui/src/features/commands/CommandsLedger.tsx:8-15`。
- 可控时序：只令 `commands:job-finished` 的 `listen` Promise reject；start 返回 queued；进度 seq 0 正常进入 running；后台完成时该终态事件没有已注册监听。
- 当前结果：注册拒绝只写 logger。start 接纳 queued 后没有主动 status 查询。连续 seq 在增量模式下不调度对账。`jobError` 保持 null，界面不显示 Retry。任务保留 running，历史也要等路由重入触发 status 才能补记。
- 对照需求：父任务 `research/frontend-audit.md` 的 F03 可观察验收包括“订阅失败进入可见恢复态”；T07 R2 要求丢失事件可用 snapshot 恢复。现有路由重入恢复已实现，但注册失败没有反馈。
- 建议：仅在 command listener 注册失败时通知同一个 job owner，并提供既有 status 对账入口；覆盖拒绝早于 start、运行期间拒绝和随后重试。不要扩大 Usage 事件或修改 registry/DTO。采用什么监听恢复策略由实现者和主代理决定。
- 未修原因：处于只读检查阶段；实现者仍拥有源文件写权限。已向主代理和实现者报告。

### CP-T07-02 — P2 — 历史持久化失败只记录日志

- 证据等级：源码控制流确认；未执行故障注入测试。该限制存在于旧页面实现，不能描述为本次新回归。
- 文件：`ccr-ui/src/features/commands/commandJobStore.ts:69-78`、`:81-88`；`ccr-ui/src/features/commands/commandJobState.ts:50-60`。
- 可控时序：接收 terminal；令 `addRecentItem` 确定拒绝；再投递相同终态或重新挂载页面。
- 当前结果：job ID 在持久化成功前进入 `recordedJobIds`。拒绝只有日志，重复终态被 reducer 和 claim 双重忽略，页面重挂载也只读取已有 history。因此“每终态最多发起一次写入”已实现，“失败后仍不遗漏持久历史”没有保证。
- 已确认范围：主代理明确保留 history 后端 API/schema。前端区分 pending/saved/failed，拒绝必须可见；保留 terminal 和未持久摘要，不得虚称已经保存。同一 job 仍至多提交一次。当前 API 每次生成新 UUID，没有 job 幂等键，因此本任务不进行盲目重试，也不新增跨 IPC 持久 exactly-once 承诺。
- 未修原因：主代理已将该补充交给实现者；本轮保持只读。冻结后核对状态、可见错误和受控失败回归。

## 核对结果

- 单一任务投影：页面从 `useCommandsStreamStore` 选择 job、submitting、cancelling、error 和 history。页面原有三组 command listener 已删除。shell 在 `useShellRuntime.ts:15` 安装 bridge；Query 保留命令 catalog。
- snapshot 没有 seq watermark：Rust DTO 已确认。首次 queued 响应允许连续 seq 增量；显式对账或序号间隙后转为 snapshot 权威模式，后续增量不再重复拼接已覆盖输出。
- 终态：reducer 拒绝同 job 的终态回退；start 仅以响应 job ID 选择提前缓存的 terminal；generation 和 snapshotRequest 隔离迟到响应；cancel 返回 running 时不伪造 cancelled。
- 调度：bridge 首事件启动 250 ms 批量窗口，store 首次待对账事件启动 250 ms 定时器。后续事件不反复重置定时器。terminal 回调先立即提交 command batch，再接纳 terminal。
- 清理：disposed guard 阻止清理后的 command callback；listen 迟到 resolve 时立即调用 unlisten。
- 读取的测试源码包含 14 个真实页面/MemoryRouter 场景、8 个展开后的 reducer 用例和 4 个 bridge 生命周期场景。测试覆盖路由运行中返回、离页完成、start 前终态、同步双击、序号重复/间隙、持续输出、在途 status 合并、失去终态后重入、过期与瞬态故障、旧 job/status/cancel、cancel 非终态、start 失败、确认门槛和旧 history 读回。此处只记录覆盖意图，不能据此声称测试通过。
- 现有“旧响应”页面场景覆盖旧 status/cancel 和旧 event。由于 `submitting` 阻止第二次 start，公开 API 下不存在先启动新 job 再接纳更早 start 的通常路径；若最终报告声称单独覆盖旧 start 响应，应给出实际测试或说明该路径由 admission 排除。

## Verification

- Lint：本轮未执行；按主代理要求避免与实现者重复。
- TypeCheck：本轮未执行；T03 持有 bindings/inventory 生成窗口。
- Tests：本轮未执行；不引用实现者运行结果作为独立验证。
- Native / Web / 视觉：本轮未执行。
- 产物证据：对应 JSON 记录检查时的源码 SHA-256、检查范围和未执行项。源文件尚未冻结，后续完整检查必须重新确认相关 diff。

## 冻结后检查入口

1. 核对上述两处恢复边界的实现和新增测试。主代理已要求补 listener 拒绝的可见恢复态及真实 shell/CommandsView 对账回归，并按 CP-T07-02 的确认范围补历史持久化状态。
2. 读取实现报告及最终 diff，确认新增规范收录 owner、watermark 缺失、固定窗口、终态/history 和清理契约。
3. 在主代理释放检查窗口后执行 focused commands/shell 行为测试、type-check、lint 和 scoped diff check；原有临时脚本导致的全局 lint 失败单独保留。
4. 不触发 bindings/inventory 生成，不改原 Insights 任务状态，不提交或归档。
