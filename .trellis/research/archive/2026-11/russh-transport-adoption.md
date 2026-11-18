# russh 持久会话在 CCR 工作区的落地约束

## 摘要

- 当前稳定版是 **russh 0.65.1**（2026-11-03，MSRV 1.89）、**russh-sftp 3.0.1**、**keyring 4.2.0**。russh 0.65 已迁到 RustCrypto 新一代（`digest 0.11`、`rand_core 0.10`、`sha2 0.11`、`cipher 0.5`），与工作区的 `sha2 0.11`、`rand 0.10`、`aes-gcm 0.11`、`argon2 0.6` 是同一代。`ccr-ui/src-tauri/Cargo.toml` 里「russh 0.57 的 rand_core 预发布版与 ccr-db 冲突」的注释已经过时：探针工程把这些版本放在一起，可以正常解析。
- **加密后端**：russh 默认 feature 是 `aws-lc-rs`。在本机 Windows MSVC 上没有 NASM 时，`aws-lc-sys 0.47.0` 构建失败（实测）。改用 `default-features = false, features = ["ring", ...]` 可以构建通过（实测 `cargo check`）。三份 lock 里已经有 `ring 0.17.14`（由 rustls-webpki 等引入）。
- **审计门禁（阻塞项）**：russh 0.65.1 在 `rsa` feature 下精确锁定 `rsa =0.10.0-rc.18`。`cargo audit` 把它判为 RUSTSEC-2023-0071，报错退出（实测）。根仓 `.cargo/audit.toml` 的 `ignore = []`，CI 会审计根 lock 与 Tauri lock，所以开着 `rsa` 的已发布 russh 过不了现有门禁。上游 main 已在 2026-11-11 改为 `rsa 0.10` 与 `ssh-key =0.7.0-rc.12`（#821），但还没发版。关掉 `rsa` 可以通过审计，代价是不支持 RSA 主机密钥和 RSA 私钥。
- **Windows 认证**：russh 自带 `AgentClient::connect_named_pipe(path)` 与 `connect_pageant()`（均为 `cfg(windows)`）。`connect_env()` 只在 unix 上有，Windows 要显式传 `\\.\pipe\openssh-ssh-agent`。私钥支持 OpenSSH 新格式、PKCS#8（含加密）、PKCS#1 RSA、PKCS#5 加密 PEM 和 PuTTY PPK，口令通过 `decode_secret_key(secret, Some(pw))` 传入。客户端支持 password 与 keyboard-interactive。
- **主机密钥**：`client::Handler::check_server_key` 默认拒绝，由调用方判断。russh 自带的 `learn_known_hosts_path` 是非原子追加写，不符合仓库「原子写、加锁」约束，应继续用现有的 `persist_known_host`。现有「指纹比对 + challenge 确认」可以挪进 `check_server_key`，不再需要 `ssh-keyscan`。
- **会话生命周期**：`client::connect` 用 `tokio::spawn` 拉起常驻会话任务，TCP 连接本身不带超时。支持 `keepalive_interval`、`keepalive_max`、`inactivity_timeout` 和 `Handle::is_closed()`。断线后不会自动重连。`authenticate_*` 需要 `&mut self`，`channel_open_session` 只需 `&self`，同一连接上可以并发开多个 channel。
- **凭据存储**：keyring 4.x 拆成 `keyring-core` 加各平台 store。`v1` feature 提供开箱即用的 `Entry`，后端是 Windows 凭据管理器、macOS Keychain、Linux Secret Service（zbus）。API 是同步的。Windows 单条密文不超过 `CRED_MAX_CREDENTIAL_BLOB_SIZE`。zbus 系列会带来 3 条 unmaintained 警告（不导致 audit 失败）。
- **Tauri**：`ccr-ui/src-tauri` 不使用 `environments::ssh::Workspace`，只用 `ssh_security` 的 OpenSSH 命令构造、keyscan 解析、known_hosts 写入，以及 `ssh_session` 的重连与断开判定。只要 `ccr-app` 无条件依赖 russh，russh 就会进入根 lock、`ccr-desktop` lock 和 Tauri lock，审计门禁同时作用于三处。

研究票 #198（地图 #195）。代码基线 `origin/dev` @ `8d498849`。外部来源的访问日期都是 2026-11-15。探针工程放在会话临时目录，没有改动产品代码，也没有改动任何 lock。

## 1. 版本与依赖

### 1.1 当前版本（crates.io API，2026-11-15）

| crate | 最新稳定 | 发布日 | MSRV | 许可 | 备注 |
|---|---|---|---|---|---|
| `russh` | 0.65.1 | 2026-11-03 | 1.89 | Apache-2.0 | 0.63.0（08-21）→ 0.64.0（10-04）→ 0.65.0（10-25），约每月一次 0.x 破坏性升级 |
| `russh-sftp` | 3.0.1 | 2026-09-28 | 未声明 | Apache-2.0 | 对 russh 只有 dev 依赖（`^0.63.2`），运行时与 russh 版本解耦 |
| `pageant` | 0.2.5 | 2026-10-25 | — | — | russh 的 `cfg(windows)` 依赖，同属 warp-tech/russh 仓库 |
| `keyring` | 4.2.0 | 2026-08-29 | 1.88 | MIT OR Apache-2.0 | 依赖 `keyring-core ^1`（1.0.1，2026-10-13） |
| `aws-lc-rs` / `aws-lc-sys` | 1.19.0 / 0.47.0 | 2026-10-28 | 1.71 | ISC 等 | russh 默认后端 |
| `ring` | 0.17.14 | 2025-03-11 | 1.66 | Apache-2.0 AND ISC | 三份 lock 已有 |
| `rsa` | 0.10.0 | 2026-11-08 | — | — | russh 0.65.1 锁定 `=0.10.0-rc.18` |
| `ssh-key` | 0.6.7（稳定）/ 0.7.0-rc.12 | 2026-11-09 | — | — | russh 0.65.1 锁定 `=0.7.0-rc.11` |

