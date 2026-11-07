# Linux WSLg 原生探针尝试

日期：2026-10-17。主会话负责原生窗口；Linux build/MSRV 由另一 agent 记录在 linux-probe.md。本记录不替代独立 Linux 桌面或三平台运行验收。

## 对象和操作

- 已构建真实 ELF：`ccr-desktop/target/x86_64-unknown-linux-gnu/release/ccr-gpui`。
- SHA256：`b4c57ff3a32e63e709b70ebca866b140373821705b55bd18d3892ac7fd7477ce`；43,643,864 bytes。无临时 RPATH/RUNPATH，系统 runtime 动态库均解析，详见 build receipt。
- 实际命令：`wsl -d Ubuntu-24.04 --exec /mnt/d/Documents/Code/Github/ccr/ccr-desktop/target/x86_64-unknown-linux-gnu/release/ccr-gpui`。使用现有 WSLg；未修改 GPU、显示、系统库、HOME 或用户配置。
- 该程序仍为 fixture-only input probe，不读业务配置、不联网、不采样性能。

## 观察与结果

运行 session 59278 输出：

```text
libEGL warning: failed to get driver name for fd -1
libEGL warning: MESA-LOADER: failed to retrieve device information
libEGL warning: failed to get driver name for fd -1
MESA: error: ZINK: failed to choose pdev
libEGL warning: egl: failed to create dri2 screen
```

之后进程仍在运行，没有新的 stdout/stderr。Windows native helper `sky.list_windows()` 按精确标题 `CCR Desktop (GPUI)` 过滤返回空数组。未捕获到可验证的 Linux 窗口，未进行键盘、IME、焦点、可访问性或截图验证。helper 没找到窗口不单独证明框架不支持 Linux；上述驱动警告及可观测性缺口仍需在合适 Linux 图形环境排查。

只读 `/proc` 查询核实仅有本次 PID 11541，其 exe 解析路径与上述 ELF 完全一致。清理前再次核对该 PID 的 exe，才发送 SIGTERM，清理命令退出 0；原运行 session 59278 最终退出 **1**。这是外部清理，不能记为正常窗口关闭或应用生命周期通过。

结论：Linux 编译证据与原生运行结果分开；本次 native 尝试 **未通过验证**，没有原生截图或性能结论。不修改全局驱动配置来掩盖此结果，后续在有可靠图形/输入环境的 Linux 主机重新验证。
