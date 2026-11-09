# mbx 本机诊断（2026-10-12）

## 结论

根因是有效 PATH 未启用已经安装的 mbx Cargo shim，因此普通 Cargo 及 just 构建绕过 mbx；仅修改 cache_dir 不会让这些构建启动缓存，也不会迁移旧受管 target。

## 当前证据

| 探针 | 结果 |
| --- | --- |
| `mbx --version` | 1.23.0 |
| `Get-Command cargo` | `C:\Users\lyh\.cargo\bin\cargo.exe` |
| `mbx setup --status` | shim 已安装且 current：`C:\Users\lyh\AppData\Local\mbx\bin\cargo.exe`；rust-analyzer override current |
| 全局配置 `C:\Users\lyh\AppData\Roaming\mbx\config.toml:1` | `cache_dir = "D:/mbx-cache"` |
| 同文件第 3–4 行 | `[gc] max_total_size = "100GiB"` |
| `mbx cache dir` | `D:/mbx-cache\actions`，注意打印的是 store 子目录 |
| `mbx cache stats` / `projects` | 0 objects、0 action results、0 checkouts、0 targets；no recorded workspaces |
| 根 `target` | symlink -> `C:\Users\lyh\AppData\Local\mbx\targets\v1\6b736f3e8608d890383d61470c216d50cd84b38db0e955508f465ce6ab4ea1b5` |
| `ccr-ui/src-tauri/target` | 普通目录；`mbx adopt --dry-run` 显示可接管，28.3 GiB logical |
| 环境覆盖 | 当前无 MBX / CARGO_TARGET_DIR / RUSTC_WRAPPER 等覆盖；真实 User 和 Machine PATH 均没有 mbx，Machine PATH 已含 `.cargo\bin` |
| Windows Developer Mode | 已开启；当前进程非管理员 |
| `Get-Process cargo,rustc,mbx` | 探测时无活动构建进程；迁移前需再次核对 |

## 复现与反证

沙箱内 `mbx doctor --json` 报 D 盘不可写；在沙箱外同命令 cache pass、0 failures，证明此报错来自执行权限边界，不是磁盘 ACL 故障。禁止因此放宽 D 盘 ACL。

沙箱外原始 PATH 的 doctor setup warn：`Cargo shim is current but not active`。

仅在单次诊断子进程中前置 `C:\Users\lyh\AppData\Local\mbx\bin` 后：

- `Get-Command cargo` 指向 shim；`cargo --version` 保持 Cargo 1.98.0。
- doctor setup pass：`Cargo shim is active and current`；cache/config pass。
- 仍有旧 C 盘 target 的跨盘 restore 警告；本次未运行构建或持久修改配置。

## 项目入口

- `justfile:260` Windows build recipe；`:267` 普通 `cargo build -p {{BIN}} --timings`。
- `justfile:1302` UI 后端入口委托 UI justfile。
- `ccr-ui/justfile:551` Windows 后端 release recipe；`:553` 普通 `cargo build --release --timings`。
- `ccr-ui/package.json:44` tauri:check 使用普通 Cargo。
- `.cargo/config.toml:4` build 配置只有 incremental；没有 wrapper/target-dir。

这些入口均可由正确的进程 PATH 接入，无需全仓替换 Cargo 命令。Bun/Vite 非本次 mbx 适用对象。

## 官方依据及迁移注意

- https://mr-boxington.jdx.dev/setup ：setup 安装 shim 但不编辑启动环境；应验证实际 Cargo 解析。桌面应用需要继承正确 PATH，改变后需重启旧应用。
- https://mr-boxington.jdx.dev/configuration ：cache_dir 默认包含 actions、targets、shims；target.root 默认随 cache_dir。
- https://mr-boxington.jdx.dev/managed-targets ：已有 mbx link 在下次构建改变 root 时迁移；跨文件系统 rename 失败会删除旧输出而非复制。adopt 跳过已是 link 的 target，因此不能直接用 adopt 搬迁根 target。
- https://mr-boxington.jdx.dev/installation ：Windows NTFS 恢复可能复制；不要把 doctor 的 hard-link 检测当成实际恢复模式保证。

无需重装 mbx，也不应通过 `RUSTC_WRAPPER=mbx` 猜测其接入契约。
