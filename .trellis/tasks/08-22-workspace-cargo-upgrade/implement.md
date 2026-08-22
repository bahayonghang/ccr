# 执行计划：workspace Cargo 依赖全量升级

> 父任务：`08-22-react-migration`（旁路子任务 2b）。可与阶段 1–3 任一并行，不占迁移主线关键路径。
> 分支：`feature/react-migration/workspace-cargo-upgrade`。**PR 目标为 `dev`，不经迁移分支**（父任务 `implement.md` §5 的例外条款）。合入 `dev` 后 rebase 到 `feature/react-migration`。

## 前置确认

- [ ] `just ci` 在 `dev` 上全绿（父任务基线采集门的一项，基线 13 步）。
- [ ] `git checkout -b feature/react-migration/workspace-cargo-upgrade dev`
- [ ] 记录升级前基线：`cargo clean && cargo build --release` 计时、`crates/ccr` release 二进制体积。
- [ ] `cargo upgrade --dry-run`（或 `cargo outdated`）输出落盘，作为 `upgrade-inventory.md` 的目标版本依据。
- [ ] origin 上 7 个 dependabot 分支的目标版本记录：`anyhow` 1.0.104、`clap` 4.6.4、`thiserror` 2.0.19、`sysinfo` 0.39.6、`serde_json` 1.0.151、`rpassword` 7.5.4、`ts-rs` 12.0.1。这 7 个分支不单独合并。

## 组 C 的前置：加密测试向量（按格式分别固化）

在任何加密依赖升级之前执行，否则升级后无对比基准。按 `design.md` §3.1 的三种持久格式各做一份，通用向量不算完成。

- [ ] **Codex auth 导出格式**（`crates/ccr-codex/src/services/codex_auth_crypto.rs`）：用已知测试口令 + 已知测试明文生成完整导出文件，含其真实 salt、`KdfParams{m_cost,t_cost,p_cost}`、version、format、AAD 输入值。
- [ ] **Sync 信封 V2**（`crates/ccr-sync/src/sync/envelope.rs`）：生成完整 `EncryptedEnvelopeV2` 字节（magic / version / algorithm / kdf / kdf_params / salt / nonce / metadata），另生成一个 `PlaintextV1` 样本覆盖旧读路径。
- [ ] **CheckIn 凭据**（`crates/ccr-checkin/src/core/crypto.rs`）：生成 key 文件（base64 随机 32 字节）+ `base64(nonce || ciphertext)` 密文串。该格式无 KDF，key 来源是随机而非口令派生。
- [ ] **摘要向量**：`blake3` 与 `sha2` 各固定一组输入 → 摘要。使用位置见 `design.md` §3.1 的第二张表。
- [ ] 确认全部向量可被当前版本自身正确读回（自检，排除向量本身写错）。
- [ ] 向量不含真实凭据。

## 组 B 的前置：HTTP/2 与 Cookie 基线

- [ ] 按 `design.md` §3b 采升级前基线：`response.version()`（带临时 instrumentation，不提交）、`Cookie` 请求头（现有离线 header 测试）、本地 mock server 的 `Set-Cookie` 回发行为。
- [ ] 无真实账号时，协议版本与 ALPN 两项标「未执行」，Cookie 两项仍须执行。

## 组 A：patch 级升级

- [ ] `anyhow`、`thiserror`、`clap`、`serde` / `serde_json`、`rpassword`、`tracing`、`uuid`、`indexmap`、`once_cell` 等 patch 级条目升级。
- [ ] 一次提交。

验证：`just check-workspace`、`just lint-strict`、`just test`。

## 组 B：minor 级升级

按依赖分组提交，每组提交后验证。

- [ ] `tokio` / `tokio-util` / `futures` / `async-stream`。
- [ ] `reqwest`。**升级后立即核对三项配置**：`default-features = false`、`native-tls`、`http2`（AC10）。再按 `design.md` §3b 复采四个观测点，与「组 B 的前置」基线对比（AC11）。
- [ ] `toml`、`chrono` / `chrono-tz` / `iana-time-zone`。
- [ ] `rusqlite` / `r2d2` / `r2d2_sqlite`。核对 `bundled` 与 `functions` feature 保留。
- [ ] `axum` / `tower` / `tower-http`。
- [ ] `dirs`、`tempfile`、`walkdir`、`open`、`sysinfo`。

