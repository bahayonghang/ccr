# GPUI 原生桌面版本

## Goal

建立可逐步替代现有 Tauri 的独立 GPUI 桌面版本，在完整业务能力、现代简洁的 Codex 风格与可测性能之间取得可维护的平衡。用户无需为新界面迁移或重建已有业务数据。

## Background

研究基线为 `dev@1dfe776b`，工作区开始时干净。现有 Rust workspace 版本 7.5.0；Tauri 独立包已名为 `ccr-desktop`（`ccr-ui/src-tauri/Cargo.toml:5`）。命令清单记录 340 个通用命令、348 个 Windows 命令及 278 个 typed contracts（`ccr-ui/src/api/generated/command-manifest.json:1`）。这些包含壳操作和兼容入口，不能等同于 348 个独立产品功能。

源码证据与依赖调查见 [research/architecture.md](research/architecture.md)，逐项迁移范围见 [research/feature-matrix.md](research/feature-matrix.md)。动态路由实际展开为 76 条记录，其中 18 条重定向。用户提出的是新版本，本方案以完整功能迁移作为父任务验收范围，以十个独立里程碑控制每次变更规模。阶段性交付必须标明能力状态，空外壳不算完整桌面版本。

## Requirements

| ID | 用户结果 / 约束 | 所有者 |
| --- | --- | --- |
| R1 | GPUI 代码位于独立 `ccr-desktop/`，新 package/bin 为 `ccr-gpui`；与 CLI/Tauri 可并存 | 01 |
| R2 | 所有新公开开发命令使用 `desktop-*`，含 build/dev/check/test/fmt/lint/ci；不更名或覆盖已有命令 | 01、10 |
| R3 | 新依赖采用核查时最新兼容稳定版本，记录版本来源、必要例外、MSRV、features 和独立锁文件 | 01、10 |
| R4 | 应用层复用 Rust 领域服务，无重复用量 SQL、profile 持久化、认证或进程安全逻辑 | 02、04 至 09 |
| R5 | 按 `style:codex` 与 ui 技能设计原生 UI，轻重层次清晰、完整输入/焦点/空错误状态、中文优先并支持英文 | 03 |
| R6 | 覆盖现有配置、平台设置/认证、资源、用量/会话、任务/同步/签到、Local/WSL/SSH 与桌面集成能力 | 04 至 09 |
| R7 | 以同机同数据 release 基线证明性能，后台 I/O 不阻塞交互，大列表与流日志有上限 | 01、02、07、10 |
| R8 | Windows/macOS/Linux 分别提供构建与原生运行证据；GPU、IME、DPI、键盘与可访问性不能靠单平台推断 | 01、03、09、10 |
| R9 | 保留现有数据格式、备份、脱敏、CAS/原子写、并发和取消语义，迁移过程可回退到 Tauri/CLI | 02、04 至 10 |

## Acceptance Criteria

- [ ] AC1 (R1,R2)：独立目录、package/bin、workspace、lockfile、target 和打包身份可识别；`just --list` 无名称冲突，所有 desktop 入口都只路由至新工程。根默认 CLI 构建不编译 GPUI。
- [ ] AC2 (R3)：每个直接依赖有核查日期、稳定版本、选用版本及偏差原因；执行冻结锁文件构建，重复核心图形后端与可选依赖有审计记录。候选 gpui-kit 0.7.3 需经过真实解析/编译，不把网页信息当构建结果。
- [ ] AC3 (R4,R9)：共享层没有 tauri/gpui 依赖；Tauri 回归与新 native fixture 均通过。用量 SQL 仍只由 ccr-usage 持有。相同 profile 的两客户端并发编辑触发冲突或有序写入，不能静默覆盖。
- [ ] AC4 (R5,R8)：实际原生窗口在 1280×800、960×800、1280×640、960×640 及 100%/150%/200% DPI 检查；light/dark、中文/英文、键盘、IME、焦点、loading/empty/error/disabled 均有记录。规范为 ccr-desktop/DESIGN.md，Web/HTML 示意不能替代。
- [ ] AC5 (R6,R9)：迁移台账逐项映射现有 commands 与有效 route；业务项有真实 native action、状态反馈及测试，Tauri 专属壳项有 native 对应或书面理由。无 owner、无测试、仅占位页面的项不得计为完成。别名允许合并，业务能力不得默默删除。
- [ ] AC6 (R7)：满足 research/performance.md 的明确预算，提供原始样本、环境、版本、数据哈希与分位数；未测/不支持/失败分别记录，不声明性能达标。
- [ ] AC7 (R8)：Windows/macOS/Linux 的支持清单、构建产物和原生人工/自动检查可追溯；WSL 只在 Windows 暴露。缺少目标 OS 运行证据时保持预览状态。
- [ ] AC8 (R2,R9)：desktop 独立质量门通过，共享变更通过既有 narrow checks 和最终 just ci；版本同步、依赖审计、命令图及 CI 路径规则包含新增表面且不降低现有阈值。

## Out of Scope

本任务不删除 Tauri、不重写 CLI、不改变业务数据库 schema、不新增移动端/Web 交付、不引入 JavaScript 插件运行时或 WebView、不顺带改造 VS Code 插件。现有业务能力的 UI 合并不构成功能删除；已有自动更新/托盘等能力须在 09/10 中逐项评估。上传公开安装包、发布或签名密钥配置不在本地实现的自动操作范围内。

## Key Decisions

采用独立 Cargo workspace，按业务域迁移而非重写整个后端；先验证 GPUI 原生能力再扩展业务。目录与 Cargo 包不同名用于避开现有占用。设计源采用用户指定 style:codex，系统字体替代需下载的字体，减少首屏成本。性能预算是拟定验收合同，尚无测量结果。

## Artifact Status

2026-10-17 用户已明确批准父任务及全部十个子任务，授权从 01 按依赖顺序实施。父任务与 01 已通过 task.py start 激活；其余子任务等待各自依赖满足。PRD、技术设计、执行计划、研究、上下文清单与 DESIGN.md 为已批准合同。实际完成状态以各子任务 evidence.md 和逐项验收为准，不能将方案批准解释为实现或验证完成。
