# 首次真实呈现的锁栈边界 — 2026-11-06

只读调查实际依赖源码；未启动窗口、探针、ETW、进程扫描或性能试次。当前锁栈为 gpui-kit 0.7.3 → gpui-pre 0.3.9（非 Fast 默认路径），Tauri 2.11.5 / wry 0.55.1 / WebView2 COM 0.38.2。以下是工程边界，不是性能验收。

## GPUI

依赖源码根：`C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`。

- `gpui-pre-0.3.9/src/window.rs:1865` 的 `on_next_frame` 在下一轮 draw/present 前执行，不证明显示完成。
- 同文件 `:3604` 的私有 `Window::present` 包住 `platform_window.draw` 并记录 CPU 耗时。
- `gpui-pre-windows-0.3.9/src/directx_renderer.rs:245` 调用 DXGI `Present(0,0)`；没有向应用公开完成 token / 帧统计。`src/window.rs:1036` 将 renderer 的 Result 只作 log_err，PlatformWindow.draw 返回 void；上层 profiler 仍会记录 Present，因此记录也不能独立证明提交成功。
- Windows `src/vsync.rs:37` 的 DwmFlush 是后续 request-frame 节拍，且可能退回 sleep，不是该帧显示确认。
- `gpui-pre-0.3.9/src/profiler.rs:945` 把 PresentTiming 定义为平台提交耗时；`:1629` 的公开 FrameTimingCollector 可作 CPU 帧诊断。ring 溢出可丢项，不是完整原始样本保证。loss-aware journal 的 install 函数是 pub(crate)。本轮未确认应用级 Chrome trace exporter，不能写成已有协议。

## Tauri / WebView2

- `wry-0.55.1/src/webview2/mod.rs:643` 的 page-load 回调分别映射 ContentLoading / NavigationCompleted；它们不证明 React 首屏显示或可交互。
- `tauri-2.11.5/src/webview/mod.rs:1668` 的 with_webview 和 `:180` 的 controller 允许访问 WebView2；本轮没有找到可直接消费的 OS-present 完成事件。
- paint、rAF、DOMContentLoaded、navigation completed、main 返回或窗口对象创建均不能代替首次真实显示。

## 可审查的后续路径

保持现有窗口栈，可另做 Windows 外部 OS 观测 harness，关联 process-create、PID/HWND/交换链及显示事件；Tauri 还需归属 WebView 子进程。必须先证明能区别初始空白帧和应用首屏，再接受试次。上游若新增 hook，至少要区分提交成功、GPU 完成和 OS 显示，不能仅暴露当前 CPU profiler 数值。

action-ready 应由首个真实输入触发约定的首屏导航，handler 回传同一 trial nonce；记录输入/确认时间和探测间隔。仓库当前没有该协议。现有 `ccr-desktop/src/probe.rs:20` 仅用于输入变化可行性，不具备 process-create/first-present/action-ready 计时能力。

上述 OS 观测与真实输入验收依赖用户已延期的原生验证；本轮不启用 profiler feature、替换依赖或运行近似试次。动态 S2 输入源、共同报表时钟及隔离工程可独立推进。所有性能指标继续 `not_run/not_comparable`。
