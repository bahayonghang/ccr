# TUI Profile 信息层级与启动性能优化

## Goal

让 `ccr` 主 TUI 的 Profile 页面更容易快速判断当前路由、模型、推理强度与认证状态，同时消除启动时可感知的固定等待，使首屏出现更快且过程不再表现为空白卡顿。

## User Value

- 用户不需要逐行阅读同权重文本，即可确认当前 profile 的关键运行参数。
- Codex profile 中实际生效的 `model_reasoning_effort` 不再从详情页缺失。
- 宽屏空间优先给需要阅读的详情内容，空状态栏和重复字段不再挤占首屏。
- 输入 `ccr` 后不再为终端主题探测同步等待约 100ms。

## Confirmed Facts

- Codex profile 的 `model_reasoning_effort` 存在于 `ProfileConfig::platform_data`，`ccr-codex` 会校验并写入 Codex `config.toml`，但 `ccr-tui` 当前没有读取或渲染该字段。
- `Focus` 显示 name/current/enabled，`Context / Overview` 又显示相同内容，存在重复。
- 宽屏详情底部固定保留 3 行 `Status`；没有 apply/toast 反馈时仍渲染空条。
- 宽屏当前采用列表 54% / 详情 46%，长 URL、说明和 Usage note 的阅读空间反而更窄。
- 详情值样式由 `detail_value_style(label, value)` 对字符串做启发式匹配，不能稳定表达字段重要性或推理强度等级。
- 本机 release 探针的 `App::with_task_executor` 热启动为 9.0-12.2ms；交互终端中的 `theme::init_theme()` 为 124.8ms，和 `termbg::theme(100ms)` 的超时路径吻合。
- `run_tui()` 在 `App` 构造前进入 alternate screen，且语言配置会在 `initialize_from_config()` 和 `App::with_task_executor()` 中读取两次。
- TUI 日志初始化会同步扫描 `~/.ccr/logs`；当前滚动文件名形如 `ccr.log.YYYY-MM-DD`，现有扩展名判断不能命中过期文件，长期会积累目录项，但本机热路径仍属于次要开销。

## Requirements

### Profile 详情

- Codex `Engine` 分组必须显示 `model_reasoning_effort`，中文标签为“推理强度”；未配置时显示 `-`，不得伪造默认值。
- model、推理强度、auth mode/provider、base URL、current/enabled、token 状态和估算费用必须使用显式字段语义决定样式，不再依赖 label/value 子串猜测。
- 推理强度的 `minimal/low/medium/high/xhigh` 应有可区分但不暗示错误的强度层级；未知原始值必须可见并以 warning 表达配置异常。
- 继续遵守 secret masking，不得在详情、测试快照或日志中暴露 token 原文。
- 新增标签必须完整支持中英文并保持 CJK display-width 安全。

### 排版

- 宽屏详情列不得窄于列表列；目标比例采用列表 46% / 详情 54%，并保留窄屏/标准/宽屏三种既有响应模式。
- `Focus` 作为唯一的 profile 身份/状态摘要；详情分组不得重复 name/current/enabled。
- 无 apply/toast 反馈时不分配 `Status` 高度；有反馈时仍能稳定显示且不覆盖详情。
- 详情标签列宽按当前语言和可见字段计算并设置合理上下限，避免固定 16 列造成无效空白或中文拥挤。
- 滚动条、选中 profile、分页和每 tab 独立 selection 行为保持不变。

### 启动性能

- 默认启动路径不得在首帧前执行 100ms 级终端背景查询。
- 配置、profile 和 runtime summary 的失败仍以可恢复状态进入 TUI，不得因性能优化改为 panic 或静默丢失。
- TUI 配置只加载一次，并同时供语言、主题和 tab order 使用。
- 终端切屏后应尽快完成首帧绘制；可能变慢的 profile/runtime/usage 工作必须有明确同步预算或后台状态。
- 增加可重复的启动分段测量，至少区分 logger/CLI、theme、App construction 和 first draw；不得把一次冷编译计入产品启动基线。

## Acceptance Criteria

- [x] Codex profile 详情显示与 profile 原始配置一致的 `model_reasoning_effort`，覆盖已配置、未配置、大小写和未知值测试。
- [x] 关键字段具有稳定的显式样式测试，普通描述/计数与关键运行参数的视觉层级可区分。
- [x] 宽屏不再重复 name/current/enabled，空 `Status` 不占高度，详情宽度大于列表宽度。
- [x] 80x20、100x30、140x30 的中英文 TestBackend 渲染均无截断回归、重叠或 panic。
- [x] 默认启动不调用同步 `termbg` 探测；交互终端基线中 theme 阶段不再出现约 100ms 固定等待。
- [x] `App` 构造和首帧路径有分段计时证据；本机热启动目标为首帧前 CCR 自有同步工作 p50 < 50ms、p95 < 100ms（不含 OS 进程创建）。
- [x] `cargo test -p ccr-tui -- --test-threads=1`、`cargo test -p ccr-config -- --test-threads=1`、`cargo test -p ccr -- --test-threads=1`、`just fmt-check`、`just lint-strict` 和任务范围 `git diff --check` 通过；全局检查仅命中任务外 `TODO.md` 的既有空白。

## Out of Scope

- 重设计 Claude/Codex/OpenCode Auth 页面。
- 修改 profile 的 apply/认证语义或 `model_reasoning_effort` 合法枚举。
- 把 Usage 查询移回同步渲染路径。
- 为本任务引入数据库或长期遥测。

## Confirmed Product Decision

- 采用确定性默认主题策略：Mocha 为默认值，`Ctrl+T` 切换后持久化到 `tui.toml`，`CCR_TUI_THEME=mocha|latte` 继续作为显式覆盖。只有明确设置 `CCR_TUI_THEME=auto` 时才执行终端背景探测并接受相应等待；默认启动路径不得调用 `termbg`。
