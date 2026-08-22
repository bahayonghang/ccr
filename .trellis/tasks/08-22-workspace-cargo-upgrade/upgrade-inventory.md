# 升级清单：workspace Cargo 依赖全量升级

> 方法说明：本机无 cargo-edit（`cargo upgrade` 不可用）。采用回退方法：以 crates.io 查询（2026-08-23）的各依赖最新稳定版为目标，编辑根 `Cargo.toml` / `ccr-ui/src-tauri/Cargo.toml` 的 caret 版本要求后，按包执行 `cargo update -p <pkg>` 刷新 Cargo.lock。dependabot 分支目标版本（anyhow 1.0.104、clap 4.6.4、thiserror 2.0.19、serde_json 1.0.151、rpassword 7.5.4、ts-rs 12.0.1、sysinfo 0.39.6）作为起点复核。

## 基线（升级前）

| 项目 | 数值 |
|---|---|
| `cargo clean && cargo build --release` | 250 s，exit 0 |
| `crates/ccr` release 二进制 | target/release/ccr.exe = 18,902,016 bytes |
| `ccr-ui/src/types/generated` | 19 顶层条目，聚合 md5 = 7e3e7e09865e899ee882d1dc13fba6e7，git 状态干净 |

## 组 C 前置：加密测试向量基线（升级前自检，全部通过）

| 向量 | 位置 | 结果 |
|---|---|---|
| Codex auth 导出（Argon2id+KdfParams+salt+AAD+AES-256-GCM） | `crates/ccr-codex/tests/tmp_crypto_vector.rs`（临时，不提交） | exit 0 |
| Sync 信封 V2 + PlaintextV1 旧读路径 | `crates/ccr-sync/src/sync/envelope.rs` 临时测试模块（不提交） | exit 0 |
| CheckIn 凭据（随机 key + base64(nonce‖ciphertext)） | `crates/ccr-checkin/tests/tmp_checkin_crypto_vector.rs`（临时，不提交） | exit 0 |
| blake3 固定输入摘要 ×3 | `crates/ccr-core/tests/tmp_digest_blake3.rs`（临时，不提交） | exit 0，基线已落盘 `%TEMP%/ccr-crypto-vectors/blake3_digests.txt` |
| sha256 固定输入摘要 ×3 | `crates/ccr-db/tests/tmp_digest_sha2.rs`（临时，不提交） | exit 0，基线已落盘 `%TEMP%/ccr-crypto-vectors/sha256_digests.txt` |

向量不含真实凭据：口令 `ccr-test-passphrase`，明文均为固定测试字符串。

## 组 B 前置：HTTP/2 与 Cookie 基线（升级前）

| 观测点 | 结果 |
|---|---|
| Cookie 发送（现有离线 header 测试） | 由 `just test` 覆盖（test_checkin_request_browser_fingerprint_headers），升级前通过 |
| Cookie 接收（本地 mock server，Set-Cookie 回发） | PASS：第二次请求带回 `ccrvec=vectorvalue42`（`crates/ccr-checkin/tests/tmp_cookie_h2.rs`，临时，不提交） |
| 协议版本 `response.version()`（同配置客户端，公开 h2 端点 google generate_204） | `HTTP/2.0` |
| ALPN / 真实账号签到 | **未执行**：无真实签到账号；以公开端点协议版本观测替代，不以「签到成功」代替 |

## 升级清单（[workspace.dependencies] 全部条目）

