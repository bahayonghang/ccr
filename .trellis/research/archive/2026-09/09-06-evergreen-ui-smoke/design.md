# 设计

## 机制

在 vi.hoisted 的 stubForName 内先匹配 Agent Sessions 的具体函数，使用由真实生成 DTO 约束的局部常量；同一 fixture 在涉及底层 invoke 的路径复用，避免两套值漂移。为 providers/list/detail/start/status 返回各自正确形状，终态使用 finished 阻止不必要的轮询。vi.hoisted 中运行期不得引用后初始化的普通变量；类型 import 可擦除。优先在现有文件完成，不抽象通用 mock 注册器。保持 AgentSessionsView.tsx 不变，因为当前没有真实接口返回坏形状的证据。

## 所有权与顺序

独立于 CI/OMP，优先执行；通过后把结果交规则子任务。

## 工具分工

主线程强模型确定契约，明确方案后可由低成本档执行白名单修改，最终由独立强模型审阅。 工具边界见父任务 research/harnesses.md；优先 Codex/Claude 的 shell 与源码审查，Grok 的只读 plan/explore 不承担 shell 测试；Kimi/OMP 执行必须给完整任务内容。没有实际解析模型和价格证据时不宣称更便宜。

## 约束与撤回

不改 AgentSessionsView 产品容错，不改变 IPC DTO、后端返回或创建通用 mock 引擎。 仅撤回本次拥有的文件差异，保持其他任务变更；不创建兼容层。所有批准项通过父规则子任务回写适用工具和检查结果。
