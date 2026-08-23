# workspace Cargo 依赖全量升级

> 父任务：`08-22-react-migration`

## Goal

将根 workspace 的共享 Cargo 依赖升级到最新兼容版，覆盖 13 个 crate，并处理 `ts-rs` 11 → 12 升级对 204 个 TypeScript 类型绑定的影响。

## Scope

### workspace 成员（13 个 crate）

`ccr`、`ccr-core`、`ccr-config`、`ccr-sync`、`ccr-skills`、`ccr-store`、`ccr-codex`、`ccr-db`、`ccr-checkin`、`ccr-cli`、`ccr-tui`、`ccr-usage`、`ccr-types`。

resolver 为 `3`，`default-members = ["crates/ccr"]`。

### 共享依赖当前版本

| 分类 | 依赖 | 当前版本 |
|---|---|---|
| 异步运行时 | `tokio` | 1.52.3 |
| CLI | `clap` | 4.6.1 |
| 错误处理 | `anyhow` / `thiserror` | 1.0.104 / 2.0.18 |
| HTTP | `reqwest` | 0.13.4（`default-features = false`，`native-tls`） |
| 序列化 | `serde` / `serde_json` / `toml` | 1.0.228 / 1.0.150 / 1.1.2 |
| TS 绑定 | `ts-rs` | 11 |
| 时间 | `chrono` / `chrono-tz` / `iana-time-zone` | 0.4.44 / 0.10.4 / 0.1.65 |
| 工具 | `dirs` / `tempfile` | 6.0.0 / 3.27.0 |
| 数据库 | `rusqlite` / `r2d2` / `r2d2_sqlite` | 0.39.0（`bundled`） / 0.8 / 0.34.0 |
| 日志 | `tracing` | 0.1.43 |
| 加密与身份 | `uuid` / `aes-gcm` / `argon2` / `rand` / `sha2` / `base64` / `blake3` / `rpassword` | 1.23.2 / 0.10 / 0.5 / 0.10.1 / 0.11.0 / 0.22.1 / 1.8.5 / 7.5.3 |
| 其他 | `indexmap` / `once_cell` / `walkdir` / `futures` / `async-stream` / `tokio-util` / `open` | 2.14.0 / 1.21.4 / 2.5 / 0.3.32 / 0.3 / 0.7 / 5.4.0 |
| Web | `axum` / `tower` / `tower-http` | 0.8.6 / 0.5.3 / 0.6.11 |

### origin 上已有的 dependabot 分支

`anyhow` 1.0.104、`clap` 4.6.4、`thiserror` 2.0.19、`sysinfo` 0.39.6、`serde_json` 1.0.151、`rpassword` 7.5.4、`ts-rs` 12.0.1。可作为升级起点，逐个复核后合入本任务而非单独合并。

### ts-rs 11 → 12

`ts-rs` 仅由 `ccr-usage` 的 `ts` feature 与 `ccr-ui/src-tauri` 使用，CLI 默认依赖图不含。升级后需重新生成 `ccr-ui/src/types/generated/` 下 204 个文件（904 行），逐条比对差异。

## Requirements

- R1 共享依赖升级到最新兼容版。不兼容的升级项逐个登记，说明保留当前版本的原因。
- R2 `reqwest` 的 `default-features = false` 与 `native-tls` 配置保留。禁止改为 rustls + aws-lc-rs（需 cmake / C 构建，编译时间显著增加）。
- R3 `reqwest` 的 `http2` feature 保留。签到请求通过 ALPN 协商 HTTP/2，该行为不得回退。
- R4 `rusqlite` 的 `bundled` 与 `functions` feature 保留。
- R5 加密相关依赖（`aes-gcm`、`argon2`、`blake3`、`sha2`、`rand`）升级需逐个确认无算法参数或 KDF 行为变化。验证方式为**按三种真实持久格式分别**用旧版本生成样本、新版本读回：Codex auth 导出（`crates/ccr-codex`，Argon2id + salt + KDF 参数 + AAD）、Sync 信封 V2（`crates/ccr-sync`，含 `PlaintextV1` 旧读路径）、CheckIn 凭据（`crates/ccr-checkin`，随机 key + `nonce || ciphertext`，无 KDF）。通用向量不构成验证。`blake3` 与 `sha2` 不参与解密，其验证面是内容哈希与去重的摘要稳定性。
- R6 `ts-rs` 升级到 12.x，重新生成 204 个类型文件，差异逐条判定。
- R7 `crates/ccr-usage` 的 SQLite 投影 SQL 与 schema 版本门（`MIN_SUPPORTED_SCHEMA_VERSION`、provider schema 14）不因依赖升级而改变行为。
- R8 `llmusage_no_crate_guard` 测试继续通过，不引入上游 llmusage Rust crate 依赖。
- R9 `resolver = "3"` 与 `[profile.dev]` / `[profile.test]` 配置不变。
- R10 `workspace.package.version` 不变（7.2.0）。版本变更走 `just version-sync`，不在本任务范围。

