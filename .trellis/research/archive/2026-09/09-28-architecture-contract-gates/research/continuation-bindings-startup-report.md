# T10 Bindings 启动异常复验记录

Owner：`/root/implement_t09`。执行时间使用各 JSON 的 UTC 字段。本轮只写 `research/continuation-bindings-startup-*` 证据，没有产品源码补丁。沿用已经读取的 `trellis-before-dev`、`ccr-gate-recovery` 和 `diagnosing-bugs` 流程。

## 结果

单独的 `just tauri-bindings-check` 和完整 `just tauri-ci` 均通过。两次复验前后，230 个生成类型的路径集合及内容 SHA、263 个冻结文件的路径集合及内容 SHA 全部一致。

原始 `just ci` 仍记录 exit 1。其 `ccr_cli-957d4a190e1b8043.exe export_bindings` 在输出 `running N tests` 前以 `0xc0000005 STATUS_ACCESS_VIOLATION` 退出。后续成功复验没有替代该失败，启动异常的原因未查明。

## 二进制与事件取证

原请求路径为 `target/debug/deps/ccr_cli-957d4a190e1b8043.exe`。解析后的真实路径在 `continuation-bindings-startup-before.json` 中记录，指向既有 mbx target 映射。该路径事实不能单独证明 wrapper 导致崩溃。

首次故障后采集的 SHA 为 `a9f059fca10c030a9c07d4bb7f938c3e6041b22ad245a001efbe6b1e6bee1ab5`，采集时间为 `2026-09-28T23:59:20.442234+00:00`。原失败发生时没有单独采集二进制 SHA，因此该值明确标为 post-failure observation。

同一实际二进制执行 `--list export_bindings`：exit 0，列出 24 项测试，耗时 0.027 秒。列表动作没有执行导出测试。该探针和后续两次门禁的二进制 SHA 保持相同。

仅查询 `Application / Application Error / Event ID 1000`，时间范围为原 CI 的 `2026-09-28T23:43:55.849741Z` 至 `2026-09-28T23:55:46.639741Z`。结束时间按原始 started_at 与记录的 seconds 推算。结果没有匹配事件，没有取得 faulting module 或 offset，未输出其他程序日志。首次时间转换丢失小数的查询记录单独保留；最终查询使用原始 ISO 时间。

事件查询未命中不能证明未发生崩溃。成功列举、相同二进制复验通过也不能单独定位崩溃机制。

## 精确执行与结果

所有命令在仓库根目录执行。两项 just 命令仅显式设置与原始根 CI 相同的 `CCR_SKIP_ICON_GENERATION=1`，保持既有 Rust 1.98.0 pin 和 wrapper。没有调整系统安全策略或手动删除缓存。生成脚本已有的 export-only `RUST_TEST_THREADS=1` 保持原状。

| 命令 | 结果 | 记录耗时 | 元数据 |
| --- | --- | --- | --- |
| 同二进制 `--list export_bindings` | exit 0；24 项列举 | 0.027 秒 | `continuation-bindings-startup-list.json` |
| `just tauri-bindings-check` | exit 0；24 CLI + 9 usage + 197 Tauri 导出测试通过；无类型漂移 | 60.203 秒 | `continuation-bindings-startup-bindings-check.json` |
| `just tauri-ci` | exit 0；407 行为测试通过、1 项既有忽略；2 guard；24+9+197 导出测试；1 inventory 测试通过 | 230.503 秒 | `continuation-bindings-startup-tauri-ci.json` |

完整 Tauri recipe 同时执行 dependency governance、fmt、check、严格 bin Clippy、行为测试、bindings drift 与 command inventory。单独 bindings 检查执行一次，完整 Tauri CI 执行一次；完整 recipe 内的 bindings 检查属于其已有步骤。

本 owner 没有重跑已经通过的根 workspace Test、Release 或完整 `just ci`。原根 CI 日志和元数据的完整副本为 `continuation-bindings-startup-original-ci.log/json`。

## 内容快照与范围

取证初始快照、单独 bindings 复验前后快照、完整 Tauri CI 前后快照共 5 份，均包含全部 263 冻结路径和 230 生成类型路径的 SHA。5 份快照的路径集合与内容哈希全部一致。每项执行还记录命令、UTC 时间、exit code、日志 SHA、前后快照 SHA 和二进制 SHA。

bindings 检查依照既有事务机制暂时生成类型，再恢复调用前字节。内容一致性不代表文件修改时间没有变化。本轮未修改任务状态、父报告、矩阵或受保护脚本，未提交、推送或归档。

## 交接与剩余边界

Windows Cargo 已释放，session `19398` 已结束，本 owner 无活动构建或测试。汇总元数据为 `continuation-bindings-startup-report.json`。

原始根 CI 保持失败；原启动访问冲突原因未查明。Windows Rust 1.95 Tauri check 与 Linux Tauri process smoke 由 root 串行执行，本记录不预先宣称通过。macOS 和原生 WebView 验收不在本轮证据范围。
