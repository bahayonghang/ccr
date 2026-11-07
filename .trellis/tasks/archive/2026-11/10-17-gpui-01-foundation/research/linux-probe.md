# Linux 编译探针（WSL，2026-10-17）

范围：对当前独立 `ccr-desktop` 工程进行 Linux 工具链、系统库、冻结依赖编译验证。只写本研究文件和 `outputs/desktop/foundation/linux-*` 证据；未修改产品源码、Cargo.lock、全局工具配置，未安装工具链/系统包，未运行应用或 native tests。

最终结果：Rust 1.98 与已安装的 Rust 1.95 均通过 Linux all-targets check；标准 release 因缺开发链接入口失败，经过明确记录的 workspace-local linker shim 后 release 构建成功。doctor 仍准确返回非零。本报告不宣称独立 Linux 桌面或 macOS 原生验收通过。

## 环境

- Ubuntu **24.04.4 LTS**，glibc **2.39**；内核 **6.18.54.1-microsoft-standard-WSL2**，x86_64。
- 工作目录 `/mnt/d/Documents/Code/Github/ccr/ccr-desktop`。Cargo metadata 的 workspace_root、target_directory、build_directory 均正确指向此独立 workspace 及其 `target`。没有 clean、移动 target 或更改缓存位置。
- Rust **1.98.0 (88d9e12ae 2026-08-18)**，Cargo **1.98.0 (797e8a9bc 2026-08-05)**，LLVM **22.1.8**；host/installed target 为 `x86_64-unknown-linux-gnu`。
- `rustup toolchain list` 实际列出 stable、1.88.0、1.98.0；随后用 `rustup run stable rustc -V` 核实，**stable 实际为 Rust 1.95.0 (59807616e 2026-04-14)**。不能只凭缺少 1.95 命名条目判断未安装。既有 alias 的版本证据保存在 linux-existing-toolchain-versions.json；没有安装/更新工具链。
- WSLg 环境有 `DISPLAY=:0`、`WAYLAND_DISPLAY=wayland-0`；这些变量存在不证明 native GPU、IME、无障碍、DPI 或输入工作。
- `pkg-config` 1.8.1、`cc`/`c++` 已存在。Linux `just` 未找到，因此 doctor 实际返回 **1**；直接 Cargo 验证独立记录，不把 doctor 记为通过。
- 初次直接 WSL python 启动没有 login PATH，rustc/cargo/rustup 子命令为 127。随后仅为探针进程 prepend 已有 `~/.cargo/bin`，未更改 HOME/USERPROFILE 或持久 PATH。初次证据保留为 `linux-environment-direct.json` / `linux-preflight-direct.log`。

当前 commit 为 `1dfe776b29fc6b52c8a9d0a6c8e49efbb8026d48`，共享工作区含其他 agent 的 foundation 实施与检查修改，不是干净发布 checkout。探针开始的 `ccr-desktop/Cargo.lock` SHA-256：

`f05306cb38264a91afd699843ab7e67f380e180698ed31b9e39d6f62fd39cacd`。

## 实际命令与结果

外层通过 `wsl -d Ubuntu-24.04 -- python3 -c <probe>` 执行数组形式的 subprocess 命令，不执行 shell 拼接、不运行产品应用。WSL 服务在 sandbox 内列 distro 返回 `Wsl/EnumerateDistros/Service/E_ACCESSDENIED`；授权后的同一 Ubuntu 环境可以使用，不能将 sandbox 拒绝访问当成 distro 不存在。