| 依赖 | 当前 | 目标 | 是否升级 | 未升级原因 | 组 | 核对要点 |
|---|---|---|---|---|---|---|
| tokio | 1.52.3 | 1.53.1 | 待定 | | B1 | 异步运行时行为 |
| tokio-util | 0.7 | 0.7.19 | 待定 | | B1 | rt feature 保留 |
| futures | 0.3.32 | 0.3.34 | 待定 | | B1 | |
| async-stream | 0.3 | 0.3.6 | 待定 | | B1 | |
| clap | 4.6.1 | 4.6.6 | 待定 | | A | |
| anyhow | 1.0.104 | 1.0.104 | 否 | 已是最新稳定版 | A | — |
| thiserror | 2.0.18 | 2.0.20 | 待定 | | A | |
| reqwest | 0.13.4 | 0.13.4 | 否 | 已是最新稳定版 | B2 | default-features=false + native-tls + http2 三项核对（AC10）；观测点复采（AC11） |
| serde | 1.0.228 | 1.0.229 | 待定 | | A | derive 行为 |
| serde_json | 1.0.150 | 1.0.151 | 待定 | | A | |
| toml | 1.1.2 | 1.1.4 | 待定 | | B3 | |
| ts-rs | 11 | 12.0.1 | 待定 | | D | 三处声明对齐 + 204 文件再生成 |
| chrono | 0.4.44 | 0.4.45 | 待定 | | B3 | serde feature |
| chrono-tz | 0.10.4 | 0.10.4 | 否 | 已是最新稳定版 | B3 | — |
| iana-time-zone | 0.1.65 | 0.1.65 | 否 | 已是最新稳定版 | B3 | — |
| dirs | 6.0.0 | 6.0.0 | 否 | 已是最新稳定版 | B6 | — |
| tempfile | 3.27.0 | 3.27.0 | 否 | 已是最新稳定版 | B6 | — |
| rusqlite | 0.39.0 | 0.40.2 | 待定 | | B4 | bundled + functions 保留（R4）；ccr-usage 投影 SQL 无影响（R7） |
| r2d2 | 0.8 | 0.8.10 | 待定 | | B4 | |
| r2d2_sqlite | 0.34.0 | 0.35.0 | 待定 | | B4 | 需与 rusqlite 0.40 匹配 |
| tracing | 0.1.43 | 0.1.44 | 待定 | | A | |
| uuid | 1.23.2 | 1.25.0 | 待定 | | A | serde + v4 feature |
| aes-gcm | 0.10 | 0.11.1 | 待定 | | C | 三格式向量读回；不匹配则保留 0.10 |
| argon2 | 0.5 | 0.5.3 | 待定 | | C | Argon2id V0x13 参数不变；向量读回 |
| rand | 0.10.1 | 0.10.2 | 待定 | | C | API 兼容 + 未改弱生成器 |
| sha2 | 0.11.0 | 0.11.0 | 否 | 已是最新稳定版 | C | 摘要向量已固化，随组 C 复验 |
| base64 | 0.22.1 | 0.23.1 | 待定 | | C | 跨 0.x 大版；向量读回；不匹配则保留 0.22 |
| blake3 | 1.8.5 | 1.8.7 | 待定 | | C | 摘要向量读回 |
| rpassword | 7.5.3 | 7.5.4 | 待定 | | A | |
| indexmap | 2.14.0 | 2.14.0 | 否 | 已是最新稳定版 | A | — |
| once_cell | 1.21.4 | 1.21.4 | 否 | 已是最新稳定版 | A | — |
| walkdir | 2.5 | 2.5 | 否 | 已是最新稳定版 | B6 | — |
| open | 5.4.0 | 5.4.1 | 待定 | | B6 | |
| axum | 0.8.6 | 0.8.9 | 待定 | | B5 | default-features=false + 现有 feature 保留 |
| tower | 0.5.3 | 0.5.3 | 否 | 已是最新稳定版 | B5 | — |
| tower-http | 0.6.11 | 0.7.0 | 待定 | | B5 | 跨 0.x 大版；cors feature；与 axum 0.8 兼容性核对 |

## 段 3：ccr-ui/src-tauri（与组 D 同一提交）

| 依赖 | 当前 | 目标 | 是否升级 | 说明 |
|---|---|---|---|---|
| ts-rs | 11 | 12.0.1 | 待定 | 协同点 A：与 workspace 两侧同升 12.0.1 后再生成 |
| serde_json | 1.0.150 | 1.0.151 | 待定 | |
| sysinfo | 0.39.3 | 0.39.6 | 待定 | dependabot 目标一致 |
| async-trait | 0.1 | 0.1.92 | 待定 | |
| lru | 0.18.2 | 0.18.2 | 否 | 已是最新稳定版 |
| 其余 caret 兼容条目 | — | — | 待定 | 以 src-tauri workspace `cargo update --dry-run` 结果为准，tauri 精确 pin（=2.11.2 / =2.6.2）不动 |

## 收尾数据

（编译时间/体积对比、观测点复采、版本核对：收尾时填写）
