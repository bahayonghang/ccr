# 显式主机系统信息 consumer 收据

## 范围与领域口径

现有 Global Monitoring 页新增独立“读取系统信息”区域，没有新 destination、窗口或主题系统。日志刷新与主机读取互不触发；进入页面也不采集主机。父页 suspend 委托子 entity 清理 task、事实、错误与 busy，并递增 generation 拒绝迟到结果。

`runtime/monitoring/system.rs` 通过已有 bounded read executor 使用精确 `sysinfo=0.39.6`（default-features=false/system）。该 kernel crate 已由 ccr-app 使用；native Cargo.lock 仅增加 ccr-gpui 的直接 sysinfo 边，未增加另一版本。Root/Tauri 源、manifest、锁及构建输入未修改。

对照 `ccr-ui/src-tauri/src/commands/system.rs::get_system_info`：

- OS 名称/版本、内核、hostname、CPU 品牌、uptime 由 sysinfo 获取；架构沿用编译期 ARCH，UI明确称“应用架构”。
- CPU 仅调用 CPU refresh 两次，间隔至少 MINIMUM_CPU_UPDATE_INTERVAL，使用逻辑 CPU 使用率的算术平均；显示实际观测间隔与 UTC 采样完成时间。不把逻辑数称物理核心，不把观测间隔称操作延迟或 p95。
- 只构造 `System::new()` 并刷新 CPU/memory，不使用 `new_all` / `refresh_all`，不枚举进程、执行 CLI、启动网络或后台轮询。
- RAM/swap DTO 保留原始 bytes，展示为 1024³ 的 GiB。容量缺失或报告为0时保持 Option::None，不制造0容量/百分比；used 大于 total 时不发布无效比例。CPU 空样本/非有限/越界值保持未知，真实有效0使用率仍可表示。
- 主机 RAM 与既有 current-process RSS 分列并明确标注。所有可选缺值可读地显示不可用。

`Services::system_info` 在 probe 闭包前拒绝非 Local 与 `services.is_fixture()`，后者包括新 Benchmark。没有真实主机 probe 的 fixture fallback。运行采集只在显式点击后进入 blocking worker，双采样等待期间检查取消；它不是 GUI 线程 sleep。

## 测试与实际结果

Windows，pinned Rust1.98，desktop 独立 target，`--locked --offline`，默认并行，保留 `--skip export_bindings`。当前测试 artifact `ccr_gpui-4a39d1c098e6c17e.exe`。

| 检查 | 结果 |
| --- | --- |
| `cargo test --lib monitoring::system` | 5/5：fixture/remote fail-before-probe、actual request拒绝、缺值/CPU/比例、GiB单位、真实控件read/busy/拒绝fixture/迟到结果/suspend/无poll |
| `cargo test --lib features::monitoring::tests` | 原 Monitoring 2/2；原筛选/事件与13 destination可达性保留 |
| 既有 `environment_views_fit_native_windows_stack_with_isolated_locks` | 1/1 wrapper，内部20个真实控件子进程实际通过，RUST_MIN_STACK=1048576 + child-only poisoned lock root；包括新 system control |
| 显式本批 rustfmt / diff-check | PASS |
| desktop strict Clippy all-targets | 当前被 peer 三处 lint 阻断：benchmark.rs cmp_owned、lib.rs collapsible_if、platforms/diagnostics.rs large_enum_variant；已报对应 owner，无本批 system lint |

UI成功数据使用仅 cfg(test) 的 synthetic DTO + 同一 bounded executor；测试不会调用真实主机 collector。Benchmark owner 已把实际 Benchmark system_info 拒绝加进其 child 验收。没有运行原生窗口、真实 Local 系统信息采样、SSH、私钥读取或硬件矩阵；原生验收仍 user-deferred。以上是源码/编译/headless与1MiB证据，不是原生或真实硬件实测收据。

## 文件与规范

- 新 runtime/monitoring/system.rs 与 tests：类型、门禁、双采样、取消和缺值投影。
- 新 features/monitoring/system.rs 与 tests：显式读取、独立事实、safe error、生命周期。
- features/monitoring.rs 只增加子 entity、挂载、suspend。
- runtime/monitoring.rs 的模块声明由 benchmark owner 添加，未改其 captured Benchmark path 分支。
- native manifest/lock 直接依赖边；既有 environments 1MiB guard 登记新控件。
- `.trellis/spec/ccr-app/backend/shared-desktop-services.md` 补充独立主机事实、单位、双采样、拒绝fixture/benchmark和隐藏不采集合同。

未提交。独立 check 与整合门禁由主会话继续；peer lint 修复后的最终结果需补入总收据。

后续补充：fixture_check 已报告 desktop `--lib --tests` 严格 Clippy 退出0，原三项 peer lint 已解除。该收据发生在 SSH 控件新模块接合之前，不作为后续 SSH 改动的门禁证据。
