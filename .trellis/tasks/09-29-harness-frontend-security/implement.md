# T08 实施计划

## 启动条件
- [x] 用户在父任务最终总结后明确批准本项。
- [x] 重新核对 git status、关联来源与当前依赖版本。
- [x] 前置项 无前置子任务 满足其验收条件。
- [x] 本任务上下文通过 task.py validate 后才由主会话运行 task.py start。

## 步骤
- [x] 复核 F01 与设计文件白名单；发现新范围先报告。
- [x] 按 design.md 实施最小修改，并保留原失败证据。
- [x] 逐项满足 prd.md 的 3 条 AC。
- [x] 强模型独立审查完成；T02/T03/T06 见父任务最终集成审查，T01/T04/T05/T08 见首轮审查。
- [x] 回写规定说明并注明适用工具；不自动提交归档。主会话完成共享依赖规范回写，T08 已复核。

## 必须通过的检查
- [x] `cd ccr-ui && bun install --frozen-lockfile`
- [x] `just frontend-audit`
- [x] `cd ccr-ui && bun run test:smoke -- tests/quality/frontend-dependency-audit.smoke.test.ts`
- [x] `just frontend-check`，环境 `CCR_SKIP_ICON_GENERATION=1`。
- [x] `just frontend-coverage`
- [x] `just version-check`
- [x] `git diff --check`

## 证据
记录命令、退出码、日期、OS、版本、输入 SHA 和输出摘要。没有运行的正式环境检查保留 UNVERIFIED。不要把 shell 退出 0、jsdom、原生测试或托管结果互相替代。

实施与检查证据：[research/implementation-results.md](research/implementation-results.md)。最终独立审查已完成，见父任务 research/independent-review.md。

## 新返回公告后的继续实施

上方勾选和证据属于首次四包补丁。相同 lock SHA 的后续完整 CI 收到 7 条公告命中；该失败和旧 PASS 都保留。主会话确认只更新 UI 四个解析节点仍在原批准范围，扩展新增范围由主会话另行确认。

- [x] 核对四项 GitHub 公告、实际父范围和三个唯一 tarball SHA-512。
- [x] 保存原始 lock 字节与743节点，仅更新 UI 四节点。
- [x] 首次冻结安装、audit 0、policy 回归3/3通过。
- [x] 完整前端和覆盖率检查：169 文件/904 测试；Lines 75.21%（13083/17395），Statements 72.93%（14655/20092）；回执注明首次 CRLF SHA。
- [x] 独立审查发现首次补丁把 LF 写为 CRLF；已恢复原 LF，四行反向替换逐字节还原原文件，最终冻结安装、审计 0 与定向 diff 检查通过。
- [ ] 主会话独立复核与最终完整 CI。

后续记录：[research/new-advisories-implementation-results.md](research/new-advisories-implementation-results.md)。任务继续保持 `in_progress`，不提交或归档。
