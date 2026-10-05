# T10 门禁实施报告

日期：2026-09-28。实施 owner：`/root/implement_t10_gates`。用户已批准父任务全部规划。本文只记录门禁、绑定事务和两个已验证基线缺陷的实施结果；最终父任务验收由 root 汇总。未提交、推送、归档或修改 Insights 生命周期。

## 实现结果

1. 根 `just ci` 的 Windows、Linux、macOS 定时步骤复用现有 `tauri-ci`，覆盖独立 Tauri fmt/check/strict Clippy、全部行为测试、绑定漂移和命令 inventory。hosted required lanes、平台矩阵和 coverage 阈值未修改。
2. 普通 Rust 行为测试保留默认并行，并仅通过 `--skip export_bindings` 排除 ts-rs 的生成副作用。范围包括 root test/test-all/coverage-rust/coverage-tauri/tauri-ci，UI test-backend 和已获 root 授权的 package.json tauri:test。三个平台 workflow-governance-check 现在执行实际聚合行为 fixture。
3. `bindings-transaction.mjs` 是单一目录快照/恢复 owner；保存二进制字节、嵌套/空目录及原目录存在状态。直接生成仅在三个 export 阶段和 normalizer 全部成功后保留输出；非零、spawn 异常和其他异常恢复原目录并保持失败。恢复失败显式抛出错误并保留原操作异常。
4. `generate-bindings.mjs` 复用既有三组 Cargo exports 和 normalizer；只有 export 子进程设 `RUST_TEST_THREADS=1`。check 比较规范化基线；无论成功、漂移、子进程或 normalizer 失败，均恢复调用前原字节。没有更改 normalizer 算法，没有新增依赖。
5. Windows 文件锁回归经持锁句柄读取元数据，竞争失败前后与释放后原断言均保留。生产锁实现未变。Usage 日期跨度按本地日期的日历序号计算，避免把 DST 的23小时日期算成不足一天。

## 文件范围

- root/UI justfile、package.json 单条 tauri:test。
- `ccr-ui/scripts/{bindings-transaction,generate-bindings,check-generated-bindings}.{mjs,d.mts}`。
- `scripts/ci/check_workflow_governance.py`、现有 governance 测试及新 `test_architecture_contract_gates.py`。
- `crates/ccr-core/src/core/lock.rs` 的一个测试；`ccr-ui/src/views/usage/dateWindow.ts` 及日期回归。
- `ccr-ui/tests/quality/bindings-transaction.smoke.test.ts`。
- 本任务 research 与报告。最终16个源码/测试/编排文件的 SHA-256、授权 diff 指纹见 `research/gates-verification.json`。

## 执行验证

| 命令/证据 | 实际结果 | 范围 |
|---|---|---|
| governance Python suite | 27 passed | 三平台 recipe 合同；Windows 实际 just ci 成功/失败 fixture；既有 hosted governance |
| `just workflow-governance-check` | 27 passed + validator passed | 正式本地治理入口 |
| 日期与绑定 smoke | 25 passed /2 files | 日期9；真实临时目录事务16，包含失败恢复、二进制、嵌套/空目录、缺目录、恢复异常 |
| 单例 native metadata lock | old 0/1 failed OS33 → new 1/1 passed | Windows 实际排他文件锁，所有元数据断言保留 |
| `just tauri-bindings` | exit0；24+9+197 export tests | CLI / ccr-usage / Tauri |
| 最终 `just tauri-bindings-check` | exit0；24+9+197 export tests | 调用前后230个 types文件；无缺失、新增或字节变化 |
| `bun run type-check` | exit0 | 生成事务完成后重跑 |
| T10 scoped ESLint | exit0 | 仅修改 owner；不替代正式 lint:ci |
| `cargo fmt --all -- --check` | exit0 | root Rust 格式 |
| `git diff --check` | exit0 | 全工作区；仅 CRLF warning |

聚合行为 fixture 从真实 root justfile 提取原 `ci`、三平台定时 recipe 和 `tauri-ci`，只把实际工具依赖换为临时子进程 fixture。注入 desktop test 断言失败时真实 `just ci` 非零，且不会继续 bindings/frontend；成功路径确实调用 strict Clippy、全行为测试、bindings 和后续步骤。Windows 实际执行已验证；Linux/macOS 聚合执行尚未运行，仅其步骤列表和原 hosted 平台配置合同已检查。该 fixture 不替代真正桌面行为 suite。

## 失败记录与边界

- 初次正确日期回归：7 passed/2 failed。America/Chicago 返回 allTime=99、March=30、spring=2；修正后分别100、31、3。UTC/Shanghai保持原期望，fall=3。首次测试编写曾遇 URL 构造问题，原日志保留，`gates-date-red-final.log` 才是有效产品反例。
- 修正后的第一次 core test exe 启动失败 `0xc0000005`；没有重编译或源码更改，后续原命令通过。失败产物的运行前 SHA 未记录，不声称两个运行产物 hash 相同；成功重跑前 SHA 为 `EB621D7A777E5E8AB8EF8C4F80BFD4748BF9342B8CA758C852D1B3B073401481`。原因未查明。
- 首次完整 guard 报六个类型文件 drift并exit1，原230文件全部恢复。对应路径见 `gates-full-bindings.log`，准确差异内容当时没有在恢复前保存。随后 direct generation 与初始快照逐字节相同；分阶段 instrumentation 和最终正式 guard 均成功。分阶段目录路径及 SHA证据保留；早前漂移原因未查明，不将并行、Bun或mbx当作已证原因。`gates-generation-drift.diff`为空只表示后续 direct结果没有差异。
- 初次 type-check 误与真实生成并发启动，读取临时缺失的生成物而失败；已停止这种排序并在生成结束后通过。初期新增脚本 declaration 和测试 matcher 问题已修正；失败日志未删除。
- `src/types/generated`为230文件；此前上下文260是含API等其他产物的口径，不能套用到本目录。
- 原有两个 `.tmp` 文件 SHA 与基线一致；本切片未运行/宣称正式 lint:ci 通过。root负责最终正式全量门禁和CI集成。未运行Linux/macOS原生锁/完整桌面UI。
- 实现和生成窗口于10:54（America/Chicago）释放；源码最终冻结于10:58，随后只交叉审查非本人规范产物。
