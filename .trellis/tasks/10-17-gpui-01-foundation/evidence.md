# GPUI 01 implementation evidence

Date: 2026-10-17. Scope: foundation only. This is implementation of the approved task,
not completion of the desktop migration. No commit, push, package publication, signing,
task archival, user configuration mutation or global software installation was performed.

## Environment and ownership

- Git base/current HEAD: `1dfe776b29fc6b52c8a9d0a6c8e49efbb8026d48` (`dev`); existing untracked
  planning/design/task directories were retained. Root/Tauri manifests and lockfiles were unchanged.
- Windows 11 build 26300, x86_64-pc-windows-msvc. Rust development toolchain 1.98.0;
  installed MSRV compiler `rustc 1.95.0 (59807616e 2026-04-14)`; Cargo 1.98.0; just 1.59.0;
  Python 3.14; PowerShell 7; Pester 6.2.0. Cargo resolves through the machine's existing mbx shim.
- MSVC 14.44/14.51 and Windows SDK 10.0.26100.0 detected. GPU inventory includes NVIDIA
  RTX 5090 Laptop driver 32.0.16.1714 and Intel Graphics 32.0.101.6733. The read-only inventory
  is not rendering validation; see the separate native probe receipt below.
- Implementation owns `ccr-desktop/` except approved DESIGN.md; root justfile/.gitignore/code_map;
  version scripts/tests/catalog; this evidence. Main session owns parent inventory/task states,
  performance baseline and native computer operation. No concurrent writer's changes were reverted.
- Loaded Trellis start/continue/before-dev, child PRD/design/implement/context lists, dependency
  governance, architecture/performance contracts and existing DESIGN.md. Applied design-md
  implementation guidance; main supplied the loaded ui skill's native constraints. The referenced
  `C:/Users/lyh/.codex/RTK.md` was absent; direct ui skill read in this child was denied, while main
  successfully read it and supplied the relevant constraints.

## Delivered behavior

- Independent `[workspace]`, `ccr-gpui` package/bin, own Cargo.lock and `.cargo/config.toml`
  `target-dir = "target"`. Native identity `io.github.ccr.gpui`; window title `CCR Desktop (GPUI)`.
- Kit/platform initialized before Root/window; real retained InputState/Input with Unicode preview,
  retained event subscription, system font and initial approved semantic color mapping. Closing the
  last window exits. Probe performs no file/database/network/process business operation.
- Fourteen `desktop-*` public recipes in root and child. Root enters child justfile cwd;
  existing CLI, ui-* and tauri-* recipe behavior is unchanged.
- Every Cargo build/run/check/lint/test entry validates workspace_root, target_directory and
  (new Cargo) build_directory before compiling. Rejects foreign target/build overrides, including
  links resolving outside the exact workspace target. Clean validates metadata then passes an
  explicit absolute `--target-dir`; it never recursively deletes an inferred filesystem path.
- Windows desktop-only recipes explicitly return `$LASTEXITCODE`, preserving the originating
  failure across both justfiles. `desktop-ci` is ordered read-only validation, not a repair alias.
- Both version scripts register, require, compare, report and explicitly sync the independent
  package version. Suites construct GPUI and UI README fixtures, use fixture-local script location,
  and exercise GPUI drift/check/sync. PowerShell tests use their current no-profile host.

## Actual commands and outcomes

Generated local logs and metadata are retained under `outputs/desktop/foundation/` (ignored
rebuildable evidence). Commands below ran from repository root unless a cwd is specified.

