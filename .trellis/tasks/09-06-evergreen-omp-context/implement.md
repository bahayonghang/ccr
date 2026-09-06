# 执行计划

## 文件白名单

- .omp/extensions/trellis/index.ts
- .omp/agents/trellis-implement.md
- .omp/agents/trellis-check.md
- .omp/agents/trellis-research.md
- scripts/trellis/omp-context.test.ts（新增）

## 顺序

可以独立实现；规则子任务的 OMP 部分在本项结果确定后收口。

- [x] 获取本子项的明确实施批准，检查最新计划，再 task.py start。
- [x] 阅读 implement.jsonl/check.jsonl 的真实规范、父报告和 design.md。
- [x] 重现对应失败/缺口，实施最小变更，保留有意义的正负回归。
- [x] 逐条运行下列检查，记录命令/cwd/退出码；预期失败的负例以非零为通过。
- [x] 强模型独立审查，向父任务/规则子任务交付适用工具与证据；缺证据明确列出。

## 检查

~~~text
bun test scripts/trellis/omp-context.test.ts
git diff --check
~~~

有生成/安装副作用的聚合命令先检查组成步骤；仅使用已有依赖。本地检查不证明当前 hosted 或真实客户端通过。必要检查缺条件时该项验收保持未完成，不能用其他命令的通过替代。

## 完成和权限

本项 AC 与父任务已批准集的交付约束均满足才算完成。用户 `/goal` 已批准本子任务在 check PASS 后提交并归档；仍不发布、不编辑用户全局文件。不改 Trellis 上游仓库或用户全局配置、不新增依赖/信任机制、不重构上下文缓存/压缩流程、不擅自运行付费模型。`.omp/` 被 gitignore；提交时仅对白名单内四个 `.omp` 文件做路径级 `git add -f`，禁止 `git add -f .trellis/`。
