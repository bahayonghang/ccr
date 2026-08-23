# 技术设计：workspace Cargo 依赖全量升级

> 父任务：`08-22-react-migration`（旁路子任务 2b，与前端迁移无技术依赖）。本文件写分组升级策略、加密依赖兼容性验证方法与 ts-rs 生成流程。

## 1. 分组升级策略

按回滚粒度与风险分四组，每组独立提交。

| 组  | 内容                                                                                                                                         | 风险                     | 提交边界             |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------ | -------------------- |
| A   | patch 级升级（`anyhow`、`clap`、`thiserror`、`serde_json`、`rpassword`、`tracing`、`uuid`、`indexmap`、`once_cell` 等）                      | 低                       | 一次提交             |
| B   | minor 级升级（`tokio`、`reqwest`、`toml`、`chrono` 系、`dirs`、`tempfile`、`walkdir`、`futures` 系、`axum` / `tower` 系、`open`、`sysinfo`） | 中，需逐个核对 changelog | 按依赖分组提交       |
| C   | 加密与身份（`aes-gcm`、`argon2`、`rand`、`sha2`、`blake3`、`base64`）                                                                        | 高，见第 3 节            | 每个依赖单独提交     |
| D   | `ts-rs` 11 → 12 及 204 个生成产物                                                                                                            | 中，跨仓协同             | 一次提交，含生成产物 |

数据库组（`rusqlite`、`r2d2`、`r2d2_sqlite`）归 B，但需单列核对项：`bundled` 与 `functions` feature 保留（R4），SQLite 版本变化对 `crates/ccr-usage` 的投影 SQL 无影响（R7）。

「最新兼容版」的具体版本号由 `cargo upgrade --dry-run`（cargo-edit）或 `cargo outdated` 在实施时给出。origin 上 7 个 dependabot 分支的目标版本作为起点，但需复核而非直接采信。

## 2. 必须保留的配置

以下四项在升级后需逐项确认存在，任一缺失即视为升级失败：

| 配置                                                   | 位置            | 原因                                                                |
| ------------------------------------------------------ | --------------- | ------------------------------------------------------------------- |
| `reqwest` 的 `default-features = false` + `native-tls` | 根 `Cargo.toml` | 改为 rustls + aws-lc-rs 需 cmake / C 工具链，编译时间显著增加（R2） |
| `reqwest` 的 `http2` feature                           | 同上            | 签到请求经 ALPN 协商 HTTP/2（R3）                                   |
| `rusqlite` 的 `bundled` + `functions`                  | 同上            | `bundled` 避免依赖系统 SQLite；`functions` 被投影 SQL 使用（R4）    |
| `resolver = "3"`、`[profile.dev]`、`[profile.test]`    | 同上            | R9                                                                  |

确认方式：`rg -A3 'reqwest|rusqlite' Cargo.toml` 人工核对，结果落盘（AC10）。

## 3. 加密依赖兼容性验证方法

`aes-gcm`、`argon2`、`blake3`、`sha2`、`rand` 的升级可能改变算法参数或 KDF 行为。读 changelog 不足以判定，必须用旧版本生成产物、新版本读回。

**现状更正**：先前工件记录已加密凭据在 `crates/ccr-config` 与 `crates/ccr-db`。实测两者都不做 AES 加密（`ccr-db` 只在 `services/usage_import_service.rs` 用 `sha2::Sha256` 做导入去重）。真实的加密调用点为三处，且**三处的持久格式互不相同**，因此一组通用向量不足以覆盖。

### 3.1 三种持久格式各自固化向量

