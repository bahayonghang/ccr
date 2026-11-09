# T10 后续 rustls 安全锁文件修复

日期：2026-09-28。实施者：`/root/implement_t10_gates`。授权来源：本任务 `research/continuation-scope.md` 与 root 的 security 切片派发。

## 结果

根 `Cargo.lock` 与独立桌面 `ccr-ui/src-tauri/Cargo.lock` 均已消除 `RUSTSEC-2026-0285` 对应的 rustls 0.23.40 锁定版本。两个最终 `cargo audit` 均 exit 0。Tauri 原有 9 条 advisory warning 保留；本报告不把默认 audit 通过解释为没有未解决告警。

每个锁文件只修改两个已有 package 的 version 与 checksum，各为 4 行新增、4 行删除。根依赖包数保持 491，Tauri 保持 678。package 名称、依赖列表、manifest、feature、平台、MSRV 与 audit ignore 配置均未由本切片修改。

| Package | 修复前 | 修复后 | 选择理由 |
| --- | --- | --- | --- |
| rustls | 0.23.40 | 0.23.45 | 官方公告给出的首个修复版本 |
| rustls-webpki | 0.103.13 | 0.103.14 | rustls 0.23.45 的最低兼容要求为 `^0.103.14` |

## 官方依据与兼容边界

本地 advisory-db remote 为 `https://github.com/RustSec/advisory-db.git`，前后 commit 均为 `ef03605143a913024f864d2edf476adad5720c93`。官方公告日期为 2026-09-14，修复范围为 `>= 0.23.45`，不受影响范围为 `< 0.23.13`。公告副本保存于 `continuation-security-advisory.md`。

已读取官方 RustSec 公告、crates.io 精确版本 API 与 dependencies API，并保存原始 JSON。rustls 0.23.45 与 rustls-webpki 0.103.14 均未撤回；两者声明的最低 Rust 版本为 1.71。当前项目 manifest 的 MSRV 仍为 1.95；本轮实际编译器为 1.98.0，没有执行 Rust 1.95 编译，不能把版本声明检查当作实际 MSRV 通过。

两个锁文件均记录 `reqwest 0.13.4 → hyper-rustls 0.27.9 → rustls`，以及经过 `tokio-rustls 0.26.4` 的路径。根侧的调用方示例为 `ccr-sync → reqwest_dav 0.3.3 → reqwest`。两个 workspace 的 `cargo tree --workspace --all-features --target all -i rustls@0.23.40` 均返回 `nothing to print`。该结果与保留在锁文件中的可选 Rustls 依赖区分记录，不据此声称当前生产 TLS 后端使用受影响 rustls。

## 精确改动与指纹

rustls checksum：

- 修复前：`ef86cd5876211988985292b91c96a8f2d298df24e75989a43a3c73f2d4d8168b`
- 修复后：`0d41d731c7d2f962d1ccc364cec258de3c0e93b38c2fb3ba97ac74513048d634`

rustls-webpki checksum：

- 修复前：`61c429a8649f110dddef65e2a5ad240f747e85f7758a6bccc7e5777bd33f756e`
- 修复后：`0527518605e68109d875e248ea259b6758801cf165e4b2c2733ae3b51f12535a`

| 锁文件 | 修复前 SHA-256 | 修复后 SHA-256 |
| --- | --- | --- |
| Cargo.lock | `ca1f03e85b5e0d888109ac8af7e7943fc1ceecd20a0f6eeb6a60052c5155db79` | `22acbf9e8cad08d4a0653484681cc2ac3441f7c5f9357b1c824b8844d9dae1d3` |
| ccr-ui/src-tauri/Cargo.lock | `a21024dae2f7c909e24b70ee5296cd7ceb0821cab5c9f530a92fecaf818984d7` | `a211d3c584b853c3f038912bbfba10303d4d5f9505cfea5cda39d2a82926ae28` |

根锁更新的临时中间版本 rustls-webpki 0.103.15 checksum 为 `f3c3cf1d8b1e7d4927e2d154c3fcb02979afb9939629c62cd9048d4f07b60ac2`，来自本地官方 registry index；最终已收窄为 0.103.14。JSON 报告保留该中间状态。

逐 package 比较断言：除上述四组 version/checksum 字段外，两锁解析内容完全相同。原锁文件副本为 `continuation-security-root-before.lock`、`continuation-security-tauri-before.lock`；最终 diff 为 `continuation-security-lock-diff.patch`。

## 执行记录

所有命令的完整参数、cwd、环境覆盖、开始时间、耗时、exit code、原始日志路径与 SHA 均记录于 `continuation-security-implementation.json`。以下日志均位于本目录，前缀为 `continuation-security-`。