工作区工具链：`rust-toolchain.toml` 为 1.98.0，`crates/ccr-app/Cargo.toml` 与 `ccr-desktop/Cargo.toml` 的 `rust-version` 为 1.95。两者都满足 russh 1.89 和 keyring 1.88 的 MSRV。

### 1.2 russh 0.65.1 的 feature 与关键依赖

来源：crates.io `/api/v1/crates/russh/0.65.1`（features、dependencies）；源码 `russh-0.65.1/Cargo.toml.orig:24,68,111-120`，`src/lib.rs:90-95`。

- `default = ["flate2", "aws-lc-rs", "rsa"]`
- `aws-lc-rs = ["dep:aws-lc-rs"]`，依赖为 `aws-lc-rs ^1.16.2`（`default-features = false`，开 `alloc`、`aws-lc-sys`）
- `ring = ["dep:ring"]`，依赖为 `ring ^0.17.14`
- `rsa = ["dep:rsa", "dep:pkcs1", "ssh-key/rsa"]`，锁定 `rsa =0.10.0-rc.18`、`pkcs1 =0.8.0-rc.4`
- 两个后端都没开时，`src/lib.rs:90-93` 会 `compile_error!`。两个都开时，`cipher/{mod,gcm,chacha20poly1305}.rs` 用 `#[cfg(feature = "aws-lc-rs")]` 优先选 aws-lc-rs。两个后端的算法集合一样，只是实现不同。
- 固定依赖：`ssh-key =0.7.0-rc.11`（开 `ed25519`、`p256/384/521`、`encryption`、`ppk`、`sha1`），以及 `rand ^0.10`、`rand_core ^0.10`、`digest ^0.11`、`sha2 ^0.11`、`cipher ^0.5.1`、`getrandom ^0.4`、`ml-kem ^0.3`。
- tokio：非 wasm 目标要求 `io-util`、`rt-multi-thread`、`time`、`net`，与工作区根 `tokio` 的 feature 集合兼容。
- **feature 合并的约束**：依赖图里只要有一处用默认 feature 引入 russh，`aws-lc-rs` 就会被合并打开，同时编译两个后端，最终选用 aws-lc。所以选 ring 时，每个声明 russh 的 crate 都要写 `default-features = false`。

### 1.3 加密后端在 Windows MSVC 上的构建（实测与文档）

- **aws-lc-rs（默认）**：AWS 用户指南（Windows 要求页）写明 x86_64-pc-windows-msvc 非 FIPS 构建需要 C 编译器和 NASM。只有开启 `prebuilt-nasm` feature 或设置 `AWS_LC_SYS_PREBUILT_NASM=1` 时，才会用预编译汇编对象代替 NASM。CMake 只在 FIPS 构建时需要。源码对应 `aws-lc-sys-0.47.0/builder/main.rs` 的 `use_prebuilt_nasm()`（1155-1163 行）：没检测到 NASM，并且 feature 或环境变量**显式允许**时才走预编译。builder 选择逻辑在 `get_builder()`（709-797 行），非 FIPS 默认走 cc builder。
- **实测**：本机（VS 18 Enterprise MSVC 14.52，未安装 NASM 和 clang-cl，已有 CMake）用默认 feature 执行 `cargo check` russh 0.65.1，`aws-lc-sys` 的构建脚本 panic：`NASM command not found! Build cannot continue.`（`nasm_builder.rs:137`）。
- **ring**：本机用 `default-features = false, features = ["ring", "rsa", "flate2"]` 加上 russh-sftp 3.0.1、keyring 4.2.0 执行 `cargo check`，通过（171 个 crate，49 秒）。根、`ccr-desktop`、Tauri 三份 lock 都已有 `ring 0.17.14`（根和 Tauri 由 `rustls-webpki 0.103.16` 引入，desktop 由 `rustls`、`quinn-proto`、`rustls-webpki` 引入），说明这台机器和 CI 已经在编译 ring。
- 根 `Cargo.toml` 的 reqwest 注释写着「aws-lc-rs（needs cmake/C build）」，所以仓库此前就有意避开 aws-lc-rs。按上面的文档，非 FIPS 构建实际需要的是 NASM，不是 CMake。
- CI 的 Windows 作业用 `windows-2025`（`ci.yml:191`、`tauri-rust-ci.yml:90`），release 用 `windows-latest`（`release.yml:38,221`）。本次没有核实这些 runner 镜像有没有 NASM。

### 1.4 审计门禁（实测）