| 命令（desktop cwd，除特别说明） | 退出码 | 证据 |
| --- | --- | --- |
| `uname -srvm` | 0 | linux-environment.json / linux-preflight.log |
| `rustc -Vv` / `cargo -V` | 0 / 0 | 同上 |
| `rustup toolchain list` / `rustup target list --installed` | 0 / 0 | 同上；仅名称列表不足以判断 compiler version |
| `rustup run stable rustc -V` / `rustup run 1.88.0 rustc -V` | 0 / 0 | 既有 stable = **1.95.0**，另一工具链 = 1.88.0；linux-existing-toolchain-versions.json |
| `python3 scripts/desktop.py doctor` | **1** | 缺 just；独立 target 验证成功，核心五项 pkg-config 检查没有报缺失；linux-preflight.log |
| 修正前置清单后重跑 `python3 scripts/desktop.py doctor` | **1** | 准确报告缺 just 与 xkbcommon-x11.pc；linux-doctor-final.json / .log；LIBRARY_PATH 为空，未跳过标准前置条件 |
| `cargo metadata --locked --offline --no-deps --format-version 1` | 0 | 独立 workspace/target/build 解析成功；linux-environment.json |
| `cargo check --locked --offline --all-targets --target x86_64-unknown-linux-gnu` | **101** | Linux registry cache 中找不到 gpui-kit；linux-check-offline.json / .log；锁 hash 未变 |
| `cargo check --locked --all-targets --target x86_64-unknown-linux-gnu` | **0** | 公开 registry 取包后实际检查成功，Cargo 报告 **2m 26s**；linux-check.json / .log；锁 hash 未变 |
| `cargo build --locked --release --bin ccr-gpui --target x86_64-unknown-linux-gnu`（标准环境） | **101** | 最终 `cc`/rust-lld 链接找不到 `-lxkbcommon-x11`；linux-build-release.json / .log；锁 hash 未变 |
| 同一 release 命令（仅子进程 LIBRARY_PATH 补 workspace-local linker 入口） | **0** | Cargo 报告 **4m 49s**；linux-build-release-retry.json / .log；锁 hash 未变；不是标准环境通过 |
| `ldd` / `readelf --dynamic`（Linux release 产物） | 0 / 0 | linux-binary-libraries.json；依赖均可解析，SONAME 为系统 runtime，没有临时 RPATH/RUNPATH |
| Linux `cargo +stable check --locked --all-targets --target x86_64-unknown-linux-gnu`（已核实 compiler 1.95.0） | **0** | Cargo 报告 **1m 46s**；LIBRARY_PATH 为空，未用 shim；linux-msrv-check.json / .log；锁 hash 未变 |
| 14 份 Linux JSON 收据解析、研究文件尾随空白/终态、release/MSRV 退出码与 lock hash 一致性检查 | 0 | linux-validation.json；没有 pending 终态或隐式修复 |
| `git diff --check -- .trellis/tasks/10-17-gpui-01-foundation/research/linux-probe.md` | 0 | 文件为 untracked，另有上行显式新增文件空白检查覆盖 |

## 系统库预检

`linux-system-libs.json` 保留逐项 `pkg-config --modversion` 的退出码和原文。fontconfig 2.15.0、freetype2 26.1.20、xkbcommon 1.6.0、wayland-client/cursor 1.22.0、wayland-egl 18.1.0、xcb/xcb-render 1.15、openssl 3.0.13 返回 0。`xcb-shape`、`xcb-xfixes`、`alsa`、`libudev` 缺 `.pc`，各返回 1。

额外 `.pc` 缺失只表示此 pkg-config 名称不可解析，不自动证明当前 GPUI graph 必须链接它、也不证明 runtime 动态库缺失。只有实际编译错误及被激活的依赖路径可以形成对应 build blocker；不会预先安装这些库或修改 PKG_CONFIG_PATH 绕过。

实际 all-targets check 已通过，包括 gpui-pre-linux、gpui-pre-platform、gpui-base/component 与本应用/test targets。其后的 release build 则在最终链接暴露实际缺失项 `-lxkbcommon-x11`。这与额外 `.pc` 预检项不同；不得把“检查通过”写成“release 链接通过”。

## 实际链接失败与局部恢复

首次 release 命令于 17:59:26Z 至 18:03:59Z 执行，Cargo exit **101**；`cc` exit **1**，`rust-lld: error: unable to find library -lxkbcommon-x11`。`linux-build-release.json`/`.log` 保存原始证据。`cargo tree --locked --offline --target x86_64-unknown-linux-gnu --edges normal --invert xkbcommon` 返回 0，真实 normal 依赖链为：

`ccr-gpui -> gpui-kit 0.7.3 -> gpui-pre-platform 0.3.9 -> gpui-pre-linux 0.3.9 -> xkbcommon 0.8.0`。

link 声明位于 WSL crate cache：`/home/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/xkbcommon-0.8.0/src/xkb/x11/ffi.rs:16`，内容为 `#[link(name = "xkbcommon-x11")]`。完整只读诊断见 `linux-linker-diagnosis.json` 和 `linux-runtime-abi.json`。