| 命令 | 结果 | 原始日志后缀 |
| --- | --- | --- |
| `cargo audit --no-fetch --file Cargo.lock` | exit 1；rustls 1 项漏洞 | root-audit-before.log |
| `cargo audit --no-fetch --file ccr-ui/src-tauri/Cargo.lock` | exit 1；rustls 1 项漏洞及原有 9 条 warning | tauri-audit-before.log |
| `cargo update -p rustls@0.23.40 --precise 0.23.45` | exit 101；Schannel TLS 握手失败 | root-update-rustls.log |
| `cargo update --offline -p rustls@0.23.40 --precise 0.23.45` | exit 0；同时选择 webpki 0.103.15 | root-update-rustls-offline.log |
| `cargo update --offline -p rustls-webpki@0.103.15 --precise 0.103.14` | exit 0；收窄至最低必要版本 | root-update-webpki-minimum.log |
| `cargo update --manifest-path ccr-ui/src-tauri/Cargo.toml --offline -p rustls-webpki@0.103.13 --precise 0.103.14` | exit 0 | tauri-update-webpki-minimum.log |
| `cargo update --manifest-path ccr-ui/src-tauri/Cargo.toml --offline -p rustls@0.23.40 --precise 0.23.45` | exit 0 | tauri-update-rustls-offline.log |
| `cargo audit --file Cargo.lock` | exit 0；0 项漏洞、0 条 advisory warning | root-audit-after.log |
| `cargo audit --file ccr-ui/src-tauri/Cargo.lock` | exit 0；0 项漏洞、原有 9 条 warning | tauri-audit-after.log |
| `just version-check` | exit 0 | version-check.log |
| `just dependency-governance-check` | exit 0；6 项测试通过，检查 19 个重复依赖 | dependency-governance.log |
| `cargo check --locked --offline -p ccr-sync --all-targets --all-features` | exit 0；仅编译检查 | root-sync-check.log |
| `cargo --config .cargo/tauri-ci.toml check --locked --offline --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop` | exit 0；仅 bin 编译检查 | tauri-bin-check.log |

首次在线更新的失败原因是下载 crates.io `config.json` 时发生 Schannel SSL/TLS 握手错误。既有索引缓存已经包含所需版本，checksum 与独立读取的官方 API 一致，因此使用精确离线更新。没有修改索引缓存、证书验证或代理配置。两个最终审计均实际 fetch 官方 advisory-db 并更新索引成功。

Tauri 编译检查使用既有 `.cargo/tauri-ci.toml`，并仅为该子进程设置 `CCR_SKIP_ICON_GENERATION=1`。原始输出保留 linker 与 mbx 缓存警告；该命令不是 strict Clippy 或完整 Tauri 门禁。两锁的 `git diff --check` exit 0；Git 的 LF/CRLF 提示单独保留。

## 并行修改与剩余验证

Tauri bin 检查开始于 2026-09-28 23:02:10.454301 UTC，结束于 23:02:22.156301 UTC。编译完成后采样的 Tauri manifest SHA 为 `9c7c095139e2c85cd61e28471dd799c6fc79c0e677c3de539ee3004652974701`，与本切片开始时相同，采样时尚无 dev-dependency `tauri`。没有冻结整个共享工作区，因此本报告只记录该次命令的结果和时点。

root 随后通知：`implement_t09` 获准在 dev-dependencies 中启用与 normal 精确同版本的 Tauri `test` feature，并补充真实 Codex handler 文件往返测试。本切片没有修改该 manifest，也没有启动后续 Cargo 命令。上述 bin 检查不覆盖后续测试 feature、测试源码或 handler fixture 版本；root 负责整合后的验证。

Tauri 仍有 7 条 unmaintained warning：fxhash、proc-macro-error、unic-char-property、unic-char-range、unic-common、unic-ucd-ident、unic-ucd-version；另有 glib 与 rand 共 2 条 unsound warning。前后 warning 的 crate、version、类别与 advisory ID 相同，完整清单在 JSON 报告中。`.cargo/audit.toml` 的 `ignore = []` 未变，没有新增忽略或降低告警规则。

没有执行完整 `just ci`、完整 Tauri 测试、bindings/exports、前端完整门禁、Rust 1.95 编译或真实 TLS 漏洞复现。完整聚合由 root 统一调度。两份受保护 `.tmp` 文件在收尾采样时 SHA 未变；本切片未修改 Insights 状态、父汇总、原证据矩阵或 `render_plans.py`，未提交、推送或归档。