- 门禁现状：`justfile:398-402` 的 `audit` 依次执行 `cargo audit --file Cargo.lock` 和 `cargo audit --file ccr-ui/src-tauri/Cargo.lock`。CI 的 `security-audit` 作业（`.github/workflows/ci.yml:216-233`）做同样的事。`ccr-desktop` 的 lock 由 `ccr-desktop/justfile:68-69` 的 `desktop-audit`（`just gpui-audit`）审计，不在 CI 的 security-audit 里。`.cargo/audit.toml` 为 `ignore = []`。cargo-audit 版本是 0.22.2（`justfile:14`）。仓库里**没有** `deny.toml`，也没有 cargo-deny 门禁。
- 探针 A：russh 开 `ring` + `rsa` + `flate2`，加上 russh-sftp 和 keyring，执行 `cargo audit`。结果：
  - `rsa 0.10.0-rc.18` 命中 **RUSTSEC-2023-0071**（Marvin，严重度 5.9），报 `error: 1 vulnerability found!`。advisory 的 `patched = [">= 0.10.0"]`（`rustsec/advisory-db: crates/rsa/RUSTSEC-2023-0071.md`）。按 semver 规则，预发布版 `0.10.0-rc.18` 低于 `0.10.0`，所以不算已修复。
  - keyring 的 Linux zbus store 带入 `zbus_names 4.3.5`、`zvariant 5.15.1`、`zvariant_derive 5.15.1`，触发 unmaintained 警告 RUSTSEC-2026-0370、0371、0372，只是 allowed warnings，不影响退出码。`ccr-desktop/Cargo.lock` 里已有这三个版本，根 lock 和 Tauri lock 里还没有。
- 探针 B：russh 只开 `ring` + `flate2`（不开 `rsa`），加上 russh-sftp，执行 `cargo audit`，无漏洞。lock 里没有 `rsa`，仍有 `ssh-key 0.7.0-rc.11`。
- 上游进展：warp-tech/russh 的 main 在 `c5443372`（2026-11-11，#821「deps: update rsa to 0.10 and ssh-key to 0.7.0-rc.12」）把依赖改为 `rsa = "0.10"`、`pkcs1 = "0.8"`，工作区 `ssh-key = "=0.7.0-rc.12"`。截至访问日，crates.io 上还没有包含这个提交的版本。
- 关掉 `rsa` 的影响（`russh-0.65.1/src` 中 `#[cfg(feature = "rsa")]` 涉及 `keys/format/mod.rs:34,74-85`、`negotiation.rs:975`、`keys/key.rs:137-143` 等）：`-----BEGIN RSA PRIVATE KEY-----` 直接返回 `UnsupportedKeyType`，RSA 主机密钥和 RSA 公钥认证都不可用。

### 1.5 三份独立 lock 的影响

- `crates/ccr-app` 是根工作区成员（根 `Cargo.toml:5`），同时以 path 依赖进入 `ccr-desktop`（独立 `[workspace]`，`ccr-desktop/Cargo.toml:10,26`）和 `ccr-ui/src-tauri`（`Cargo.toml:24`，有独立 lock）。
- 只要在 `ccr-app` 无条件加 russh，三份 lock 都会新增 russh、ssh-key 等依赖：根 lock 和 Tauri lock 进入 CI 审计，desktop lock 进入 `gpui-audit`。如果改成 `ccr-app` 的可选 feature、只由 `ccr-desktop` 打开，Tauri 和根的**编译图**里不会有 russh。不过 Cargo 的 lockfile 会解析所有可选依赖，所以这些包仍会出现在根 lock 和 Tauri lock 里，`cargo audit --file` 照样会扫描到。这一点是依据 Cargo 解析行为的推断，本次没有对仓库实测。
- 三份 lock 当前的加密相关版本（用 `tomllib` 解析 lock）：`rand_core` 根 / Tauri 为 0.10.0，desktop 为 0.10.1；`digest` 都有 0.11.x；`sha2` 都有 0.11.x；三份都没有 `aws-lc-rs`。russh 0.65.1 需要的 `rand_core ^0.10`、`digest ^0.11`、`sha2 ^0.11` 都与现有版本落在同一主版本内。
- `ccr-desktop` 的 tokio 只开了 `rt`、`sync`、`time`、`macros`（`ccr-desktop/Cargo.toml`），但 `ccr-app` 经由工作区 tokio 已经带入 `net`、`rt-multi-thread`、`io-util`，feature 会合并，不需要另外改。

### 1.6 SFTP 方案