| 格式                 | 实现                                                    | 格式要素                                                                                                       | 向量需固化的内容                                                     |
| -------------------- | ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| Codex auth 导出      | `crates/ccr-codex/src/services/codex_auth_crypto.rs`    | Argon2id（`Version::V0x13`）+ `KdfParams{m_cost,t_cost,p_cost}` + salt + AAD（version / format / exported_at / account_count 的确定性拼接）+ AES-256-GCM | 完整导出文件（含其真实 salt、KDF 参数、version、format、AAD 输入值） |
| Sync 信封 V2         | `crates/ccr-sync/src/sync/envelope.rs`                  | `EncryptedEnvelopeV2{magic, version, algorithm, kdf, kdf_params, salt, nonce, metadata{asset_id,relative_path,schema}, ciphertext}`，metadata 作 AAD | 完整信封字节，另需一个 `PlaintextV1` 样本以覆盖旧读路径              |
| CheckIn 凭据         | `crates/ccr-checkin/src/core/crypto.rs`                 | 独立随机 32 字节 key（base64 落盘，经 `write_guarded`）+ `base64(nonce \|\| ciphertext)`，**无 KDF**             | key 文件 + 密文串（key 来源是随机而非口令派生，与前两者不同）        |

每种格式：升级前用当前版本生成样本并落盘，升级后用新版本读回，验证解出同一明文。向量文件不含真实凭据（用已知测试口令与已知测试明文）。

`blake3` 与 `sha2` 的验证面不同——它们不参与解密，而是内容哈希与去重：

| 依赖     | 使用位置                                                                                                              | 变化的后果                     | 验证方式                     |
| -------- | --------------------------------------------------------------------------------------------------------------------- | ------------------------------ | ---------------------------- |
| `blake3` | `ccr-core/src/core/guarded_write.rs`、`ccr-skills`（versioning / trash / toggle / hash / skills_service）、`ccr-store/src/sessions/{indexer,parser}.rs`、`ccr-cli/src/services/multi_backup_service.rs` | 缓存与索引失效，不是数据无法解密 | 固定输入的摘要值升级前后一致 |
| `sha2`   | `ccr-db/src/services/usage_import_service.rs`                                                                         | 导入去重误判为新记录           | 同上                         |

`rand` 的验证点不是可重现性，而是 API 兼容与熵源不变。`rand` 0.10 系的 trait 与生成器命名在小版本间有变化，需确认调用点编译通过且未改用弱生成器。

三类验证全部记录在 `crypto-compat-check.md`（AC9）。若某依赖导致任一格式的向量不匹配，该依赖保留当前版本并在 `upgrade-inventory.md` 中记录原因（R1 允许保留）。

测试向量的生成位置：临时测试，不入库为常驻测试。原因是常驻测试需要固定的测试口令，与 `secret-write-check` 的检查面重叠。若判定值得常驻，另开任务。

## 3b. HTTP/2 与 Cookie 的观测点（R3、AC11）

「HTTP/2 协商与 Cookie 行为不变」当前无观测点：`crates/ccr-checkin/src/services/checkin_service.rs` 的请求函数只从 response 保留 status 与 body，不读 `response.version()`；仓库唯一的 http2 测试（`test_http2_feature_enabled`）是编译期守卫，断言 `Client::builder().http2_prior_knowledge()` 存在，与实际协商无关。因此一次成功的签到请求在 HTTP/1.1 下也会通过。

观测点定义：

| 项目          | 观测点                                                                                                          | 采集方式                                                                                                                       |
| ------------- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 协议版本      | `response.version()`                                                                                            | 升级前后各跑一次带临时 instrumentation 的构建，记录该值。instrumentation 不提交（C3：`crates/` 不做功能改动）                   |
| ALPN 协商     | 同上，`HTTP_2` 即协商成功                                                                                       | 若无真实账号，退化为对一个已知支持 h2 的公开端点发同配置请求，`#[ignore]` 标记的网络测试                                        |
| Cookie 发送   | `build_checkin_request` 构造的 `Cookie` 请求头                                                                  | 已有离线测试覆盖（现有 header 断言用例），升级后重跑即可，无需真实账号                                                          |
| Cookie 接收   | `Client::builder().cookie_store(true)` 的 jar：`Set-Cookie` 是否被保留并在后续同域请求中重发                     | 该 feature 确实开启（`checkin_service.rs:617`），jar 行为是真实风险面。用本地 mock server 发 `Set-Cookie`，断言第二次请求带回该 cookie |