## Acceptance Criteria

- [ ] AC1 `just check-workspace` 退出码 0。
- [ ] AC2 `just lint-strict` 退出码 0。
- [ ] AC3 `just test` 退出码 0（Rust 测试直接运行时带 `-- --test-threads=1`）。
- [ ] AC4 `just release` 退出码 0。
- [ ] AC5 `just audit` 退出码 0，无新增高危项。
- [ ] AC6 `just version-check` 退出码 0。
- [ ] AC7 升级清单落盘：每个依赖的升级前后版本、是否升级、未升级原因，无未判定项。
- [ ] AC8 `ts-rs` 重新生成的 204 个文件 diff 逐条判定，判定记录落盘。
- [ ] AC9 加密依赖的行为变化核查记录落盘。三种持久格式各有升级前样本与升级后读回结果；`blake3` / `sha2` 的固定输入摘要升级前后一致。
- [ ] AC10 `reqwest` 配置核查：`default-features = false`、`native-tls`、`http2` 三项在升级后仍存在。
- [ ] AC11 HTTP/2 与 Cookie 行为按 `design.md` §3b 的四个观测点核对，升级前基线与升级后数据均落盘。协议版本项取 `response.version()`，不以「签到请求成功」代替。Cookie 接收项用本地 mock server 验 `cookie_store` 的 jar 行为，无外部前置条件。无真实账号或服务端不可达时，协议版本与 ALPN 两项标「未执行」并说明，不标「通过」。
- [ ] AC12 `llmusage_no_crate_guard` 测试通过。
- [ ] AC13 编译时间与二进制体积的升级前后对比记录落盘。

## 前置与后续

- 前置：`08-22-dep-upgrade`。两者的 `ts-rs` 升级需协同：本任务升级 Rust 侧并重新生成，`08-22-dep-upgrade` 的 R7 与 AC7 覆盖前端侧 204 个文件的 diff 判定。两个任务对同一批文件负责，实施时由本任务执行生成，`08-22-dep-upgrade` 执行验证。
- 后续：`08-22-design-system`（与 `08-22-dep-upgrade` 并列作为前置）。

## Out of Scope

- `crates/` 下任何 crate 的功能改动与重构。
- Rust edition 与 MSRV 变更。当前为 Edition 2024 / Rust 1.88+。
- `ccr-ui/src-tauri/Cargo.toml` 的依赖升级（属 `08-22-dep-upgrade`）。
- `ccr-vscode` 依赖。
- 版本号变更。
- 引入上游 llmusage Rust crate。该行为由 `llmusage_no_crate_guard` 测试禁止。

## Notes

- 本任务与前端迁移无技术依赖，可与 `08-22-react-foundation` 并行执行，不占用迁移主线的关键路径。
- origin 上 7 个相关 dependabot 分支不单独合并，统一在本任务内处理，避免迁移分支与 `dev` 产生多点冲突。
- 加密依赖升级是本任务的主要风险。**现状更正**：加密调用点不在 `crates/ccr-config` 与 `crates/ccr-db`，实为 `crates/ccr-codex`（Codex auth 导出）、`crates/ccr-sync`（同步信封 V2）、`crates/ccr-checkin`（凭据）。`ccr-db` 只用 `sha2` 做导入去重。三处的持久格式互不相同，验证需按格式分别做，见 `design.md` §3。
- `ts-rs` 升级同时影响 Rust 与前端两侧，需要两个任务协同。建议在两个任务的 `implement.md` 中写入同一个协同检查点。
