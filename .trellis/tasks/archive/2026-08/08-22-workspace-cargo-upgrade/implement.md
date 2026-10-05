# 执行计划：workspace Cargo 依赖全量升级

> 父任务：`08-22-react-migration`（旁路子任务 2b）。可与阶段 1–3 任一并行，不占迁移主线关键路径。
> 分支：`feature/react-migration/workspace-cargo-upgrade`。**PR 目标为 `dev`，不经迁移分支**（父任务 `implement.md` §5 的例外条款）。合入 `dev` 后 rebase 到 `feature/react-migration`。

## 前置确认
- **分支偏差（主线程批准，2026-08-23）**：不在 `dev` 上开 `feature/react-migration/workspace-cargo-upgrade`，改在当前迁移分支 `react-migration/react-foundation` 上执行；PR 到 `dev` 的步骤推迟到发布时由人工决定。

- [x] ~~`just ci` 在 `dev` 上全绿~~（分支偏差：不在 dev 执行，基线由父任务采集门覆盖）
- [x] ~~checkout 独立分支~~（偏差：在 `react-migration/react-foundation` 上执行，PR 到 dev 推迟人工决定）
- [x] 记录升级前基线：release 构建 250 s / ccr.exe 18,902,016 B（见 upgrade-inventory.md）。
- [x] `cargo upgrade --dry-run` 不可用（cargo-edit 缺失），以 crates.io 查询 + caret 编辑 + `cargo update -p` 回退方法落盘。
- [x] origin 上 7 个 dependabot 分支目标版本已记录并复核，全部被本次目标覆盖或持平。
- [x] **组 C 前置**：三格式向量 + blake3/sha256 摘要向量升级前固化并自检通过（临时测试已删除未提交），结果见 crypto-compat-check.md。
- [x] **组 B 前置**：Cookie 回发 PASS、公开端点 `response.version()`=HTTP/2.0 基线落盘；真实账号项「未执行」。
- [x] 组 A 一次提交（commit d1da6768）。
- [x] 组 B 六个子组各自提交（B1=11a164ad、B2 reqwest 无变化仅复采、B3=bd2f9f12、B4=766d4f3d、B5=2c225f7e、B6=306d6756）；reqwest 三项配置核对与观测点复采落盘；rusqlite 组投影测试 45/45 与 llmusage_no_crate_guard 2/2 通过。
- [x] 组 C 逐依赖提交（base64 eee2df4b → sha2 无变化 → blake3 855fb50c → rand 5067a4ae → argon2 613cc564 → aes-gcm 保留 0.10）；每步全量向量读回；crypto-compat-check.md 落盘（bfc2b998）。
- [x] 组 D + dep-upgrade 段3 一次提交（1176a416）：ts-rs 12.0.1 双侧对齐，204 文件重生成，tauri-bindings-check exit 0；diff 判定归对方 AC7 跟进。
- [x] 收尾：upgrade-inventory.md 全量落定、release 复测（214 s / 18,872,832 B）、version-check / audit / 全验证门 exit 0（9e711ec2 及后续记录提交）。
- [x] `anyhow`、`thiserror`、`clap`、`serde` / `serde_json`、`rpassword`、`tracing`、`uuid`、`indexmap`、`once_cell` 等 patch 级条目升级。
- [x] 一次提交。

验证：`just check-workspace`、`just lint-strict`、`just test`。

## 组 B：minor 级升级

按依赖分组提交，每组提交后验证。

- [x] `tokio` / `tokio-util` / `futures` / `async-stream`。
- [x] `reqwest`。**升级后立即核对三项配置**：`default-features = false`、`native-tls`、`http2`（AC10）。再按 `design.md` §3b 复采四个观测点，与「组 B 的前置」基线对比（AC11）。
- [x] `toml`、`chrono` / `chrono-tz` / `iana-time-zone`。
- [x] `rusqlite` / `r2d2` / `r2d2_sqlite`。核对 `bundled` 与 `functions` feature 保留。
- [x] `axum` / `tower` / `tower-http`。
- [x] `dirs`、`tempfile`、`walkdir`、`open`、`sysinfo`。

验证（每组后）：`just check-workspace`、`just test`。`rusqlite` 组额外跑 `crates/ccr-usage` 的投影相关测试与 `llmusage_no_crate_guard`（AC12）。

## 组 C：加密与身份

每个依赖单独提交。顺序：`base64` → `sha2` → `blake3` → `rand` → `argon2` → `aes-gcm`（风险递增）。

- [x] 每个依赖升级后，用新版本读回组 C 前置产出的**全部**向量（三种持久格式 + 摘要向量），验证解密与摘要一致（`design.md` §3.1）。
- [x] `rand` 的验证点为 API 兼容与未改用弱生成器（`design.md` §3.1 末段）。
- [x] 任一格式的向量不匹配，该依赖保留当前版本，原因写入 `upgrade-inventory.md`。
- [x] `crypto-compat-check.md` 落盘，逐格式列结果（AC9）。

验证：`just test`、`just secret-write-check`。

## 组 D：ts-rs 11 → 12

- [x] 与 `08-22-dep-upgrade` 确认 `src-tauri` 侧同步升到同一 12.x 版本（协同点 A）。版本不一致则不执行生成。
- [x] 升级 workspace 的 `ts-rs` 声明（`crates/ccr-usage`、`crates/ccr-cli` 的 `ts` feature 路径）。
- [x] `cd ccr-ui && just bindings` 生成 204 个文件。
- [x] 通知 `08-22-dep-upgrade` 执行 diff 逐条判定（其 R7 / AC7）。本任务提交生成产物，不做判定。
- [x] 一次提交，含 Rust 侧版本变更与生成产物。

验证：`just tauri-bindings-check` 退出码 0；`cd ccr-ui/src-tauri && cargo check`。

## 收尾

- [x] `upgrade-inventory.md` 落盘，覆盖 `[workspace.dependencies]` 全部条目，无空缺（AC7）。
- [x] `cargo clean && cargo build --release` 计时与二进制体积记录，与前置基线对比（AC13）。
- [x] HTTP/2 与 Cookie 的四个观测点数据与基线对比落盘（AC11）。协议版本项取 `response.version()`，不以「签到请求成功」代替。无真实账号时该项标「未执行」并说明。
- [x] `just version-check` 确认 `workspace.package.version` 仍为 7.2.0（AC6、R10）。

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

- [x] AC1–AC13 全部满足。
- [x] 三份记录落盘：`upgrade-inventory.md`、`crypto-compat-check.md`、编译时间与体积对比。
- [x] `reqwest` 三项配置核对记录落盘（AC10）。
- [ ] 签到真实请求验证通过（AC11）——「未执行」：无真实签到账号，按 AC11 规则以公开端点协议版本观测替代，不标通过。

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
