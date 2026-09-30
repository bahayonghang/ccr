# T01 实施计划

## 启动条件
- [x] 用户在父任务最终总结后明确批准本项。
- [x] 重新核对 git status、关联来源与当前依赖版本。
- [x] 本项无前置子任务。
- [x] 本任务上下文通过 task.py validate，主会话已运行 task.py start。

## 步骤
- [x] 复核 F02、F03 与设计文件白名单；缓存依赖与 artifact 可信路径补齐已由主会话确认。
- [x] 按 design.md 实施最小修改，并保留原失败证据。
- [x] 逐项满足 prd.md 的 3 条 AC；交付候选与已提交 clean clone 的证据边界见实施记录。
- [x] 强模型独立复核代码、边界、负例与证据；见[独立审查](../09-29-evergreen-harness-audit/research/independent-review.md)。
- [x] T02 已回写双语 harness 说明，明确 OMP 注入、交付候选和原生验证边界；未提交归档。

## 必须通过的检查
- [x] `bun test scripts/trellis/omp-context.test.ts`：14 pass / 0 fail。
- [x] 使用 `git ls-files --cached --others --exclude-standard` 及明确路径构建自有临时夹具，同一 Bun 测试 14 pass / 0 fail；未使用 ignored 文件补齐，未声称已提交 clean clone。
- [x] `git diff --check`；新增源码另以 no-index 检查，无空白错误。

## 证据
记录命令、退出码、日期、OS、版本、输入 SHA 和输出摘要。没有运行的正式环境检查保留 UNVERIFIED。不要把 shell 退出 0、jsdom、原生测试或托管结果互相替代。

结果与边界见 [实施记录](research/implementation-results.md)；机器记录见 [验证结果](research/evidence/verification.json)。

## 本地交付补证（2026-09-30）

- [x] 从新提交 git archive 构造三文件夹具，14 tests / 99 assertions 通过，完成 T01-AC3 的已提交源码验证。
- 记录见 [已提交源码夹具](research/committed-source-fixture.md)。
- 用户已授权本地提交与验收完成后的归档；原始计划中的提交限制由父任务 delivery_authorization 覆盖。