- **russh-sftp 3.0.1**：只实现 SFTP v3（`README.md`）。客户端入口是 `SftpSession::new(stream)`，流来自 `channel.request_subsystem(true, "sftp")` 之后的 `channel.into_stream()`。提供 `read/write/create/open_with_flags/rename/remove_file/create_dir/metadata/try_exists/canonicalize` 等方法（`src/client/session.rs:38-289`）。默认请求超时 10 秒，可用 `set_timeout(secs)` 调整（`session.rs:86-91`）。支持的扩展只有 `limits@`、`hardlink@`、`fsync@`、`statvfs@`、`expand-path@openssh.com`（`src/extensions.rs:3-7`）。
- **原子替换的坑**：现有的 `sftp_write_plan`（`ssh_security.rs:324`）是「先 put 临时文件，再 rename 覆盖」，用的是 OpenSSH `sftp` 客户端。OpenSSH 客户端在服务端支持时，会把 `rename` 自动转成 `posix-rename@openssh.com`（`openssh-portable/sftp-client.c:1238-1270`，`use_ext = (conn->exts & SFTP_EXT_POSIX_RENAME) && !force_legacy`）。russh-sftp 的 `SftpSession::rename` 发的是普通 `SSH_FXP_RENAME`（`session.rs:230-236`）。OpenSSH 服务端处理普通 rename 时先 `link(old,new)`（`sftp-server.c:1261-1301`），目标已存在就返回失败，所以**不能覆盖已有配置文件**。要做原子覆盖，得通过 `RawSftpSession::extended("posix-rename@openssh.com", data)`（`rawsession.rs:713`，公开类型见 `client/mod.rs:9`）自己编码请求，或者在 exec channel 里执行远程 `mv`。
- **远程命令方案**：russh 的 `Channel::exec`（`channels/mod.rs:237,542`）、`data`、`eof`、`wait` 和 `make_reader` 可以直接替代现在 `ssh ... 'printf'` 和 `command -v` 这类单次命令（`ssh.rs:426-440`，`ssh/detect.rs:64-71`）。

## 2. 认证能力

### 2.1 SSH agent（Windows）

- russh 源码 `src/keys/agent/client.rs`：
  - `#[cfg(unix)] impl AgentClient<UnixStream>`：`connect_uds`、`connect_env`，后者读取 `SSH_AUTH_SOCK`（61-88 行）。**Windows 上没有 `connect_env`。**
  - `#[cfg(windows)] AgentClient::connect_pageant()`（93-99 行），底层是 `pageant::PageantStream`。
  - `#[cfg(windows)] AgentClient::connect_named_pipe(path)`（101-120 行），使用 `tokio::net::windows::named_pipe::ClientOptions`。遇到 `ERROR_PIPE_BUSY`（231）时每 50ms 重试，**没有上限也没有超时**，调用方要自己套 `tokio::time::timeout`。
  - `AgentClient<R>` 实现了 `auth::Signer`（`src/auth.rs:255-`），配合 `Handle::authenticate_publickey_with(user, pubkey, hash_alg, &mut agent)`（`client/mod.rs:535`）使用，私钥不经过本进程。
- Win32-OpenSSH 的 agent 管道名：`PowerShell/openssh-portable` 的 `contrib/win32/win32compat/ssh-agent/agent.c:50` 定义为 `AGENT_PIPE_ID L"\\\\.\\pipe\\openssh-ssh-agent"`。`wmain_common.c:53-54` 在 `SSH_AUTH_SOCK` 未设置时把它设成 `\\.\pipe\openssh-ssh-agent`。现在 `ssh.exe` 子进程走的就是这个默认值，换成 russh 后要由应用自己做同样的回退：先读 `SSH_AUTH_SOCK`，没有再用默认管道名。
- `pageant 0.2.5`：默认 feature 是 `wmmessage` + `namedpipes`（`Cargo.toml [features]`）。`wmmessage` 通过 `FindWindowW("Pageant")`、`WM_COPYDATA` 和共享内存通信，单条消息上限 8192 字节（`src/wmmessage.rs:193,201`）。`namedpipes` 连接 `\\.\pipe\pageant.<user>.<suffix>`（`src/namedpipes.rs:50-53`）。

### 2.2 私钥格式

来源：`russh-0.65.1/src/keys/format/mod.rs:44-107`，`keys/mod.rs:251` 的 `load_secret_key(path, password)`。

| 格式 | 识别头 | 支持 | 口令 |
|---|---|---|---|
| OpenSSH 新格式（ed25519 / ecdsa / rsa） | `BEGIN OPENSSH PRIVATE KEY` | 是（`decode_openssh`） | 支持（bcrypt-pbkdf，ssh-key `encryption` feature） |
| PKCS#1 RSA | `BEGIN RSA PRIVATE KEY` | 只在 `rsa` feature 下 | 支持 PKCS#5 `DEK-Info: AES-128-CBC` 加密 PEM（`Pkcs5Encrypted`） |
| PKCS#8 / SEC1 EC | `BEGIN PRIVATE KEY` / `BEGIN EC PRIVATE KEY` | 是 | — |
| 加密 PKCS#8 | `BEGIN ENCRYPTED PRIVATE KEY` | 是（`pkcs8` 开 `encryption`） | 支持 |
| PuTTY PPK | `PuTTY-User-Key-File-` | 是（ssh-key `ppk`） | 支持 |

- ed25519 由 ssh-key 和 `ed25519-dalek ^3` 提供。RSA 依赖 `rsa` feature（见 1.4）。
- 认证调用为 `authenticate_publickey(user, PrivateKeyWithHashAlg)`（`client/mod.rs:493`）。RSA 要配合 `best_supported_rsa_hash()`（809 行）选择 `rsa-sha2-256/512`。

### 2.3 密码与 keyboard-interactive