- 已有 runtime 文件：`/usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0.0.0`；系统 `libxkbcommon-x11.so.0` 指向该文件，但无 `libxkbcommon-x11.so`。
- `dpkg-query -S` owner：`libxkbcommon-x11-0:amd64`；版本 **1.6.0-1build1**。`libxkbcommon0` 同版本，`libxcb-xkb1` 为 **1.15-1ubuntu2**；`libxkbcommon-x11-dev` 未安装。
- `readelf`：ELF64 little-endian x86-64，SONAME `libxkbcommon-x11.so.0`；NEEDED 为 xkbcommon、xcb、xcb-xkb、libc 的系统 runtime。
- `nm -D --defined-only` 确认 crate 声明的四个 `xkb_x11_*` FFI 函数均存在，符号版本 `V_0.5.0`。这支持链接兼容性判断，但不替代实际运行行为验证。
- runtime SHA-256：`109a5c2095ab6bcc36f751d3c0deb3b066ecbfb664a36904b6ca212f987a408a`。

主会话在核查上述事实后明确授权最小可逆恢复：仅建立
`outputs/desktop/foundation/linux-native-libs/libxkbcommon-x11.so -> /usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0`，解析后仍为该已核实 runtime；仅对 retry Cargo 子进程前置
`LIBRARY_PATH=/mnt/d/Documents/Code/Github/ccr/outputs/desktop/foundation/linux-native-libs`。
原 LIBRARY_PATH 为空；未改 RUSTFLAGS、Cargo manifest/config/lock/source、ldconfig 或系统目录，也未安装开发包或制作空库。

本地 symlink、解析路径、runtime version/hash 与精确环境记录在 `linux-linker-shim.json` 和 retry 收据中。该目录属于 ignored 输出；恢复只补开发链接入口，不打包或替换系统 runtime。retry 已返回 **0**，产物为 `ccr-desktop/target/x86_64-unknown-linux-gnu/release/ccr-gpui`，**43,643,864 bytes**，SHA-256：

`b4c57ff3a32e63e709b70ebca866b140373821705b55bd18d3892ac7fd7477ce`。

`ldd` 全部声明的动态库可解析，`libxkbcommon-x11.so.0` 来自 `/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0`；`readelf --dynamic` 没有 RPATH/RUNPATH 或 outputs 路径，NEEDED 仍是系统 SONAME。两条只读检查均 exit 0，完整结果在 `linux-binary-libraries.json` 和 retry 收据。动态加载的 GPU/backend 组件仍需原生运行验证，不能从 ldd 推断。

依赖解析不代表启动应用。doctor 缺 just 的原始失败继续保留，不能用 shim 冒称标准 Linux 环境就绪。需要在标准 Linux 构建环境准备匹配的开发包入口后另做无例外验证；本轮没有安装包或修改全局 linker 配置。

产品 writer 随后修正 doctor 的 Linux 清单，加入 `xkbcommon-x11` 与 pkg-config 原始错误展示，并更新 README；本 agent 未写入这些产品文件。对修正后的真实 doctor 已复验：exit **1**，准确报告 `just: MISSING` 和 `xkbcommon-x11.pc` 缺失，isolated target 验证成功。收据记录脚本 SHA-256 `0b2b70423b1e8db8282e3d86991029e6d2515a40e7a6063705d3d219304e0ed1`，不使用 local linker env；详情见 linux-doctor-final.json/.log。

## 验证边界和恢复

这是真实 Linux 工具链针对 Linux target 的编译探针，但宿主是 Windows 的 WSL2。即使编译成功，也不能冒充独立 Linux 桌面的运行/打包证据，更不能替代 macOS 构建。没有启动 GPUI 或 Tauri，没有采集 Linux native screenshot、IME、焦点、无障碍、DPI 或性能样本；原生窗口由主会话统一掌管。

上述“未启动”与收据 `native_application_launched=false` 均指本构建探针 agent 的操作范围。主会话后来独占启动了单独的 WSLg 原生探针，其窗口/renderer/输入/退出结果由主会话另行记录，不由本报告代判。

如实际编译缺系统开发库，保留首次失败、列出 crate/构建脚本和精确库名，由后续有相应预装环境的 Linux 主机或明确批准的系统准备步骤恢复；本 agent 不安装系统包。若只缺公开 registry cache，则在保持 `--locked` 与 lock hash 不变的前提下允许网络取包。每次构建 receipt 记录起止时间、实际命令、退出码与前后 lock hash；未运行项保持 not_run。
