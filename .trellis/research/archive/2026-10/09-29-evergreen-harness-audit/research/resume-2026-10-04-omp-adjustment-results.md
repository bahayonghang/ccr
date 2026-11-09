# T01 后续验收调整实施结果

日期：2026-10-04（America/Chicago）。基线：`dev / c50716b5`。用户批准：`批准保留本地生成并调整验收（推荐）`。状态：批准的测试状态输出与双语说明已实施，两个受控运行边界已验证。

## 变更

- `scripts/trellis/omp-context.test.ts`：保留原 `describe.skip` 分支。缺少本地生成扩展时输出 `SKIPPED_UNVERIFIED`，明确全部 14 项未执行、需要匹配版本生成扩展，以及 native loading/trust 另验。14 项既有测试和断言未改。
- `docs/agents/harnesses.md` 与 `docs/en/agents/harnesses.md`：Git 交付章节增加同义段落。匹配版本本地生成扩展存在且全部 14 项实际执行并通过才可记录契约 PASS；缺扩展的退出 0 和 skip 不关闭验收。新检出生成、原生加载和 trust 分别记录。

`.omp/` 继续保持本地生成且被忽略；未安装工具、恢复已跟踪扩展或修改本地扩展。历史验收和归档材料由主会话维护，原归档时间边界未重写。

## 验证

| 边界 | 实际命令与结果 | 回执 |
| --- | --- | --- |
| 本机现存生成扩展 | `bun test scripts/trellis/omp-context.test.ts`，退出 0，14 pass / 0 fail / 99 assertions | [本机](resume-2026-10-04-omp-adjustment-local.json) |
| 仅当前工作区测试源码的临时夹具 | 同一命令，退出 0，0 pass / 14 skip / 0 fail；明确输出 `SKIPPED_UNVERIFIED` 与 14 项未执行说明 | [缺失扩展](resume-2026-10-04-omp-adjustment-absent.json) |
| 三个修改文件的 `git diff --check` | 退出 0 | [空白检查](resume-2026-10-04-omp-adjustment-diff-check.json) |

临时夹具只复制当前测试源码的原始字节，没有 `.omp/` 或个人目录补齐。自有临时路径在删除前限定于系统临时目录。Bun 版本为 1.4.2。所有命令的 stdout/stderr 原始日志、字节数、SHA-256、日期、OS、HEAD、测试源码/本地 extension/双语说明的运行前后 SHA 均保留在同名前缀证据中。两组运行的所有输入哈希前后一致。

缺失扩展夹具状态为 **SKIPPED_UNVERIFIED**；该退出 0 不构成契约 PASS。本机结果针对记录的生成扩展输入 SHA，仅证明现存源码的受控 14 项契约通过。本轮未执行全新客户端生成流程或原生 OMP session。native loading/trust、fresh-client 和 hosted 保持 **UNVERIFIED**。父任务负责完整 `just ci` 与独立复核。

补查 `trellis --version` 返回安装 CLI 0.6.17，项目 `.trellis/.version` 为 0.7.0-beta.4。扩展源码没有嵌入生成版本，无法用当前工具版本证明原扩展来自匹配版本。**匹配版本生成仍为 UNVERIFIED**；不能据本机 14 pass 关闭调整后的 AC3。原始版本输出、项目版本及源码前后 SHA 见[版本回执](resume-2026-10-04-omp-adjustment-version.json)。没有升级 CLI 或重新生成扩展。

没有修改 `.trellis/scripts/`、ignored extension、全局配置、task/spec、共享任务指针，也没有提交、归档或推送。