- `Handle::authenticate_password(user, password)`（`client/mod.rs:362`）。
- `authenticate_keyboard_interactive_start(user, submethods)` 和 `authenticate_keyboard_interactive_respond(responses)`（383、409 行）。返回 `KeyboardInteractiveAuthResponse::{Success, Failure{remaining_methods, partial_success}, InfoRequest{name, instructions, prompts}}`（279-292 行）。很多服务器只开 keyboard-interactive 不开 password，所以密码认证需要处理 `InfoRequest` 往返。
- `AuthResult::Failure { remaining_methods, partial_success }`（`src/auth.rs:153-162`）可以用来呈现「服务器允许的方法」。
- 现有代码的边界：`ssh_session::boundary_before_connect` 在 `password_only` 时返回 `AUTH_UNSUPPORTED`（`ssh_session.rs:35-46,65-67`）。Tauri 端 `commands/ssh.rs` 也使用这套判定。地图要求支持密码，这条规则需要改，而且它与 Tauri 共享（见第 6 节）。

## 3. 主机密钥

### 3.1 现有规则（代码事实）

- 应用私有 known_hosts 路径：`$CCR_ROOT/ssh/known_hosts`，默认 `~/.ccr/ssh/known_hosts`（`ssh_security.rs:375-383`）。desktop fixture 用 `<root>/ssh/known_hosts`（`ccr-desktop/src/runtime/environments.rs:76-92`）。
- 信任数据有两份：SQLite 的 `ssh_known_hosts` 行（`SshKnownHost { host, port, key_type, fingerprint, confirmed_at }`，`crates/ccr-db/src/models/ssh.rs:46-52`），以及给 OpenSSH 用的 known_hosts 文件。每个 `host:port` 只保留一行，写入时替换同一 marker（`persist_known_host_sync`，`ssh_security.rs:592-`）。写入用 `write_guarded_versioned` 做 CAS，最多重试 4 次（`KNOWN_HOSTS_CAS_ATTEMPTS`，26 行），`secret: true`，再加 `LockManager` 锁。
- 指纹格式：`SHA256:` + `STANDARD_NO_PAD(base64(sha256(key_blob)))`（`parse_keyscan_output`，392-）。
- 流程（`ssh.rs`）：
  - `probe`（331-357）：先 keyscan，再把指纹与 SQLite 比对，得到 `HostKeyStatus`，然后 `SshTrustService::register` 生成 challenge，TTL 120s，一次性使用（`ssh_security.rs:24,463-`）。
  - `confirm`（359-395）：消费 challenge，写文件，再 upsert SQLite。
  - `connect`（396-447）：重新 keyscan。`Mismatch` 返回 `ssh_host_key_mismatch`，`New` 返回 `ssh_host_key_untrusted`。只有 `Matched` 才发起带 `StrictHostKeyChecking=yes`、`GlobalKnownHostsFile=none`、`UserKnownHostsFile=<app>`、`BatchMode=yes` 的 ssh 连接（`configure_common_options`，`ssh_security.rs:101-120`）。
  - `ssh_trust::decide_host_key` 永远返回 `may_store/may_trust/may_accept = false`（`ssh_trust.rs:60-88`），也就是说，比对结果本身不授予任何写入或信任权限。
- 现有实现有两条独立链路：keyscan 拿到的密钥用来做 UI 比对，OpenSSH 握手时再按 known_hosts 文件做一次强校验。

### 3.2 russh 能接管的部分

- `client::Handler::check_server_key(&mut self, &PublicKeyOrCertificate) -> Result<bool, _>`，默认返回 `Ok(false)`，即拒绝全部（`client/mod.rs:2799-2812`）。文档说明库会先校验 KEX 签名，再调用这个回调。对首次 KEX，`check_server_key` 接受之后才回调 kex 完成（2814-2819 行）。
- 返回 `Ok(false)` 时连接以 `Error::UnknownKey` 失败（`lib_inner.rs:136`）。所以 probe 可以改成：发起一次 russh 握手，在 handler 里把观测到的公钥存进共享状态（例如 `Arc<Mutex<Option<PublicKey>>>`），然后返回 `false`。这样不会进入认证阶段，也就不再需要 `ssh-keyscan.exe`。connect 时在回调里比对 SQLite 或文件中的指纹，只有 `Matched` 返回 `true`。这正好对应「新密钥和未知密钥不保存、变更密钥拒绝」：回调里只读不写，写入仍然只发生在 `confirm`。
- 指纹可以用 ssh-key 的 `PublicKey::fingerprint(HashAlg::Sha256)` 计算，它的 Display 形式也是 `SHA256:<base64 无填充>`。按 ssh-key 文档，这与现有格式一致，但本次没有拿同一把密钥逐字节对照。
- russh 自带的 `keys::known_hosts`：
  - `check_known_hosts_path(host, port, pubkey, path)`（`known_hosts.rs:24-48`）：同算法不同密钥返回 `Err(KeyChanged{line})`，**不同算法时返回 `Ok(false)`**，不算变更。这和现有的「每个 host:port 一把密钥、按 SHA256 指纹全文比对」语义不同。
  - `learn_known_hosts_path`（137-175）：`OpenOptions::append` 直接追加，没有临时文件、rename、锁或 CAS，**不满足**仓库的「原子写、加锁、先备份」约束。默认路径函数读的是 `~/.ssh/known_hosts`，与应用私有文件无关。