| Command / operation | Exit | Result / retained evidence |
| --- | --- | --- |
| official crates.io API query `https://crates.io/api/v1/crates/gpui-kit` | 0 | max_stable/newest 0.7.3, not yanked, Apache-2.0, rust_version null; `gpui-kit-registry.json` |
| `cargo generate-lockfile` (desktop cwd) | 0 | 826 resolved dependencies plus application; latest Rust 1.95 compatible resolution; no floating git/patch |
| `cargo metadata --locked --no-deps --format-version 1` (desktop cwd) | 0 | root `.../ccr-desktop`, target/build `.../ccr-desktop/target`; `workspace-metadata.json` |
| `cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc` | 0 | full selected-target metadata; `metadata-windows.json` |
| `cargo tree --locked --edges normal --prefix none` (desktop cwd) + forbidden graph assertion | 0 | 920 entries, no gpui-fast or tauri; `normal-tree.txt` |
| `just --list` / `just --dry-run desktop-build` / `just --dry-run desktop-dev` | 0 / 0 / 0 | parse success, unique public entries and child forwarding; `just-list.log`, `dry-run-{build,dev}.log` |
| `just desktop-doctor` (authorized read-only hardware query) | 0 | tool versions, isolated metadata, MSVC/SDK paths and actual GPU/driver inventory; `doctor.log` |
| `cargo check --locked --all-targets` (desktop cwd, Rust 1.98) | 0 | Windows all targets; repeated within desktop-ci |
| `cargo clippy --locked --all-targets -- -D warnings` (desktop cwd) | 0 | strict lint; repeated within desktop-ci |
| `cargo build --locked --bin ccr-gpui` (desktop cwd) | 0 | 2m33s initial debug build; `build-debug.log`; actual exe used for native interaction |
| `just desktop-fmt-check` | 0 | read-only formatting validation; explicit `cargo fmt --all` ran separately during implementation |
| `python -m unittest discover -s tests -p 'test_*.py'` (desktop cwd) | 0 | 5 tests, including 21 real foreign-env entrypoint rejections and exact exit-23 propagation; `command-tests.log` |
| `just desktop-test-ui` | 0 | 1 real Kit headless interaction test, Unicode/focus/redraw/backspace/close; `test-ui.log` |
| `cargo +1.95 check --locked --all-targets` (desktop cwd) | 0 | 2m42s on actual Rust 1.95.0, not inferred from resolver; `msrv-check.log` |
| `just desktop-ci` | 0 | fmt-check/check/lint/Python 5 tests/Rust native 1 test/release build passed; release compilation 5m05s; `desktop-ci.log` |
| `just desktop-audit` | 0 | fetched RustSec, 1338 advisories, 827 lock packages; 8 unmaintained upstream warnings retained; `audit.log` |
| `./scripts/version/version-sync.ps1 -Check -Verbose` | 0 | GPUI version 7.5.0 matches root and existing targets |
| `just version-check` | 0 | version/docs-lock/root-Tauri dependency/MSRV checks; `version-check.log` |
| Pester `Invoke-Pester -Configuration` with task script path, `TestRegistry.Enabled=false`, current no-profile host | 0 | 7/7; no registry fixture is needed for file version tests; `version-pester.log` |
| Git Bash `bash -n scripts/version/version-sync.sh` | 0 | Bash syntax validation |
| Git Bash portable `outputs/desktop/tools/bats-core/bin/bats --print-output-on-failure scripts/version/version-sync.bats` | 0 | 8/8; `version-bats.log`; official Bats v1.14.0, commit eb7f42f8d608ac693d7a4b67474f6714ea68cfc5, local ignored tool only |
| `git diff --check` | 0 | no whitespace errors; Git CRLF working-copy notices are informational |
| `just desktop-dev` (main, authorized existing-cache access) | 0 | Actual root entry -> child preflight -> cargo run -> visible native probe -> Alt+F4 -> normal wrapper exit; `research/native-probe.md`, `probe-desktop-dev.png`; initial sandbox cache denial exited 1 and remains recorded |

Root normal CLI graph was independently checked by main: locked offline metadata still defaults to
`ccr`, and normal `cargo tree -p ccr` contains no GPUI/Tauri. See parent implementation evidence.

## Audit details and preserved failures

RustSec revision: `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`, fetch succeeded. Audit warnings:
instant 0.1.13 (RUSTSEC-2024-0384), paste 1.0.15 (2024-0436), rustls-pemfile 2.2.0
(2025-0134), rustybuzz 0.20.1 (2026-0206), ttf-parser 0.25.1 (2026-0192), zbus_names
4.3.5 (2026-0371), zvariant 5.15.1 (2026-0370), zvariant_derive 5.15.1 (2026-0372).
No advisory ignore or threshold relaxation was added. An optional package's lockfile presence
does not establish activation in the normal Windows graph.

