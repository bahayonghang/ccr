# CLI 与 Tauri 架构任务行政归档

用户授权：请提交改动和归档任务。

源码提交：f22732fc319a11adc38338603ea627def13c6348。分支：dev。本轮仅本地提交，未推送。

## 生命周期与验收

父任务和 11 个子任务按用户要求归档。Trellis 生命周期标记为 completed。整体技术验收仍为 partial_required_acceptance_remains_open，归档不补全缺失的验收证据。

Windows just ci、前端、VS Code、docs 及 Linux workspace/Tauri/coverage 门禁已有通过记录。本轮复用冻结验证收据，仅执行提交、归档与完整性核对。

## 保留的验收限制

- 必需的 macOS process/workspace matrix 未执行。
- 历史 exporter 0xc0000005 原因未查明，失败前 exe SHA 缺失。后续通过记录不替代历史失败。
- 原生验证限于 Linux WSLg WebKitGTK、debug binary、生产 custom-protocol/CSP 和合成本地 Claude raw settings。
- 保存后 style DOM 移除、body scroll-lock 恢复未验证。
- Windows/macOS 原生交互、Profiles/Configs/Commands 原生交互、真实 SSH/远程 WSL/OAuth、交互式 TUI、物理输入及发布包未验证。
- 部分旧基线反例只有源码证据；审查作者独立性以各报告记载为准。
- 12 个 context validations / 24 个 JSONL 保留 4 条注入大小警告：33513 > 32768 字节。

## 历史证据和路径

closure.json 保存原始父子关系和 12 个旧目录到归档目录的映射。历史 JSON、报告和脚本中的旧路径表示执行时位置；读取证据时按该映射解析，不批量改写历史路径。原有 committed=false、fix_commit=null 等字段保留为提交前快照。当前源码提交见本关闭记录和 research/source-commit-receipt.json。

research/archive-integrity.json 记录归档前后的原始文件 SHA256 核对结果。仅 12 个 task.json 的生命周期、提交和关闭元数据按归档流程更新。历史证据文件保持原始字节。Git 文本规范化规则继续生效，原始文件 SHA256 不等同于规范化后的 Git blob 哈希。

中央 requirements-evidence.json 纳入归档提交。7 个大型原始证据或编译产物以及已忽略的日志随目录保留在本机，不纳入 Git；路径、大小和 SHA256 见 closure.json。需要复现实验时仍须使用这些本地原始产物。

## 保留的其他工作

57 个图标改动和 2 个原有临时脚本保留在工作区，不纳入本任务提交。两个 Insights 任务保持原状。未执行 push、PR、任务验收补签或长期记忆写入。

## 归档暂存检查

完整归档 diff 检查返回 2，包含 7 个历史 unified-diff 证据文件的 28 条空白提示。逐行核实后，28 行均为补丁格式所需的空上下文行标记（单个空格）。原始证据保持不变；其余归档文件检查返回 0。详见 research/archive-index-review.json。源码提交的暂存检查已通过。
