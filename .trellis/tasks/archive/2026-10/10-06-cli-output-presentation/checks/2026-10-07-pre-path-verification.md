# CLI 信息展示：当前源码验收

状态：UNVERIFIED，实施与安全检查已推进，完整验收未完成。2026-10-07。HEAD `528d4bae1b145fd74d7bbc97454caac249a4b896`，分支 `dev`。父任务与 C1–C3 保持 `in_progress`；未新增提交、推送或归档。

## 已完成的里程碑

- C1：共享主体已在 `c72ba99f`。启动调用、环境降级进程测试、独立源码复查和共同 strict lint 已有当前源码证据。补充验收 3/3 可关闭。
- C2：独立复查已完成源码边界、建议、参数、JSON/业务调用审查；修正空导入结果的重复 warning，纯回归 3 项通过。修正后的完整进程复测未运行。
- C3：非 Auth 提示、Doctor 标记、文档和共享消费者已复核。独立检查修正无效/缺参数建议、步骤分类、重复进度及正常空状态；Doctor 安全渲染探针验证四种状态、stdout 与空 stderr。完整 CLI/native 验收仍开放。
- 新增 `.trellis/spec/ccr-cli/backend/cli-output-presentation.md` 并更新 index；独立 checker 已复核。
- 保留 931 个基线调用及 452 个基线语义入口。调用处置：修改 494、共享覆盖 302、保留 135。追加当前 1091 个引用与 961 个语义入口；72 份源码 SHA-256 与清单一致。

## 当前源码门槛

| 命令 / 检查 | 结果 | 回执 |
| --- | --- | --- |
| `just version-check` | PASS，exit 0 | `2026-10-07-continuation-version-check-final.txt` |
| `just fmt-check` | PASS，exit 0；前一轮 exit 1 保留 | `2026-10-07-continuation-fmt-check-retest.txt` |
| `just lint-strict` | PASS，exit 0，完整 Rust workspace/all-targets/all-features | `2026-10-07-continuation-lint-strict-retest.txt` |
| `just check-workspace` | PASS，exit 0 | `2026-10-07-continuation-check-workspace-retest.txt` |
| `git diff --check -- crates docs .trellis/spec` | PASS，exit 0 | `2026-10-07-continuation-diff-check.txt` |
| 当前调用/语义清单及源码 hash 审计 | PASS | `2026-10-07-continuation-inventory-verification.json` |
| Windows 原生共享输出混合流矩阵 | PASS，12/12；normal、force、dumb-force、NO_COLOR × stdout/stderr/both 重定向 | `2026-10-07-native-stream-verification.json` |
| docs `bun run build`、`bun run audit` | PASS，各 exit 0；4 份活动文档此后未变化 | C3 `checks/verification.md` 及独立回执 |
| C1 logging / process / feedback | PASS，11 / 7 / 3 项 | C1 当前源码独立回执 |
| C2 Codex / Claude Auth unit | PASS，6 / 1 项；import 纯回归 3 项 | C2 当前独立回执 |
| Doctor 安全 renderer unit | PASS，2 项；ignored 测试探针由测试实际启动 | C3 `2026-10-07-independent-doctor-renderer-result.json` |
| 最终完整 CLI binary / 相关命令与 crate 测试 | NOT_RUN，Windows 路径隔离前提不成立 | 下述边界 |
| 最终 handler 40/80/120 列 × 明暗背景 × normal/NO_COLOR/dumb | NOT_RUN，日志路径未隔离 | 下述边界 |
| `just ci` | NOT_RUN，包含未隔离的 CLI 进程测试 | 下述边界 |

源码身份见 `2026-10-07-continuation-source-manifest.json`。首轮 PASS、独立修正后 fmt FAIL 和最终复测分别保存。没有改写首失败，也没有以静态 PASS 替代完整门槛。

共享输出原生矩阵启动的是已核对的 core 测试探针；探针不调用 logger、DoctorService 或账号服务。该证据只验证共享输出与目标流能力。Windows console 40 列预检成功，但预检和共享探针不关闭完整 handler 终端验收。

## AC1–AC10

| AC | 状态 | 当前证据与剩余项 |
| --- | --- | --- |
| AC1 | UNVERIFIED | save 源码、字段和既有显示断言已复核；最终实际终端 save 首行未执行。 |
| AC2 | UNVERIFIED | 状态 formatter、四状态 Doctor renderer、计数/取消/空状态源码已复核；完整隔离的 CLI 状态矩阵未完成。 |
| AC3 | PASS | 调用语义复核、共享格式/样式断言与原生共享输出矩阵支持；保留布局另有明确范围理由。 |
| AC4 | UNVERIFIED | 安全参数与建议源码审查完成；修正后的完整进程、原生复制命令复测未运行。 |
| AC5 | UNVERIFIED | C1 环境测试与 12 组原生混合流 PASS；最终 handler 与保留布局的完整原生矩阵未完成。 |
| AC6 | UNVERIFIED | service/DTO/JSON/返回路径边界审查完成；最终完整隔离的机器接口复测未运行。 |
| AC7 | UNVERIFIED | Auth 纯单测及已有输出断言保留；Windows logger 未隔离，完整合成进程验收不能关闭。Unknown 仅源码复核。 |
| AC8 | PASS | 931 基线调用、452 语义入口均有处置；当前 1091 引用和 961 语义入口已追加，hash 审计通过。 |
| AC9 | UNVERIFIED | 实际 PTY 可用；完整 CLI 窄终端、主题、长字段、建议与多行矩阵未运行。 |
| AC10 | UNVERIFIED | 子任务安全检查与静态门槛有当前源码证据；父 `just ci` 未执行。Linux/macOS/hosted 均 NOT_RUN。 |

## Windows 隔离边界

`dirs::home_dir()` 通过系统 Known Folder 解析目录，子进程 HOME/USERPROFILE 覆盖不生效。Doctor ConflictChecker 仍直接读取该目录下的官方设置；普通 CLI logger 仍在该目录的 `.ccr/logs` 初始化 writer 并执行 14 天清理。`CCR_LOG_LEVEL=off` 不关闭日志初始化。Doctor/Validate 在 main 的 logger 初始化前返回，保留只读诊断流程。

Windows 权限 helper 为 no-op。已执行进程对真实日志目录的实际变更范围未审计；未读取真实日志或凭据来补证。已生成的 Doctor 失败回执不能标为完全合成；已做敏感冲突值布尔审计，结果为 false。

发现前提冲突后停止 CLI binary/native 复测与完整进程套件。新增任务进程测试在 Windows 保留编译并使用有具体原因的 ignore；ignored 不计作验收 PASS。已执行历史 binary 记录仅保留其显示断言与业务数据断言边界。最初 release artifact 的源码身份未证明，仅属辅助历史输出。

路径业务和 logger 不在原展示授权范围内。待授权方案见 [windows-fixture-isolation-scope.md](../research/windows-fixture-isolation-scope.md)。该方案未实施；全部必需未验证项保持开放。

## 2026-10-07 路径隔离扩展批准

用户已明确批准 research/windows-fixture-isolation-scope.md 中的最小路径修复。上述暂停状态保留为发现缺口时的证据；当前阶段为 APPROVED_PENDING_IMPLEMENTATION。实施通过独立安全复核后，恢复完整进程、原生终端及 just ci。提交、推送和归档仍未授权。
