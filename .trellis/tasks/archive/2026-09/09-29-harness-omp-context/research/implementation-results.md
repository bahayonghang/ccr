# T01 实施与验证记录

日期：2026-09-29（America/Chicago）；执行时间戳见 evidence/verification.json 的 UTC 字段。

## 结果

T01 实现和本地验收已完成，等待主会话独立复核。任务保持 in_progress。未暂存、提交、归档或修改全局配置。共享 harness 说明由 T02 回写。

## 文件与行为

- .omp/extensions/trellis/index.ts：新增同一份 TASK_CONTEXT_ARTIFACTS 列表，包含 prd.md、design.md、implement.md 和兼容 info.md。注入与缓存签名共用列表。
- 任务文档经现有 resolveProjectFile 和 trustedRoots 解析，然后走原有 materialize、UTF-8 处理和 ContextBudget。越界符号链接不读取内容。
- main、implement、check、research 都收到公开任务文档；各角色原有 JSONL 选择不变。research 不加载 implement/check manifest。
- .gitignore：仅使根目录 .omp/extensions/trellis/index.ts 可交付。设置、agents、其他扩展、缓存及嵌套 .omp 仍被忽略。
- scripts/trellis/omp-context.test.ts：保留语义标记断言；来源标题断言改为实际的路径和 inline 状态。新增 9 个边界或兼容测试。

## 验收对应

| AC | 本地结果 | 证据 |
| --- | --- | --- |
| T01-AC1 | PASS | 四类角色收到 PRD/设计/实施标记；info 兼容；JSONL 角色隔离；设计修改、实施创建、实施修改、设计删除分别触发同一会话缓存刷新。 |
| T01-AC2 | PASS | 轻量与缺文件；非法、绝对和相对越界路径；越界符号链接拒绝；显式 trustedRoots 接受；按完整 UTF-8 字符截断；总预算、极小预算、二进制和超额 manifest。 |
| T01-AC3 | PASS（交付候选） | 用明确路径和 git ls-files --cached --others --exclude-standard 选出 3 个文件。临时目录内没有 node_modules、个人状态或安装后的 Trellis 文件；同一 Bun 测试通过。 |

交付候选包含未提交、非忽略的新源码。该检查没有证明已提交 clean clone 的状态。最终纳入提交和主会话源码复核仍由主会话负责。

## 检查

基线为 3 pass / 2 fail，原始失败保留在父任务 research/evidence 与审查记录中：真实设计文档缺失和旧 PRD 标题断言分别处理。

| 检查 | 结果 |
| --- | --- |
| bun test scripts/trellis/omp-context.test.ts | 14 pass，0 fail，99 expect；无跳过。 |
| 独立 selected-source 目录执行同一命令 | 14 pass，0 fail，99 expect；无跳过。 |
| git check-ignore --no-index --stdin -z 的 7 路径探针 | 1 个指定源码放行；6 个本地/非白名单路径仍被忽略。 |
| 独立目录 bun build --target=bun | PASS；外部类型导入已消除。 |
| git diff --check（既有两文件） | PASS。 |
| git diff --no-index --check（新源码） | 无空白错误；退出码 1 表示新文件有差异，另要求输出和错误流均为空。临时命令参数处理 CRLF，未修改 Git 配置。 |
| task.py validate T01 | PASS，两个 manifest 各 3 条有效上下文。 |

可复现入口：python .trellis/tasks/09-29-harness-omp-context/research/verify_selected_source.py。执行参数、退出码、OS、Bun 版本、HEAD、文件 SHA-256 和日志 SHA-256 见 evidence/verification.json。

运行时静态导入只有 node:fs、node:path、node:child_process、node:crypto。@oh-my-pi/pi-coding-agent 仅作类型导入，独立构建确认产物不含该导入。测试不需要安装该包。

验证脚本第一轮误报已单独保留在 evidence/verification-attempt-1.json：Windows 文本 stdin 将路径行尾转换为 CRLF；no-index 新文件退出码 1 被当成失败。随后使用 NUL 分隔并检查空白诊断，第二轮 8 项检查通过。原产品测试在这两轮均通过。

## 保留边界

- 本机 Bun 1.4.2；仓库声明 1.4.0，未升级或降级全局工具。
- 原生 OMP 加载、扩展 API 类型检查、客户端信任和新会话注入：UNVERIFIED。
- Linux/macOS、托管 CI：UNVERIFIED。本次 Windows 文件符号链接测试实际执行并通过。
- Trellis 更新保留策略：不修改 .trellis/.template-hashes.json；本地定制仍由原有更新冲突检测管理，最终共享说明由 T02 负责。
- 不修改 ContextBudget、UTF-8 算法、预算默认值、可信根配置、角色权限或原生客户端设置。