验证（每组后）：`just check-workspace`、`just test`。`rusqlite` 组额外跑 `crates/ccr-usage` 的投影相关测试与 `llmusage_no_crate_guard`（AC12）。

## 组 C：加密与身份

每个依赖单独提交。顺序：`base64` → `sha2` → `blake3` → `rand` → `argon2` → `aes-gcm`（风险递增）。

- [ ] 每个依赖升级后，用新版本读回组 C 前置产出的**全部**向量（三种持久格式 + 摘要向量），验证解密与摘要一致（`design.md` §3.1）。
- [ ] `rand` 的验证点为 API 兼容与未改用弱生成器（`design.md` §3.1 末段）。
- [ ] 任一格式的向量不匹配，该依赖保留当前版本，原因写入 `upgrade-inventory.md`。
- [ ] `crypto-compat-check.md` 落盘，逐格式列结果（AC9）。

验证：`just test`、`just secret-write-check`。

## 组 D：ts-rs 11 → 12

- [ ] 与 `08-22-dep-upgrade` 确认 `src-tauri` 侧同步升到同一 12.x 版本（协同点 A）。版本不一致则不执行生成。
- [ ] 升级 workspace 的 `ts-rs` 声明（`crates/ccr-usage`、`crates/ccr-cli` 的 `ts` feature 路径）。
- [ ] `cd ccr-ui && just bindings` 生成 204 个文件。
- [ ] 通知 `08-22-dep-upgrade` 执行 diff 逐条判定（其 R7 / AC7）。本任务提交生成产物，不做判定。
- [ ] 一次提交，含 Rust 侧版本变更与生成产物。

验证：`just tauri-bindings-check` 退出码 0；`cd ccr-ui/src-tauri && cargo check`。

## 收尾

- [ ] `upgrade-inventory.md` 落盘，覆盖 `[workspace.dependencies]` 全部条目，无空缺（AC7）。
- [ ] `cargo clean && cargo build --release` 计时与二进制体积记录，与前置基线对比（AC13）。
- [ ] HTTP/2 与 Cookie 的四个观测点数据与基线对比落盘（AC11）。协议版本项取 `response.version()`，不以「签到请求成功」代替。无真实账号时该项标「未执行」并说明。
- [ ] `just version-check` 确认 `workspace.package.version` 仍为 7.2.0（AC6、R10）。

## 验证命令

| 时机      | 命令                                                                                                                        |
| --------- | --------------------------------------------------------------------------------------------------------------------------- |
| 每组后    | `just check-workspace`、`just test`                                                                                         |
| 组 A–C 后 | `just lint-strict`                                                                                                          |
| 组 C 后   | `just secret-write-check`                                                                                                   |
| 组 D 后   | `just tauri-bindings-check`                                                                                                 |
| 交付前    | `just check-workspace` → `just lint-strict` → `just test` → `just release` → `just audit` → `just version-check`（AC1–AC6） |

Rust 测试若绕过 `just test` 直接运行，带 `-- --test-threads=1`。

## 交付门

- [ ] AC1–AC13 全部满足。
- [ ] 三份记录落盘：`upgrade-inventory.md`、`crypto-compat-check.md`、编译时间与体积对比。
- [ ] `reqwest` 三项配置核对记录落盘（AC10）。
- [ ] 签到真实请求验证通过（AC11）。

## 回滚点

| 组  | 回滚方式                                          |
| --- | ------------------------------------------------- |
| A   | 单次 revert                                       |
| B   | 按依赖分组 revert，可只回退某一组                 |
| C   | 每依赖单独 revert，可精确回退到出问题的那一个     |
| D   | 单次 revert，同时恢复 204 个生成文件与 Cargo.lock |

本任务的回滚独立于前端迁移。已合入 `dev` 后的回滚为 revert merge commit。

## 协同点

| 编号 | 内容                                              | 对方                | 时机   |
| ---- | ------------------------------------------------- | ------------------- | ------ |
| A    | `ts-rs` 版本号对齐；本任务执行生成，对方判定 diff | `08-22-dep-upgrade` | 组 D   |
| —    | 合入 `dev` 后 rebase 到 `feature/react-migration` | 父任务              | 交付后 |