Earlier failures were repaired and rechecked: initial probe syntax trailing comma (fmt failed),
PowerShell fixture missing required targets and old host startup policy, Bats fixture root derived
from BASH_SOURCE under the runner, and Windows native exit-23 becoming exit-1. The last defect
was caught by an actual stub subprocess, not merely a dry-run assertion. Initial registry/WMI
requests were denied inside the sandbox; authorized read-only retries succeeded. First Pester
attempt could not create an unrelated HKCU TestRegistry fixture; the suite uses no registry,
so the documented Pester config disables that unused fixture. All final checks above retain
their actual outcomes; earlier failure is not represented as pass.

## Native receipt and acceptance

Main performed actual Windows GPU/window/Chinese IME/focus/selection/close operation.
Full receipt: [research/native-probe.md](research/native-probe.md).
Screenshots/raw capture: `outputs/desktop/native/windows/` (`probe-unicode-input.png`,
`probe-ime-composition.png`, `probe-ime-committed.png`, `probe-keyboard-selection.png`,
`probe-capture.json`). Tested debug executable SHA256:
`7d58450248e04ac958d464723f12ef01e838dbaf6422de27a1a9f7eb71d9af4d`.
Cargo.lock SHA256: `f05306cb38264a91afd699843ab7e67f380e180698ed31b9e39d6f62fd39cacd`.

| AC | Status | Evidence / limit |
| --- | --- | --- |
| 1.1 | pass (Windows graph) | Independent metadata/lock/target; no Tauri/Fast normal graph; root graph checked by main |
| 1.2 | pass (Windows) | actual desktop-dev native launch/normal exit, release build and exact failure propagation; main receipt records both direct exe and public entry runs |
| 1.3 | pass (Windows contract) | ci has no implicit repair; tests default parallel with --skip export_bindings; metadata plus env/foreign-path rejection tests guard clean/build isolation |
| 1.4 | pass (Windows) | complete direct dependency decision, checked registry, locked debug/release + both compiler checks |
| 1.5 | partial / external blocker | Windows native input receipt exists; macOS compile receipt absent; subsequently discovered WSL Linux probe is tracked separately in research/linux-probe.md, not inferred from Windows |

Release executable: `ccr-desktop/target/release/ccr-gpui.exe`, 21,417,472 bytes, SHA256
`8d179ea6c5d9b6f5d243c8c0503aabb1c4f568891e328c380591bc980da1f4ec`.

Product writer ownership was handed to the Trellis check role after implementation and passing
Windows gates. The final independent check report belongs under `research/check-final.md`.
Implementation and independent check are now settled; main session resumes evidence ownership.
Independent check report: [research/check-final.md](research/check-final.md), Windows AC1.1–1.4
verified, 01 kept open for AC1.5 and actual baseline/probe measurements.

## Not run / remaining gates

- macOS build/native receipt remains absent. Later WSL Linux probes are recorded below and in
  research/linux-probe.md; they do not establish independent Linux desktop or macOS native support.
- The four requested logical sizes and 100/150/200% effective DPI were not validated; capture
  is decorated 1282×831 and a resize attempt did not change it. light/dark complete matrix,
  full keyboard navigation and OS accessibility remain unverified. Native accessibility capture
  returned null; 03 must investigate rather than treating headless metadata as native support.
- Tauri release baseline and GPUI performance samples remain `not_run` under main's ownership.
  No S1/S2 fixture samples, startup/idle/frame/latency comparisons or budget pass is claimed.
- Clean is boundary-tested; production target was retained so compiled/native evidence remains
  available. The destructive command was not run against actual root/Tauri outputs.
- Parent command/route inventory updates and subsequent tasks remain main-owned. No migration
  item is marked complete by this feasibility page; no placeholder business page was introduced.
- Final full root `just ci`, independent packaging and complete migration acceptance belong to
  the later integration milestone and were not run or claimed by foundation.

## Main-session integration and late narrow correction

