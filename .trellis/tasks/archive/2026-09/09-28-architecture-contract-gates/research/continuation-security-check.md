# T10 后续安全锁文件非作者检查

日期：2026-09-28。审查者：`/root/implement_t10_specs`。授权范围为两份 Cargo.lock 的安全修复及对应证据；另外复核 Linux 定向测试完整日志。本审查只写本报告及同名 JSON。

安全锁文件切片通过本次非作者范围审查，阻断性发现为 **0 项**。两锁修复前副本与 Git HEAD、continuation-baseline 字节一致。最终差异仅涉及两个已有包的 version/checksum。原始失败日志与 9 条既有告警完整保留。后续完整集成验收仍由 root 统一执行。

## 改动范围与指纹

| 锁文件 | Package 数前后 | Git 行差异 | 修改 |
| --- | --- | --- | --- |
| `Cargo.lock` | 491 / 491 | +4 / -4 | rustls 0.23.40 → 0.23.45；rustls-webpki 0.103.13 → 0.103.14 |
| `ccr-ui/src-tauri/Cargo.lock` | 678 / 678 | +4 / -4 | 同上 |

根锁锚点为 `Cargo.lock:3206`、`Cargo.lock:3228`；桌面锁锚点为 `ccr-ui/src-tauri/Cargo.lock:4399`、`ccr-ui/src-tauri/Cargo.lock:4421`。每个变更包只修改 version 与 checksum。Package 顺序、名称、source、依赖列表及其他 TOML 字段全部一致，没有新增 package。逐行位置比较与 Git numstat 独立确认每锁 4 行替换。

| 锁文件 | 修复前 SHA-256 | 修复后 SHA-256 |
| --- | --- | --- |
| 根 | `ca1f03e85b5e0d888109ac8af7e7943fc1ceecd20a0f6eeb6a60052c5155db79` | `22acbf9e8cad08d4a0653484681cc2ac3441f7c5f9357b1c824b8844d9dae1d3` |
| Tauri | `a21024dae2f7c909e24b70ee5296cd7ceb0821cab5c9f530a92fecaf818984d7` | `a211d3c584b853c3f038912bbfba10303d4d5f9505cfea5cda39d2a82926ae28` |

`HEAD = 34d8a85e0e48b793733835e0304c8ed33940fcee`。修复前字节同时匹配父 continuation-baseline、security-baseline、before.lock 副本和 Git HEAD；最终字节匹配实施者报告。项目 manifests、toolchain、audit config 与实施者前后采样一致。Tauri manifest 相对 Git HEAD 的既有架构任务差异不属于本安全切片。

独立执行的 `git diff --check -- Cargo.lock ccr-ui/src-tauri/Cargo.lock` 返回 0。Git 的 LF/CRLF 提示保留在 JSON，未转换产品文件换行。

## 官方公告与兼容性

已独立在线读取 RustSec 固定 commit `ef03605143a913024f864d2edf476adad5720c93` 的公告，以及 crates.io 两个精确版本 API。三个响应 SHA 均与实施者保存的对应原始文件一致。具体 URL、SHA、公告字段、版本字段和本地 registry 条目保存在同名 JSON。

`RUSTSEC-2026-0285` 日期为 2026-09-14，修复范围为 `>= 0.23.45`，不受影响范围为 `< 0.23.13`。旧锁 0.23.40 属于受影响范围；新锁 0.23.45 满足修复范围。新 rustls 对 webpki 的约束为 `^0.103.14`，最终选择 0.103.14 符合最小必要传递更新。两包均未撤回，官方 checksum 与两锁一致，声明 MSRV 均为 1.71。

所有受检查的 `crates/*/Cargo.toml` 与独立 Tauri manifest 仍声明 Rust 1.95；根 Cargo.toml 本身不声明 `rust-version`。开发工具链为 1.98.0。依赖版本声明与 Rust 1.98 编译结果不能替代实际 Rust 1.95 验收。

Feature 结论需要限定范围。项目 manifest 的 feature 声明和锁中依赖列表未变化。上游 rustls-webpki 0.103.14 的 `aws-lc-rs-unstable` 映射移除了 `aws-lc-rs/unstable`；两个上游包的可选 aws-lc-rs 约束由 `^1.14` 升至 `^1.18`；webpki 的开发依赖 base64 约束由 `^0.22` 升至 `^0.23`。这些上游定义变化已记录，两个项目锁均没有 aws-lc-rs package。不能将“本切片未改项目 feature”扩展为“上游全部 feature 语义不变”。

实施者保存的 root/Tauri `cargo tree --workspace --all-features --target all -i rustls@0.23.40` 均显示 `nothing to print`。该证据与锁中的可选依赖分别解释；本审查不声称生产 TLS 后端曾实际启用受影响 Rustls。

## 审计与失败证据

独立核验 16 条命令的 32 个 metadata/raw-log 文件，字段、SHA-256 和字节数全部匹配，差异数为 0。两个最终 audit 都采用默认 fetch，没有 `--no-fetch` 或 `--ignore`；原日志明确记录官方 advisory-db fetch 与 crates.io index update。`.cargo/audit.toml` 仍为 `ignore = []`，其 SHA 与安全修复前一致。

