# 实施顺序与验证

## 状态与开始条件

2026-10-17 用户已明确批准现有父任务及全部子任务，授权从 01 按依赖顺序执行。父任务与 01 已正常 start；后续子任务仅在依赖满足后激活，不重复请求实施批准。父任务负责范围与最终整合，不直接让一个 agent 同时改全部模块。

| 顺序 | 子任务 | 前置依赖 | 独立交付 |
| --- | --- | --- | --- |
| 01 | gpui-01-foundation | 无 | 三平台构建探针、Windows 原生初验、独立工程和 desktop-* 基础命令 |
| 02 | gpui-02-services | 01 | 共享应用层、线程/生命周期边界、真实配置读取链 |
| 03 | gpui-03-shell | 01、02 | Codex 原生 shell 与输入/可访问性证据 |
| 04 | gpui-04-profiles | 02、03 | 配置与 profile 操作闭环 |
| 05 | gpui-05-platforms | 04 | 各平台设置与认证闭环 |
| 06 | gpui-06-resources | 02、03、05 | MCP 与扩展资源闭环 |
| 07 | gpui-07-insights | 02、03 | 用量/会话/观测闭环 |
| 08 | gpui-08-jobs | 02、03、04 | 命令/同步/签到任务闭环 |
| 09 | gpui-09-environments | 04、05、06、07、08 | Local/WSL/SSH 与系统生命周期 |
| 10 | gpui-10-acceptance | 01 至 09 | 完整台账、性能、三平台与交付 |

任务树本身不强制依赖，执行者必须读每个子任务 implement.md。可以在前置满足后调度不同功能，但共享 Cargo、ccr-app 契约和 DESIGN.md 只有当前负责者编辑，避免并行重写。dispatch 使用 Trellis implement/check 角色并加载已填写的 jsonl，不因角色改变权限。

## 起步步骤

### 2026-10-18 继续执行的依赖判定

用户在 01 工程基础已实施并检查后要求继续后续任务。将可消费的代码/接口依赖与未完成的平台、性能验收分开跟踪：01 的 Windows AC1.1–1.4 已验证，足以支撑 02 的共享服务实现；macOS、完整 Tauri fixture 基线和原生矩阵继续作为开放验收项，01 不标完成。后续按代码依赖顺序推进，不能借此豁免任何子任务或父任务的最终验收。仅在实际依赖代码尚未可用或发现兼容/安全问题时阻断相应实施，不让无关的外部验收证据阻断全部后续代码。

1. 对第 01 任务重新核查版本与平台系统要求，先生成可重复的最小 native probe。
2. 记录 Tauri release 基线，禁止把两端 debug/release 混比；基线数据根独立。
3. GPUI 包可编译、能显示文本并正确关闭后，再做主题和业务。先验证中文 IME 和图形兼容，早暴露平台风险。
4. 新增各 desktop 命令时同步写 command graph 检查，执行 just --list 和 dry-run，并实际运行基础命令。
5. 其余每个子任务按“共享逻辑 -> adapter -> native UI -> 回归 -> 台账证据”推进。

## 验证层级

- 规划现在：task.json 父子关系、三个计划文档、manifest 引用存在、需求映射、无待填写占位符、设计对比度计算、git diff --check。
- 01 起：desktop-doctor/fmt-check/check/lint/test/build，独立 target/lock 验证；版本与命令治理小测试。
- 02 起：根 just version-check / fmt-check；相关 crates 单测；共享接口影响 Tauri 时执行对应 Tauri tests/typed bindings gate。
- 数据写入/认证/任务：失败与冲突矩阵必须覆盖，不只成功路径；secret-write-check 和原有 security contracts 继续有效。
- 10：desktop-ci/audit、性能原始数据、三平台 native 截图与手工交互记录；just ci 最终整合，不自动 fmt/version-sync 修复。
- WebView 旧表面视觉若有变化才使用 ccr-ui visual workflow；GPUI 窗口不用浏览器截图冒充验证。

## 易出错文件与回退

- 根 Cargo.toml/Cargo.lock：仅增加中立共享 crate 及必要依赖，不把 GPUI 拉进根默认图。
- ccr-ui/src-tauri/Cargo.toml/Cargo.lock 与服务/命令：保持 wire API 与 existing tests，迁一个域验证一个域。
- scripts/version/version-sync.{ps1,sh}：新增 GPUI manifest 时两端及其测试同时更新。
- scripts/drift/check_dependency_drift.py 与 scripts/ci/ci_surface_policy.py：在 10 阶段覆盖新 surface；不改现有 required 名称和阈值。
- 新锁文件与平台配置：出现不兼容时回到候选验证，不临时浮动 git 或混用核心。
- 数据写入测试只碰 fixture。应用回退仅改变启动入口，不回滚或删除真实数据库。

## 交付证据

每个子任务写 evidence.md，含实际命令、退出码、平台/工具版本、截图或日志路径、未运行项。父任务的 command/route 台账每项关联 owner、native 行为、测试与完成状态。最终不允许以“页面存在”代替业务闭环，以“编译通过”代替 native 运行，以“使用 GPUI”代替性能达标。
