# 性能验收合同

状态：拟定预算，尚未采集基线或验证数值。数值作为首轮工程目标，失败必须分析并回到评审，不能直接放宽或宣称优秀。

## 可重复条件

记录 OS、CPU/GPU/驱动、内存、屏幕刷新率、DPI、电源模式、Rust/toolchain、git SHA、lockfile hash 和 fixture hash。两端均 release、关闭调试 overlay，使用同机同数据同操作，无真实账户。Windows 计整个应用进程树含 WebView；共享 WebView 进程若无法准确归属须另报限制，不能简单忽略。

S0：新 fixture、无用量记录。S1：100 profiles、10,000 sessions、100,000 usage records。S2：10,000 可见候选列表记录、持续 1,000 log events/s 60 秒。较大数据库单独记录，不能拿 S0 内存冒充 S1。

启动分别记录 process-create 到 first-present，以及 process-create 到首屏 action 可处理。自动计时需埋点真实首帧；main 入口时间不等于进程创建时间。热启动 30 次；冷启动在真实 OS cache 重置/重启条件下至少 5 次并报告中位数/max，不能将重复开关窗口称为冷启动。启动过程不得自动导入数据或联网。

## 预算与统计

| 项目 | 目标 | 测法与结果要求 |
| --- | --- | --- |
| 热启动可交互 | p95 ≤ 500 ms | 30 次 nearest-rank p95，保留所有样本 |
| 冷启动可交互 | median ≤ 1,200 ms | 至少 5 次，另列 max，不伪算可靠 p95 |
| S1 空闲 CPU | ≤ 单核 1% | 稳定后 120s 的进程树 CPU time / wall time；总机百分比须换算 |
| S1 常驻内存 | ≤ 200 MiB | 记录 Windows private bytes/working set、Linux RSS、macOS footprint，跨 OS 不混比 |
| S1 相对改善 | 热启动 p95 与同定义内存分别 ≤ Tauri 的 80% | 基线异常/无法归属时标不可比较，不判 pass |
| 10k 行滚动 | p95 frame ≤ 16.7ms，p99 ≤ 33.3ms | 固定 60Hz、60s 连续输入，记录 presented/long frames，不能用平均 FPS 替代 |
| 输入/导航反馈 | p95 ≤ 50ms | S1/S2 各 ≥ 100 个 action，输入到对应 UI 提交 |
| 100 次页面切换 | 增量稳定内存 ≤ 20 MiB | warm-up 后计首尾稳定窗口，同时检查 task/subscription 数归零 |
| 取消反馈 | UI ≤ 100ms | cancel_requested 立即反馈；进程清理沿既有五秒预算，绝不提前报告终态 |

## 机制对应

启动预算由 shell-first + 延迟加载实现；空闲预算由无持续动画/隐藏页轮询、局部 notify 实现；列表预算由 virtualization 与后台分页实现；S2 由 10Hz 合并快照、128 通道容量和 2,000 行/2 MiB 日志上限实现；泄漏预算由明确 task/subscription ownership 与关闭测试实现。性能 profiler 仅在采样构建启用，报告它是否影响被测数据。

## 输出

outputs/desktop/perf 下保存 JSON 原始样本及 Markdown 汇总：metric、unit、fixture、mode、samples、statistic、budget、baseline、ratio、result。结果只有 pass/fail/not_run/not_comparable。单个平台无法采样不代表三平台通过。首个里程碑提供探针和基线，最后里程碑以完整功能再测，避免拿空窗口代表最终产品。