| 命令组 | 结果及边界 |
| --- | --- |
| 修复前 root / Tauri audit | 均 exit 1，记录 RUSTSEC-2026-0285；失败日志保留 |
| 首次在线 cargo update | exit 101；crates.io config.json 的 Schannel TLS 握手失败 |
| 后续精确离线更新 | 成功；没有修改证书验证或 advisory ignore |
| 最终 root audit | exit 0，491 packages，0 条 advisory warning |
| 最终 Tauri audit | exit 0，678 packages，保留 9 条 warning |
| version-check / dependency-governance | 原日志 exit 0；后者 6 项测试、19 个重复依赖检查 |
| ccr-sync / Tauri bin cargo check | 原日志 exit 0，仅当时窄范围编译检查 |

根锁更新曾先解析到 webpki 0.103.15，随后精确收窄至 0.103.14；中间状态和失败记录均未覆盖。另有两条 Linux 离线命令因缺少 webpki 0.103.14 缓存而 exit 101，其日志保留；后续在线定向命令成功。

| 保留的 Tauri 告警 | 版本 | 类别 | Advisory |
| --- | --- | --- | --- |
| fxhash | 0.2.1 | unmaintained | RUSTSEC-2025-0057 |
| proc-macro-error | 1.0.4 | unmaintained | RUSTSEC-2024-0370 |
| unic-char-property | 0.9.0 | unmaintained | RUSTSEC-2025-0081 |
| unic-char-range | 0.9.0 | unmaintained | RUSTSEC-2025-0075 |
| unic-common | 0.9.0 | unmaintained | RUSTSEC-2025-0080 |
| unic-ucd-ident | 0.9.0 | unmaintained | RUSTSEC-2025-0100 |
| unic-ucd-version | 0.9.0 | unmaintained | RUSTSEC-2025-0098 |
| glib | 0.18.5 | unsound | RUSTSEC-2024-0429 |
| rand | 0.7.3 | unsound | RUSTSEC-2026-0097 |

前后 9 条 warning 的 crate、version、类别和 ID 完全一致：7 条 unmaintained、2 条 unsound。默认 audit 返回 0 不表示全部告警已经消除。

## 检查时点与源码边界

两个锁的更新于 23:00:23 UTC 前完成；最终 audit 于 23:01:06 UTC 和 23:01:19 UTC 开始。窄范围编译于 23:01:53 UTC 和 23:02:10 UTC 开始。更新顺序、命令记录与当前最终锁 SHA 一致。安全命令 metadata 没有逐命令 lock SHA 或完整源码 SHA，不能重建共享工作区在每条命令开始时的全部字节。

Tauri bin check 于 23:02:10.454301 UTC 开始，23:02:22.156301 UTC 结束。实施者保存的 manifest 前后 SHA 与本审查采样匹配。原报告明确排除随后获授权的 handler fixture / Tauri test-feature 工作。当前源码采样仅证明审查时状态，不能替代执行时快照。root 仍需在全部授权改动整合后运行正式门禁。

## Linux 完整日志复核

`continuation-linux-process-online.log` 开头包含 `running 9 tests`、9 个唯一 selector 的 `ok`，以及 `9 passed; 0 failed; 0 ignored; 0 measured; 106 filtered out` 汇总。九条记录已逐项解析；输出没有缺失。

编译 stderr 位于日志后半，测试 stdout 位于开头。stdout/stderr 的显示顺序不保证事件时序，不能据此判断编译与测试的执行先后。全日志读取已经解决 tail-only 疑点，无需为该疑点重跑 Cargo、修改进程组实现或调整并行设置。

Rust 1.98 Codex Clippy 的完整日志成功结束，exit 0；process 定向日志为 9 通过。两个 metadata 的根锁 SHA 都匹配最终安全锁。`source_sha = bc10aa1cf8b17af5b49f337963a703b8ece68c63283ea8219f53ee42f2013cea` 对应 `crates/ccr-codex/src/services/codex_process_service.rs`，当前字节匹配。该字段不对应 `crates/ccr-core/src/core/process_gateway.rs`，不能解释为 core 测试源码的执行时冻结指纹。

两项定向结果有效，但未完成 Linux 全量 `just ci` / `tauri-ci`，也未替代设计要求的 macOS process-tree 验证。本节更新此前 continuation-acceptance-review 的 Linux 定向证据状态；其他验收边界保留。

## 交付与剩余工作

仅新增本 Markdown 和同名 JSON。未改源码、锁、manifest、规范、矩阵、父汇总或任务状态；未运行 Cargo、full CI、bindings 或产品测试；未操作真实账户；未提交、推送或归档。只读报告辅助脚本的编码、diff 计数及 CRLF 解析失败均已修正后重验，记录在 JSON 中，没有修改产品文件。

剩余工作为最终整合 root/Tauri/frontend 门禁、规划要求的 Linux/macOS 验证以及实际 Rust 1.95 MSRV 检查。9 条既有 Tauri 告警持续记录。独立性仅覆盖本次非作者 security 切片；本人先前编写的规范和矩阵不计为独立检查。本报告不宣称专用 trellis-check 成功。
