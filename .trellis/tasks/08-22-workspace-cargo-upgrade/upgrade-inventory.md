# 升级清单：workspace Cargo 依赖全量升级

> 方法说明：本机无 cargo-edit（`cargo upgrade` 不可用）。采用回退方法：以 crates.io 查询（2026-08-23）的各依赖最新稳定版为目标，编辑根 `Cargo.toml` / `ccr-ui/src-tauri/Cargo.toml` 的 caret 版本要求后，按包执行 `cargo update -p <pkg>` 刷新 Cargo.lock。dependabot 分支目标版本（anyhow 1.0.104、clap 4.6.4、thiserror 2.0.19、serde_json 1.0.151、rpassword 7.5.4、ts-rs 12.0.1、sysinfo 0.39.6）作为起点复核，全部被本次目标覆盖或持平。
> 状态标签：已升级 / 无变化（已是最新稳定版）/ 保留（有目标但未升级）。

## 基线与收尾对比（AC13）

| 项目 | 升级前 | 升级后 |
|---|---|---|
| `cargo clean && cargo build --release` | 250 s，exit 0 | 见下方「最终验证数据」（收尾实测） |
| `crates/ccr` release 二进制 | 18,902,016 bytes | 同上 |
| `ccr-ui/src/types/generated` | 聚合 md5 = 7e3e7e09865e899ee882d1dc13fba6e7 | 重生成后 204 文件中 8 个变化（ts-rs 12 输出格式），`just tauri-bindings-check` exit 0 |

## 组 C 前置：加密测试向量基线（升级前自检，全部通过）

| 向量 | 位置 | 结果 |
|---|---|---|
| Codex auth 导出（Argon2id+KdfParams+salt+AAD+AES-256-GCM） | `crates/ccr-codex/tests/tmp_crypto_vector.rs`（临时，已删除未提交） | exit 0 |
| Sync 信封 V2 + PlaintextV1 旧读路径 | `crates/ccr-sync/src/sync/envelope.rs` 临时测试模块（已删除未提交） | exit 0 |
| CheckIn 凭据（随机 key + base64(nonce‖ciphertext)） | `crates/ccr-checkin/tests/tmp_checkin_crypto_vector.rs`（临时，已删除未提交） | exit 0 |
| blake3 固定输入摘要 ×3 | `crates/ccr-core/tests/tmp_digest_blake3.rs`（临时，已删除未提交） | exit 0，基线 `%TEMP%/ccr-crypto-vectors/blake3_digests.txt` |
| sha256 固定输入摘要 ×3 | `crates/ccr-db/tests/tmp_digest_sha2.rs`（临时，已删除未提交） | exit 0，基线 `%TEMP%/ccr-crypto-vectors/sha256_digests.txt` |

向量不含真实凭据：口令 `ccr-test-passphrase`，明文均为固定测试字符串。逐依赖读回结果见 `crypto-compat-check.md`。

## 组 B 前置与复采：HTTP/2 与 Cookie 观测点（AC11）

| 观测点 | 升级前基线 | 组 B 后复采 | 结论 |
|---|---|---|---|
| Cookie 发送（现有离线 header 测试） | 通过 | 通过（每次 `just test` 均含） | 不变 |
| Cookie 接收（本地 mock server Set-Cookie 回发） | PASS：第二次请求带回 `ccrvec=vectorvalue42` | PASS：同一断言通过 | jar 行为不变 |
| 协议版本 `response.version()`（同配置客户端访问公开 h2 端点 google generate_204） | `HTTP/2.0`（status 204） | `HTTP/2.0`（status 204） | 协商行为不变；reqwest 版本本身未变（0.13.4 已是最新稳定版） |
| ALPN / 真实账号签到 | **未执行**：无真实签到账号 | 未执行（同因） | 以公开端点协议版本观测替代，不以「签到成功」代替 |

### AC10 reqwest 配置核对（升级后）

根 `Cargo.toml` L48-54 实测保留三项：`default-features = false`、features 含 `"native-tls"`、features 含 `"http2"`。src-tauri 侧同名配置同样保留。编译期守卫 `test_http2_feature_enabled` 随 `just test` 通过。

## 升级清单（[workspace.dependencies] 全部条目，36 行无空缺）