- 算法协商对比对结果的影响：`Preferred::DEFAULT.key` 的顺序是 Ed25519 → ECDSA P-256/384/521 → RSA（`negotiation.rs:213-232`）。现有 keyscan 用 `-t ed25519,ecdsa,rsa`，取输出里第一条有效行（`ssh.rs:308-316`，`parse_keyscan_output`）。服务器同时有多把主机密钥时，两边选中的密钥类型通常相同，但不能保证。要稳定命中已存的那一把，可以把 `Config.preferred.key` 限定为已存的 `key_type` 对应算法。没开 `rsa` feature 时，只有 RSA 主机密钥的服务器无法完成 KEX。

## 4. 会话生命周期

### 4.1 russh 机制（源码事实）

- **建立连接**：`client::connect(config, addrs, handler)` 先执行 `TcpStream::connect(addrs).await`，然后调用 `connect_stream`（`client/mod.rs:1188-1210`）。**TCP 连接和 KEX 都没有内置超时**，要用 `tokio::time::timeout` 包住整个 connect 和认证过程。现有实现的 `ConnectTimeout=10`（`ssh.rs:428`）也要改成这种方式。
- **常驻任务**：`connect_stream` 用 `russh_util::runtime::spawn(session.run(...))` 拉起会话事件循环（同文件约 1261 行，非 wasm 下就是 `tokio::spawn`），`Handle` 持有它的 `JoinHandle`（316-322 行）。因此**必须在 tokio runtime 上下文里调用**，会话任务挂在调用方所在的 runtime 上，生命周期跟着 runtime，而不是跟着某一次 job。`Handle` 的 `Drop` 只打一条 debug 日志（324-328 行），不会主动断开，断开要调 `disconnect(reason, desc, lang)`（1034 行）。
- **保活与断线检测**：`Config { keepalive_interval, keepalive_max (默认 3), inactivity_timeout }`，默认都是 `None`（`client/mod.rs:2724-2769`）。事件循环在一段时间没收到数据时发送 keepalive，连续超过 `keepalive_max` 次没有响应就以 `KeepaliveTimeout` 结束（1409-1492 行，`lib_inner.rs:177`）。`Handle::is_closed()` 读取内部发送端是否已关闭（331 行）。`Handler::disconnected(reason)` 回调在断开时触发（3193-3207 行）。还有手动的 `send_keepalive(want_reply)`（1081 行）。
- **重连**：russh 不会自动重连，断线后要重新 `connect` 并重新认证，旧 `Handle` 和 `Channel` 都作废。
- **channel 并发**：`channel_open_session(&self)`（844 行）、`exec(&self)` 和 `data(&self)` 都只需共享引用，所以同一个 `Handle` 可以放在 `Arc` 里并发开多个 channel。`authenticate_*` 需要 `&mut self`（346-535 行），认证阶段必须独占。`Config.channel_buffer_size` 默认 100，`window_size` 默认 2 MiB（2761-2763 行）。
- **channel 关闭**：`channels/mod.rs` 和 `channel_ref.rs` 里没找到 `Drop` 实现，关闭要显式调用 `eof()` 或 `close()`（407、416 行）。
- **取消**：russh API 不接收 `CancellationToken`，要在调用方用 `tokio::select! { _ = cancel.cancelled() => ..., r = fut => ... }` 包起来。取消后 future 被丢弃，会话任务本身还在跑。取消时是否要 `disconnect`，由调用方决定。

### 4.2 与 `ccr-app::runtime` 和桌面执行器的配合（代码事实）

- `RuntimeOwner::new` 构建 multi-thread runtime：`worker_threads(2)`，`max_blocking_threads(BLOCKING_CONCURRENCY=4)`（`runtime.rs:42,207-224`）。`read`、`job` 和 `job_with_finalizer` 把 work 放进 `spawn_blocking`，并发数受同名 semaphore 限制（`runtime.rs:340-376`）。
- 桌面端 SSH 操作的执行方式：在 blocking 线程里 `tokio::runtime::Handle::current().block_on(work)`（`ccr-desktop/src/runtime/environments.rs:150-192`）。work 是 `ssh::Workspace` 的 async 方法。在这里调用 `russh::client::connect`，内部的 `tokio::spawn` 会把会话任务挂到 `RuntimeOwner` 的 worker 上。job 结束后会话还在，这正是「持久会话」需要的。
- 由此得到的约束：
  1. 会话任务**不在** `RuntimeHandle.tasks`（`TaskTracker`）中。`RuntimeOwner::shutdown` 只等待被跟踪的任务（`runtime.rs:227-246`），runtime drop 时会直接中止会话任务，不会发 SSH disconnect。想优雅断开，需要在 shutdown 前显式调用 `Workspace::disconnect`。
  2. 每个 SSH 操作在等待网络期间都占用一个 blocking 槽位（总共 4 个）。会话的 I/O 循环跑在 2 个 worker 上。
  3. 现有 `Workspace.serial: tokio::sync::Mutex<()>` 把所有操作串行化（`ssh.rs:140`，在 save、probe、connect、read、save 和 `detect_cli` 入口加锁）。如果保留这把锁，russh 在 channel 并发上的优势就用不上。
  4. 现在的 `Session` 只记录「逻辑授权」，不反映 TCP 是否存活（`ssh.rs:281-289`，`retained_session_count` 注释）。russh 可以通过 `is_closed()` 提供真实的存活信号。`Document.session_id` 和 `CliSession` 的「重连后不得复用旧授权」规则（`ssh.rs:64-70,577-579`，`ssh/detect.rs:17-22,57-59`）可以照旧绑定到每次 connect 生成的 uuid 上。
  5. 现有 `Transport` trait 的参数是 `tokio::process::Command` 和 `ProcessDescriptor`（`ssh.rs:82-91`）。fixture 是 `FixtureTransport`（`ssh/fixture.rs`），桌面测试也通过 `Workspace::fixture` 注入（`ccr-desktop/src/features/environments/remote/detect/tests.rs:60`）。换成 russh 后，这个注入点的形状必须改。
  6. 现有错误码集合（`ssh_cancelled`、`ssh_network_error`、`ssh_write_outcome_unknown`、`ssh_cleanup_failed` 等）由桌面端映射成 `Completion` 变体（`environments.rs:203-209`），换传输后要保持这些字符串和语义不变。