- `just fmt-check`：退出 0，5 项 JSON formatting tests、11 个受治理 JSON、根及 Tauri Rust 格式检查通过。没有隐式 fmt/version-sync 修复。
- `just ci-governance-check` 首次退出 1：19 个测试失败来自 Windows legacy WSL bash 的 `Bash/Service/CreateInstance/E_ACCESSDENIED`，不认作产品代码失败或 pass。仅在该次进程 PATH 优先使用已有 Git Bash 后，48 项 workflow/aggregate tests、47 个 immutable action refs、serial-only count 0、6 项 dependency tests、19 个重复依赖与 0 例外均通过；随后 Tauri inventory 子门禁因 sandbox 对现有 mbx cache 拒绝访问而退出 1。该次日志 `outputs/desktop/foundation/root-ci-governance.log` 保留失败。
- `just tauri-command-inventory-check` 必要 cache 权限重试：退出 0，实际 handler inventory 测试 1/1，608 项无关测试被筛选，没有运行真实用户配置测试；日志 `root-tauri-inventory.log`。聚合最终回执另见 `root-ci-governance-final.log`，只有真实最终退出码可判该聚合结果。
- `just ci-governance-check` 最终在进程内 Git Bash PATH + 必要既有 cache 权限条件下完整复验：**退出 0**；48 项 workflow tests、47 refs、serial count 0、6 项 dependency tests、19 重复依赖/0 例外、实际 Tauri handler inventory 测试 1/1 全部通过。日志 `root-ci-governance-final.log`。未更改全局 PATH、阈值或 hosted context 名称。
- WSL Linux 真实 all-targets check 已通过；初次 release 链接实际缺 `-lxkbcommon-x11`。系统有 runtime `.so.0`，开发 `.so`/`.pc` 不存在。研究、原始失败及经授权的 outputs-local linker 入口恢复详见 [research/linux-probe.md](research/linux-probe.md)，不能将局部恢复冒充标准 doctor 通过。
- Trellis check 对这个真实缺口窄修 Linux doctor：增加 `xkbcommon-x11` 的 pkg-config 先决条件与错误说明；同步 README，依据官方 Ubuntu 文件清单核实模块名。没有自动安装。新增缺包失败回归和全部 Python **6/6** 通过（32.592s），保留既有 21 条 override 和 exact exit 23 验证。初次 mock 失败及修正均在 [research/check-final.md](research/check-final.md) addendum 记录；未因纯 Python/文档更改重编 Rust。
- 后续可访问性调研确认该锁定栈本就有 Windows AccessKit/UIA 接入，不需要猜测新增 Cargo feature。实际 helper null 仍是未验证结果，不能宣称 transport 或 disabled/read-only 状态通过。定位与恢复步骤见 [research/native-a11y.md](research/native-a11y.md)。
- 主会话另外尝试 Linux release WSLg 原生运行，出现 EGL/ZINK 驱动警告且 helper 未找到可验证窗口；只清理经过两次路径核对的本次探针 PID，session 最终退出 1，确认无残留进程。该次不计正常窗口/输入/关闭通过，详见 [research/linux-native-probe.md](research/linux-native-probe.md)。
- Linux probes 最终收尾：Rust 1.98 all-targets check **0**；标准 release **101**（缺开发 linker 入口）；授权 outputs-local symlink + 单次子进程 LIBRARY_PATH 条件下 release **0**（4m49s），43,643,864 bytes，SHA256 `b4c57ff3a32e63e709b70ebca866b140373821705b55bd18d3892ac7fd7477ce`。`ldd`/`readelf` **0**，runtime 全部解析，无临时 RPATH/RUNPATH。已有 stable 别名实际为 Rust **1.95.0**，`cargo +stable check --locked --all-targets --target x86_64-unknown-linux-gnu` **0**（1m46s），该次没有 LIBRARY_PATH override。最终 Linux doctor **1**，准确报告 just 与 xkbcommon-x11.pc 缺失。所有阶段 lock hash 未变；未安装系统包或工具链。见 Linux 报告和 14 个已解析通过的 JSON receipt。

01 继续保持 in_progress；用户对既有父子方案的实施授权仍有效。当前证据没有解除所有 01 验收条件，后续业务子任务未作为已完成或已解锁交付计入。
