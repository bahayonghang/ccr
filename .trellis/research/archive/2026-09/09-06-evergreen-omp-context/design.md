# 设计

## 机制

在现有 buildTaskContext 的相邻读取段增加 design.md 和 implement.md 标题/内容；保留 prd/info 既有行为，不新建抽象层、不改变路径信任或JSONL解析。测试用 Bun 内建 test，不安装 OMP 依赖；index.ts 的 ExtensionAPI 为 type-only import。通过默认 extension 导出注册 mock ExtensionAPI，触发 session_start，临时 fixture 目录中设置 .trellis 当前任务解析所需最小文件，捕获 sendMessage 内容。按 PI_BLOCKED_AGENT 三角色和主会话逐项运行并恢复环境；不启动真实代理。缺文件与manifest角色隔离是独立行为判据。真实 OMP 当前环境需在授权/可用条件下另验证，不以 mock 冒充真实 hook。

## 所有权与顺序

行为夹具使用临时仓库中的 .trellis/.runtime/sessions/omp_review-fixture.json，内容 current_task 指向相对任务目录；该目录含 task.json（status/title）及三件套。ctx.cwd 指向临时仓库，sessionManager.getSessionId() 固定为 review-fixture；buildContextKey 的 session 分支生成 omp_review-fixture（不含 session_ 中缀）。补齐 ctx.ui.notify 与 extension 注册所需的无副作用 mock，捕获 pi.sendMessage。分别设置/清除 PI_BLOCKED_AGENT 后注册新扩展实例，测试结束恢复环境并只清理自有临时目录。主会话夹具不放 .trellis/scripts/get_context.py，因此 buildSessionContext 直接返回空，无 Python/模型子进程。不要使用该 loader 不消费的 .current-task 作为夹具入口。

可以独立实现；规则子任务的 OMP 部分在本项结果确定后收口。

## 工具分工

主线程强模型确定契约，明确方案后可由低成本档执行白名单修改，最终由独立强模型审阅。 工具边界见父任务 research/harnesses.md；优先 Codex/Claude 的 shell 与源码审查，Grok 的只读 plan/explore 不承担 shell 测试；Kimi/OMP 执行必须给完整任务内容。没有实际解析模型和价格证据时不宣称更便宜。

## 约束与撤回

不改 Trellis 上游仓库或用户全局配置、不新增依赖/信任机制、不重构上下文缓存/压缩流程、不擅自运行付费模型。 仅撤回本次拥有的文件差异，保持其他任务变更；不创建兼容层。所有批准项通过父规则子任务回写适用工具和检查结果。
