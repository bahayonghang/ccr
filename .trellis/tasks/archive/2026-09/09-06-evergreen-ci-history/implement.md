# 执行计划

## 文件白名单

- .trellis/tasks/09-06-evergreen-ci-history/research/verification.md（新增）

## 顺序

可独立批准；前端coverage复核等UI子任务通过后；本项不阻塞前四项局部完成。

- [x] 获取本子项的明确实施批准，检查最新计划，再 task.py start。
- [x] 阅读 implement.jsonl/check.jsonl 的真实规范、父报告和 design.md。
- [x] 按设计限定的次数和现有环境进行只读复核。
- [x] 逐条运行下列检查，记录命令/cwd/退出码；预期失败的负例以非零为通过。
- [x] 强模型独立审查，向父任务/规则子任务交付适用工具与证据；缺证据明确列出。

## 检查

~~~text
cargo test -p ccr-cli --all-features --locked --offline non_dry_run_doctor_persists_sanitized_report
just tauri-process-smoke
已有工具和授权条件满足时：just coverage-rust；cd ccr-ui && bun run test:smoke --coverage
~~~

有生成/安装副作用的聚合命令先检查组成步骤；仅使用已有依赖。本地检查不证明当前 hosted 或真实客户端通过。上述条件式覆盖率命令缺条件时允许以 UNVERIFIED 结束调查，不允许标该gate通过。

## 完成和权限

本项 AC 与父任务已批准集的交付约束均满足才算完成。用户 `/goal` 已批准本子任务在 check PASS 后提交并归档；仍不发布、不编辑用户全局文件。不自动调高timeout、串行化、降低阈值、吞IO/teardown错误，不编辑产品代码、不安装工具、不触发hosted重跑。