## 5. 系统凭据存储

- **keyring 4.2.0**（`keyring-4.2.0/src/lib.rs`、`src/v1.rs`）：
  - 必须开 `v1` 或 `cli` 之一，否则 `compile_error!`。`default = ["v1"]`。
  - `v1`：`Entry::new(service, user)` 首次调用时自动安装平台 store。macOS 用 Keychain Services，Windows 用 Credential Manager，*nix 用 Secret Service（zbus，`crypto-rust`）。store 初始化失败时返回 `Error::NoDefaultStore`，可以通过 `Entry::store_status()` 查看原因。
  - crate 文档明确写着：需要自己选择 store 的应用不应该链接 `keyring`，而应直接用 `keyring-core` 加具体的 store crate。
  - 编译体量参考（探针 A，Windows 目标）：`windows-native-keyring-store` 依赖 `windows-sys`。Linux 目标会额外带入 zbus 系列。
- **keyring-core 1.0.x**：`Entry::{set_password, get_password, set_secret, get_secret, delete_credential}`（`keyring-core-1.0.0/src/lib.rs:130-281`），**同步阻塞 API**，在 ccr-app 里要放在 blocking 线程中调用。`set_default_store` 和 `unset_default_store` 是进程级全局状态（65、89 行）。自带 `mock` store（`src/mock.rs`），测试时可以注入，不碰真实凭据库。`Credential` 和 `CredentialStore` 都是 `Send + Sync`（`api.rs:146,258`）。`get_password` 返回普通 `String`，库本身不做 zeroize。
- **Windows 限制**（`windows-native-keyring-store-1.0.0/src/utils.rs:55-119`）：用户名、target 名和 secret 分别受 `CRED_MAX_USERNAME_LENGTH`、`CRED_MAX_GENERIC_TARGET_NAME_LENGTH`、`CRED_MAX_CREDENTIAL_BLOB_SIZE` 限制，超限会被拒绝。按 Win32 文档，最后一项是 5×512 字节，本次没有另外核对。SSH 密码远低于这个上限。
- **Linux 限制**：Secret Service 需要一个 D-Bus 会话和正在运行的 secret service（GNOME Keyring、KWallet 等）。headless 或纯 SSH 登录的环境里可能拿到 `NoDefaultStore`。这是 Secret Service 的通用机制，本次没有实测。`keyring` 另有 `linux-keyutils-keyring-store`（只在 `cli` feature 下可选），它的数据重启后不保留。
- **审计**：见 1.4，zbus 系列有 3 条 unmaintained 警告。`ccr-desktop/Cargo.lock` 已含这些包，引入 keyring 后它们会第一次出现在根 lock 和 Tauri lock 中（如果 keyring 放在 `ccr-app`）。
- 仓库已有约束：「密码进系统凭据存储，不进 SQLite」（地图 #195）。现有 `ssh_hosts` 表只有 `identity_file`，没有认证方式或密码列（`crates/ccr-db/src/models/ssh.rs:8-17`）。

## 6. 对 Tauri 端的影响

- `ccr-ui/src-tauri` **不调用** `ccr_app::environments::ssh::Workspace`（`rg` 无命中）。它依赖的共享 API 只有下面两组：
  - `ssh_security::*` 的整体重导出（`ccr-ui/src-tauri/src/ssh/security.rs:2`）。实际用到 `SshTarget`（`ssh_command`、`sftp_command`）、`run_openssh_command`、`parse_keyscan_output`、`persist_known_host`、`app_known_hosts_path`、`classify_host_key`、`HostKeyStatus`、`ScannedHostKey`、`SshTrustService`、`classify_ssh_failure`、`openssh_error`、`posix_single_quote`、`sftp_read_batch`、`sftp_write_plan`、`RemotePosixPath`、`SftpWritePlan`（`commands/ssh.rs:21-24`，`platform/ssh.rs:10-11`，`ssh/connection.rs:11-12`，`main.rs:232`）。
  - `environments::{ProcessHostTrust, ReconnectGate, allow_reconnect, allow_reconnect_target, disconnect_error, plan_disconnect, reconnect_gate, DisconnectReport}`（`commands/ssh.rs:10-13,842`，`state.rs:13`）。