| 依赖 | 当前 | 目标 | 是否升级 | 未升级原因 | 组 | 核对要点 |
|---|---|---|---|---|---|---|
| tokio | 1.52.3 | 1.53.1 | 已升级 | | B1 | 异步运行时行为，测试全过 |
| tokio-util | 0.7 | 0.7.19 | 已升级 | | B1 | rt feature 保留 |
| futures | 0.3.32 | 0.3.34 | 已升级 | | B1 | |
| async-stream | 0.3 | 0.3.6 | 已升级 | 仅声明面：声明于 workspace.dependencies 但无成员引用 | B1 | |
| clap | 4.6.1 | 4.6.6 | 已升级 | | A | |
| anyhow | 1.0.104 | 1.0.104 | 无变化 | 已是最新稳定版 | A | — |
| thiserror | 2.0.18 | 2.0.20 | 已升级 | | A | |
| reqwest | 0.13.4 | 0.13.4 | 无变化 | 已是最新稳定版 | B2 | 三项配置核对见上（AC10）；观测点复采一致（AC11） |
| serde | 1.0.228 | 1.0.229 | 已升级 | | A | derive 行为，测试全过 |
| serde_json | 1.0.150 | 1.0.151 | 已升级 | | A | |
| toml | 1.1.2 | 1.1.4 | 已升级 | | B3 | lockfile 解析为 1.1.4+spec-1.1.0 |
| ts-rs | 11 | 12.0.1 | 已升级 | | D | 双侧对齐 + 全量重生成，见组 D 记录 |
| chrono | 0.4.44 | 0.4.45 | 已升级 | | B3 | serde feature 保留 |
| chrono-tz | 0.10.4 | 0.10.4 | 无变化 | 已是最新稳定版 | B3 | — |
| iana-time-zone | 0.1.65 | 0.1.65 | 无变化 | 已是最新稳定版 | B3 | — |
| dirs | 6.0.0 | 6.0.0 | 无变化 | 已是最新稳定版 | B6 | — |
| tempfile | 3.27.0 | 3.27.0 | 无变化 | 已是最新稳定版 | B6 | — |
| rusqlite | 0.39.0 | 0.40.2 | 已升级 | | B4 | bundled + functions 保留（R4）；ccr-usage 投影测试 45/45 过（R7）；MIN_SUPPORTED_SCHEMA_VERSION 行为不变 |
| r2d2 | 0.8 | 0.8.10 | 已升级 | | B4 | |
| r2d2_sqlite | 0.34.0 | 0.35.0 | 已升级 | | B4 | 与 rusqlite 0.40 匹配（crates.io 元数据核对 req=^0.40） |
| tracing | 0.1.43 | 0.1.44 | 已升级 | | A | |
| uuid | 1.23.2 | 1.25.0 | 已升级 | | A | serde + v4 feature 保留 |
| aes-gcm | 0.10 | 0.11.1 | **保留** | 0.11 基于 aead 0.6 移除 `aes_gcm::aead::OsRng` 再导出，升级需改 crates 源码导入路径，违反 C3（源文件零改动）；按加密规则保留当前版本，详见 crypto-compat-check.md | C | lockfile 实际解析 0.10.3；三格式向量读回全过 |
| argon2 | 0.5 | 0.5.3 | 已升级 | | C | Argon2id V0x13 参数不变；向量读回通过 |
| rand | 0.10.1 | 0.10.2 | 已升级 | | C | 调用点全 OsRng（CSPRNG），无弱生成器；向量读回通过 |
| sha2 | 0.11.0 | 0.11.0 | 无变化 | 已是最新稳定版 | C | 摘要向量固化后随组 C 复验一致 |
| base64 | 0.22.1 | 0.23.1 | 已升级 | | C | 跨 0.x 大版；三格式向量读回全过；lockfile 中 0.22 为传递依赖独立槽位 |
| blake3 | 1.8.5 | 1.8.7 | 已升级 | | C | 固定输入摘要前后一致 |
| rpassword | 7.5.3 | 7.5.4 | 已升级 | | A | dependabot 目标一致 |
| indexmap | 2.14.0 | 2.14.0 | 无变化 | 已是最新稳定版 | A | — |
| once_cell | 1.21.4 | 1.21.4 | 无变化 | 已是最新稳定版 | A | — |
| walkdir | 2.5 | 2.5 | 无变化 | 已是最新稳定版 | B6 | — |
| open | 5.4.0 | 5.4.1 | 已升级 | | B6 | |
| axum | 0.8.6 | 0.8.9 | 已升级 | 仅声明面：无成员引用，无 lockfile 变化 | B5 | default-features=false + 现有 feature 保留 |
| tower | 0.5.3 | 0.5.3 | 无变化 | 已是最新稳定版 | B5 | — |
| tower-http | 0.6.11 | 0.7.0 | 已升级 | 仅声明面：无成员引用，无 lockfile 变化 | B5 | cors feature 保留；与 axum 0.8/tower 0.5/http1 兼容性经 crates.io 元数据核对 |

## 段 3：ccr-ui/src-tauri（与组 D 同一提交）

| 依赖 | 当前 | 目标 | 是否升级 | 说明 |
|---|---|---|---|---|
| ts-rs | 11 | 12.0.1 | 已升级 | 协同点 A 完成：双侧同升 12.0.1 后执行生成 |
| serde_json | 1.0.150 | 1.0.151 | 已升级 | |
| sysinfo | 0.39.3 | 0.39.6 | 已升级 | dependabot 目标一致 |
| async-trait | 0.1 | 0.1.92 | 已升级 | |
| lru | 0.18.2 | 0.18.2 | 无变化 | 已是最新稳定版 |
| 其余 caret 兼容条目 | — | — | 已随 update 刷新 | src-tauri Cargo.lock 在 rusqlite links 对齐（B4 提交）与本提交两次刷新中升至 caret 兼容最新；tauri 精确 pin（=2.11.2 / tauri-build =2.6.2）未动 |

## 组 D 执行记录

- `rm -rf ccr-ui/src/types/generated` 后经 `just bindings`（三条 cargo test + normalize 脚本）全量重生成。
- `git diff --stat` 汇总：**18 files changed, 56 insertions(+), 57 deletions(-)**（含两侧 Cargo.toml/Cargo.lock）；生成文件部分 **8 files changed, 8 insertions(+), 8 deletions(-)**，单一模式：mapped type `{ [key in string]?: T }` → `{ [key in string]: T }`（CodexJsonValue / OpenJsonValueDto / events JsonValueDto / GrokSettingsPatchDto.set / CliVersionsResponse.versions / CapabilityReport.features / HeatmapResponseDto / HomeUsageOverviewResponse）。逐条 diff 判定归 `08-22-dep-upgrade` R7/AC7（跟进任务），本任务只生成不判定。
- `just tauri-bindings-check` exit 0；src-tauri `cargo check` exit 0、`cargo clippy` 0 warning/error、`cargo test -- --test-threads=1` 490+2 全过。

## 收尾数据

- 最终验证数据（release 构建计时/体积、version-check、audit）：见本文件末次更新记录。
