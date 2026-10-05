# 架构续作验收检查点

当前完整报告：integration-report.md。状态明细：execution-ledger.json。原始成功、失败和来源指纹均保留；本轮未提交、推送或归档。

## 已完成

- 两锁安全修复与审计通过；原 Tauri 9 条告警保留。4 项 Tauri all-targets lint、Codex model-only 磁盘往返、Settings 双语回归、Doctor 夹具就绪时序均已修正并经过非作者审查。
- 根 CI 的完整 workspace 测试、Release、安全审计与治理通过。该次 root aggregate exit 1：CLI exporter 返回 0xc0000005，未观察到该失败进程的 running 测试计数 banner，原因未查明。后续单次 bindings 和独立完整 Tauri 均 exit 0；407 行为测试、2 guard、230 exporters 和 1 inventory 通过，1 helper ignored。后续成功不改写原 aggregate 失败。
- 前端 168 files / 901 Vitest、i18n 24 / 4523 leaf keys、Lines 75.19% 通过；bun run build 实际 exit 0。采集器后续 Unicode 输出导致的 wrapper exit 1 与构建结果分开记录。
- Linux Rust 1.95 workspace check、Windows Rust 1.95 Tauri check 通过。Linux Rust 1.98 ccr-codex 与 Tauri 严格检查通过；core process 9、Tauri process 8、Doctor 3 及OAuth aggregate 20 通过（18 个完整 selector 可提取）。Windows 最后 cfg 修正后的 OAuth 1 项通过。以上均为各自作用范围，不代表完整 Linux lane。
- Web 合成 IPC 验证 model-only 单次 patch、通知数组及未知值保留、确认后原文编辑、纯 Web runtime unavailable 与重试。未执行原生桌面启动，没有截图证据。
- 最后只修正 Windows 专用测试的 Mutex 导入 cfg 条件，单行变更独立审查通过。263 路径（261 现存 + 2 删除标记）、230 类型、两保护脚本、HEAD 和任务状态已核对，diff-check 通过。新旧冻结清单及其来源补充分别记录，旧运行不改归新源码。

## 仍未完成

- 当前主工作区 frontend-check：两保护脚本共 5 条 no-console；原文件最小修改授权仍待用户回答。
- T06 必需 macOS 进程清理矩阵：尚无设备或 runner 信息，未验证，未改为可选。
- 原生 WebView/CSP：启动隔离不完整，未启动会访问真实 home 的桌面流程。
- 完整 pinned Linux lane 未执行；根 aggregate 启动访问冲突原因未查明。
- 专用 trellis-check 派发重试仍为 thread limit；非作者交叉审查不等同专用角色派发成功。

两次错误 OAuth 选择器匹配 0 个用例，不作为通过证据；按完整日志准确名称重跑后 Linux aggregate20 / Windows1 通过，Linux 18 个完整 selector 可提取。矩阵校验通过：39 AC / 42 测试组 / 9 P1 / 41 本轮运行 / 227 个哈希核验产物。该结果不替代完整门禁。矩阵和最终产物复核结果见 T10 research/requirements-evidence.json、evidence-mapping-validation.json、continuation-evidence-check.md。

所有架构父子任务保持 in_progress。真实 OAuth 登录、SSH 登录和交互 TUI 是明确未测试扩展，未要求用户登录。完整验收未授予通过。