- Tauri 端仍然通过 `ProcessDescriptor::{openssh, sftp, ssh_keyscan, ssh_keygen}`（`jobs/process/gateway.rs:144-206`）调用系统 OpenSSH，`ssh/auth.rs` 用 `ssh-keygen -lf` 取指纹，注释写明「不依赖 russh 库」（`ssh/auth.rs:1-7`）。
- 需要保持的边界：
  1. 上面这些公开函数的签名和语义要保留（或者在 Tauri 端提供等价实现）。特别是 `persist_known_host` 写的文件格式，Tauri 的 OpenSSH 进程会通过 `UserKnownHostsFile` 读取，两端共用同一个 `app_known_hosts_path()`，所以文件必须保持 OpenSSH 可读的格式。
  2. `boundary_before_connect` 和 `password_only` 是共享规则（`environments/mod.rs:11-15` 导出）。为了 desktop 支持密码去改它，会改变 Tauri 的行为。Tauri 端目前只 import 重连和断开相关的函数，没有 import `boundary_before_connect`（见 `commands/ssh.rs:10-13`），但它们都由 `ccr-app` 导出。
  3. 依赖图：russh 如果是 `ccr-app` 的无条件依赖，Tauri 构建会编译 russh。它的 Windows 构建在 `tauri-rust-ci.yml`（`windows-2025`），会受 1.3 节加密后端选择的影响；Tauri lock 进入 CI 审计，会受 1.4 节 rsa 问题的影响。
  4. `ccr-ui/src-tauri/Cargo.toml:125-127` 里注释掉的 `russh = "0.57.1"`、`russh-sftp = "2.1.1"` 和冲突说明已经过时（见摘要）。
- 地图 #195 把 `ccr-ui` 的 SSH 体验改造列为范围外，只要求「传输替换不得破坏 Tauri 端」。

## 7. 与地图既定前提的冲突或缺口

| 前提 | 事实 | 性质 |
|---|---|---|
| 使用 russh 持久会话 | 已发布的 0.65.1 开 `rsa` 时过不了 `cargo audit`（`ignore = []`） | **阻塞项**（三选一：等上游发布含 #821 的版本；关掉 `rsa`，放弃 RSA 主机密钥和私钥；在 `.cargo/audit.toml` 中忽略这条 advisory） |
| Windows 构建 | 默认 `aws-lc-rs` 在没有 NASM 的 MSVC 上构建失败 | 约束（`ring` 可用，但需要全图 `default-features = false`） |
| SSH agent（Windows） | 有 `connect_named_pipe` 和 `connect_pageant`，没有 Windows 版 `connect_env`；`connect_named_pipe` 没有超时 | 约束（应用自己解析管道路径并加超时） |
| SFTP 原子写 | russh-sftp 没有 `posix-rename@openssh.com` 封装，普通 rename 在 OpenSSH 服务端不能覆盖已有文件 | 约束（用 `RawSftpSession::extended` 或远程 `mv`） |
| 已信任主机密钥迁移 | russh 自带的 known_hosts 写入不是原子的；按算法比对的语义与现有不同 | 约束（继续用现有 `persist_known_host` 和指纹比对） |
| 密码认证 | 共享的 `boundary_before_connect` 目前拒绝 password-only | 共享规则需要调整，影响 Tauri |

## 来源

访问日期都是 2026-11-15。

- crates.io API：`/api/v1/crates/{russh,russh-sftp,keyring,russh-keys,ssh-key,pageant,aws-lc-rs,aws-lc-sys,ring,rsa,keyring-core,windows-native-keyring-store,apple-native-keyring-store,zbus-secret-service-keyring-store}`，以及 `russh/0.65.1`、`russh-sftp/3.0.1`、`keyring/4.2.0` 的 `dependencies` 端点。
- 从 crates.io 下载的源码包：`russh-0.65.1`、`russh-sftp-3.0.1`、`pageant-0.2.5`、`aws-lc-sys-0.47.0`（只解压了 `builder/`）、`keyring-4.2.0`、`keyring-core-1.0.0`、`windows-native-keyring-store-1.0.0`。
- docs.rs：https://docs.rs/keyring/4.2.0/keyring/
- AWS-LC for Rust 用户指南（Windows 要求页）：https://aws.github.io/aws-lc-rs/requirements/windows.html
- GitHub warp-tech/russh：`russh/Cargo.toml`、`Cargo.toml`（main），提交 `c5443372`（#821，2026-11-11）。
- GitHub rustsec/advisory-db：`crates/rsa/RUSTSEC-2023-0071.md`。
- GitHub openssh/openssh-portable：`sftp-server.c`（`process_rename`），`sftp-client.c`（`sftp_rename`、posix-rename）。
- GitHub PowerShell/openssh-portable：`contrib/win32/win32compat/ssh-agent/agent.c:50`，`contrib/win32/win32compat/wmain_common.c:53-54`。
- 本仓库 `origin/dev` @ `8d498849`：文中引用的各文件与行号。
- 本机实测（cargo 1.98.0、cargo-audit，Windows 11 x86_64 MSVC）：探针 A（ring + rsa + russh-sftp + keyring，加工作区 RustCrypto pin）解析、`cargo check` 和 `cargo audit`；探针 B（ring，不开 rsa）`cargo audit`；默认 feature（aws-lc-rs）`cargo check` 失败。
