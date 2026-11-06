# GPUI 01：独立工程与开发命令：执行计划

## Preconditions

无；开始前需要父任务最新方案审阅通过。 用户须审阅父任务最新方案。若批准后发现需要改变范围/兼容行为，重新评审变化部分。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 创建 ccr-desktop 独立 workspace、ccr-gpui bin、独立 lock/target；最小 native window 含真实输入控件。
3. 根与子 justfile 增加 desktop-help/doctor/dev/build/build-debug/check/fmt/fmt-check/lint/test/test-ui/ci/audit/clean；不改变已有入口。 核查最新兼容版本与 MSRV，完成 Windows/macOS/Linux 编译探针；先在可用 Windows 主机验证窗口、文字、IME、焦点与关闭。 采集 Tauri 基线及 GPUI 探针数据；版本脚本双端加入新 manifest，文档标明能力尚在建设。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- just --list；just --dry-run desktop-build；just --dry-run desktop-dev
- just desktop-doctor；just desktop-ci；just desktop-test-ui；just desktop-audit
- just version-check；相关 version-sync PowerShell/Bash 测试；根命令路由回归
- 三 OS cargo check/build 的 CI receipt；Windows native probe 截图和 IME/focus 检查

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；按用户 2026-11-05 续作授权，在独立 check 后本地提交当前任务改动与规划产物，不发布、不 archive 未完成工作。父任务最终验收负责全量整合。

## 2026-11-05 当前可执行边界

- 本轮只读复查当前 owning source 和宿主/已安装 target；不运行 doctor、WSL 或任何原生应用，不安装 target/工具。
- Tauri 当前 Windows 启动链无条件调用带缓存的 WSL 探测入口；未命中有效磁盘缓存时会执行 WSL，且整应用 DB/迁移/偏好/认证/WebView 隔离仍未接通；不得通过启动真实应用来填补基线。
- GPUI 已有配置/Insights/UI-state 的 fixture path seams，但不构成两端一致 S0/S1/S2 和真实 first-present/action-ready 收据。不得把空窗口或 Web paint 指标替代合同采样。
- 本轮仅更新任务文档与源码复查收据，不改产品代码、不重复既有 Windows gates。独立 check 核对源码定位、状态与文档差异；性能维持 `not_run`，相对比较维持 `not_comparable`。
- 未来恢复需依次接通整应用显式隔离、领域生成器和 OS 时钟/presentation/action instrumentation，之后才采同机 release 基线。macOS AC1.5 只消费真实对应机器收据，不能从 Windows 推断。

## 2026-11-05 产品工程续作

- 修复根/子 `desktop-dev` 的参数边界；Python script recipe 直接传递 argv，含空格的显式 fixture 路径不再经第二次 shell 文本插值。
- 增加 `desktop-package` 本机便携 ZIP：独立 `target/sidecar` 构建同版本 CLI、版本校验、仅 GPUI 构建进程注入 sidecar SHA256，双可执行文件与许可证/哈希清单原子生成。保留 release 命令工作台校验，不把单个 UI exe 当成完整可执行包。
- 实施验证：Python 入口全集 14 tests 退出 0；增加 sidecar 后 package 窄回归 2 项退出 0。独立 check 补齐构建前 sidecar 与 GPUI release 链接逃逸拒绝，最终 package 窄回归 3 项及七文件 `git diff --check` 均退出 0；未修改的 argv 与其余回归复用前序结果。根 `just version-check` 退出 0。
- 真实 release 构建/打包仍待集中执行，未认领产物或原生验收。不运行 doctor/WSL，不扩建性能测量基础设施，不以 Windows 产物关闭 AC1.5。
- 沙箱重映射 LOCALAPPDATA 会令 mbx shim 状态与实际 PATH 不一致，校验保持 fail closed。仅验证进程使用 `rustup which cargo` 已定位的真实工具链 bin，Git Bash 目录仍置首；不改全局 PATH、HOME、USERPROFILE 或工具配置。
