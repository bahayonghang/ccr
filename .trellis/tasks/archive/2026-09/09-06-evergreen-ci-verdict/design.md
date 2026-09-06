# 设计

## 机制

仅给 VSCode coverage run step 添加 shell: bash。精确将 .cargo/tauri-ci.toml 加 tauri，将 .cargo/config.toml 加 root+tauri，将 .cargo/audit.toml 加 root。依据：根 .github/workflows/ci.yml:148 在根目录运行 cargo audit；当前 Tauri workflow/justfile 没有 cargo audit 消费者，不能推定 tauri 也要触发。沿用现有 SURFACE_PATHS 与测试类，不改 required job 的无关逻辑。在现有治理测试中加有行为价值的 path 正负用例；管道用失败生产者+tee验证真实退出码，正常生产者验证成功。若需持续自动验证管道，可在现有测试中读取该step的shell后调用本机Bash，缺Bash则明确skip，不加新验证器。

## 所有权与顺序

独立于 UI/OMP；修改完成后向规则子任务交付检查退出码和精确路径映射。

## 工具分工

主线程强模型确定契约，明确方案后可由低成本档执行白名单修改，最终由独立强模型审阅。 工具边界见父任务 research/harnesses.md；优先 Codex/Claude 的 shell 与源码审查，Grok 的只读 plan/explore 不承担 shell 测试；Kimi/OMP 执行必须给完整任务内容。没有实际解析模型和价格证据时不宣称更便宜。

## 约束与撤回

不新增 CI 引擎，不改依赖版本，不远程触发 workflow，不改变 coverage 阈值。 仅撤回本次拥有的文件差异，保持其他任务变更；不创建兼容层。所有批准项通过父规则子任务回写适用工具和检查结果。