前置条件（任一不满足则该项标「未执行」，不标「通过」）：真实签到账号、目标服务端可达、无中间代理改写协议。Cookie 接收项用本地 mock server，无外部前置条件，应始终执行。

升级前必须先采一次基线，否则「不变」无参照。

## 4. ts-rs 11 → 12

`ts-rs` 的使用面：

- `crates/ccr-usage` 的 `ts` feature。
- `crates/ccr-cli` 的 `ts` feature。
- `ccr-ui/src-tauri`（其 `Cargo.toml` 独立声明 `ts-rs = "11"`，由 `08-22-dep-upgrade` 升级）。

CLI 的默认依赖图不含 `ts-rs`（仅 `ts` feature 下引入）。

生成流程（已核实，`cd ccr-ui && just bindings`）：

```
rm -rf src/types/generated
cargo test --manifest-path ../Cargo.toml -p ccr-cli   --features ts export_bindings
cargo test --manifest-path ../Cargo.toml -p ccr-usage --features ts export_bindings
cargo --config ../.cargo/tauri-ci.toml test --manifest-path src-tauri/Cargo.toml export_bindings
bun ./scripts/normalize-generated-bindings.mjs
```

三条命令跨越 workspace 与 `src-tauri` 两个 manifest，因此两侧 `ts-rs` 版本必须同时升到 12.x。版本不一致时三条命令产出不同格式，`just tauri-bindings-check` 会报漂移，但报的是版本不一致而非真实类型变化。

协同点 A 的分工：本任务负责升级三处 `ts-rs` 声明中的 workspace 两处、执行生成、提交生成产物；`08-22-dep-upgrade` 负责 `src-tauri` 一处的升级与 204 个文件 diff 的逐条判定。生成产物与 Rust 侧版本变更在同一提交内（组 D）。

`normalize-generated-bindings.mjs` 的存在意味着 ts-rs 输出的空白格式不稳定。因此 diff 判定必须在 normalize 之后进行，否则格式噪声淹没类型变化。

## 5. 升级清单格式

`upgrade-inventory.md` 的列：依赖名、当前版本、目标版本、是否升级、未升级原因、所属组、核对要点。

覆盖根 `Cargo.toml` 的 `[workspace.dependencies]` 全部条目，无空缺（AC7）。

## 6. 编译时间与体积对比

`reqwest` 的 TLS 后端与 `rusqlite` 的 `bundled` 是编译时间的主要来源，升级后需确认未劣化：

- 编译时间：`cargo clean && cargo build --release` 计时，升级前后各一次。
- 二进制体积：`crates/ccr` 的 release 产物大小。

两项落盘（AC13）。该数据也是 R2 禁止改 TLS 后端的依据来源。

## 7. 不变量

- `workspace.package.version` 保持 7.2.0（R10）。版本变更走 `just version-sync`，不在本任务。
- `llmusage_no_crate_guard` 测试继续通过（R8、AC12）。该测试禁止引入上游 llmusage Rust crate；升级传递依赖时若意外引入，该测试会失败。
- Edition 2024 与 MSRV 1.88+ 不变（Out of Scope）。
- `crates/ccr-usage` 的 schema 版本门（`MIN_SUPPORTED_SCHEMA_VERSION`、provider schema 14）行为不变（R7）。

## 8. 未决项

- 各依赖的具体目标版本，见第 1 节最后一段。
- 加密依赖若出现向量不匹配，是保留旧版本还是写数据迁移，由实施时的具体情况决定。本设计倾向保留旧版本：迁移已加密的用户凭据数据的风险高于停留在旧版本。
