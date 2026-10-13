# TUI Profile 工作台重设计与性能优化

## Goal

让 Codex、Claude、Grok 的 Profile 工作台更容易识别当前运行状态、比较配置和了解活动，用可验证的性能优化消除交互阻塞。

## Background

用户提供终端截图，要求改进 Current runtime 配色和文字排版；Profile 列表使用有表头、辅助线、对齐列的表格并增加上次使用时间；右侧提高信息密度并利用已有数据库信息；分析并优化查询、渲染、切换性能；创建三平台 Trellis 优化任务；Claude Code 页面标题改为 Claude Profile；优先官方图标，去除通用表情。

目标为 Rust Ratatui `crates/ccr-tui`。改动前三平台共享布局，Current runtime 仅 Codex 有顶部区域，列表为手工拼接 List，右侧为单列字段。初始诊断见 [视觉证据](research/visual-audit.md)、[数据合同](research/profile-data-contracts.md) 和 [性能研究](research/profile-performance.md)。

## Requirements

- R1：运行状态区用有语义的颜色和明确的文字层级区分当前生效配置、运行模式和认证状态；列表焦点不冒充当前运行配置。
- R2：三平台共享带表头的表格，含状态、Profile、上次使用时间；空间足够时增加描述和模型列。列按终端显示宽度对齐，保留鼠标、分页、选择记忆。
- R3：上次使用以可验证的最近成功应用/切换记录为建议口径，详情标注含义。无记录、坏时间戳、读取失败分别表达，不能推断实际最后一次 API 调用。
- R4：右侧按“身份与连接 → provider 用量 → 启用记录”的顺序展示；用同刻度条形和对齐数值比较已有 token 分类，合并重复时间信息，正常状态隐藏空值摘要和无关说明。详见 [第二版设计](research/right-panel-v2.md)。
- R5：英文页面身份统一为 `Codex Profile`、`Claude Profile`、`Grok Profile`；品牌位置去除通用 emoji。可移植文本为基线，官方图标只在来源及终端支持验证后使用，不以近似字符冒充官方图形。
- R6：消除主事件线程的 Profile 加载、切换和重载阻塞；保留锁、备份、原子写、恢复、成功伴随告警等原有语义。用量缓存继续复用。
- R7：渲染和选择为纯内存工作；按实际测量减少重复布局、文本生成或空闲重绘。性能报告区分代码发现、实测基线和实测收益。
- R8：覆盖中英文、Mocha/Latte、窄屏/短屏/宽屏、空态/错误/忙碌态；Auth 内容和既有快捷键保持兼容。

## Acceptance Criteria

- A1 → R1：顶部状态与当前配置一致，选中其他 Profile 后不变化；中性、可用、待认证、错误有文字和颜色双重提示。
- A2 → R2：所有可见表头与数据列边界一致，新增表头后分页容量正确，点击表头/辅助线不选中记录，首末数据行命中正确。
- A3 → R3：同名跨平台、无历史、重复激活、坏时间戳、历史写入告警测试通过；表格时间有明确来源，未知不变成刚刚。
- A4 → R4：右侧第二版 A 方案的字段就近对齐，token 分类共用刻度和数值列；窄屏转为数值列表，长文本换行，短屏滚动可达所有有效字段；最新启用时间仅出现一次；费用明确为 provider 级全时段官方等价估算，不用条形推导占比、配额或命中率。
- A5 → R5：所有英文 Claude Profile 页面身份均为 `Claude Profile`，Auth 身份不变；三平台共享导航没有机器人/电脑/火箭类品牌表情。
- A6 → R6：延迟加载/切换的 fixture 下键盘事件和重绘继续响应，重复 Apply 不重复写，异步结果回到原平台，退出不会截断事务。
- A7 → R7：渲染和移动选择的 I/O 计数为零；刷新只加载所需数据，基线和优化后记录相同 fixture/构建/机器环境，不宣称未经测量的加速比。
- A8 → R8：代表尺寸、两种语言/主题、三平台状态矩阵和安全契约测试通过，实施后提供渲染证据。

## Out of scope

不扩展 Web/Tauri/VS Code，不增加全新用量采集平台，不对个人 Profile 或认证文件进行试切换，不削减持久化安全步骤，不要求特殊字体或终端协议，不发布或自动提交代码。

## Task map

父任务负责共享表格/详情布局、只读历史投影、异步加载/切换、渲染策略及联合验收，有直接实施工作。子任务负责平台特有行为，依次整合以避免多人并行修改同一 `ui.rs`。

| Child | Responsibility |
|---|---|
| [Codex](../10-12-codex-profile-workbench/prd.md) | runtime、Engine/Routing/Auth、provider 用量 |
| [Claude](../10-12-claude-profile-workbench/prd.md) | Claude Profile 标题、模型映射、运行态和认证、provider 用量 |
| [Grok](../10-12-grok-profile-workbench/prd.md) | backend、认证来源、session 状态、活动、已有 Grok provider 投影接入 |

## Artifact status and review

用户已明确回复“采用A作为三平台的实施版本，开始实施”。共享布局、只读活动投影、异步操作以及 Codex/Claude/Grok 的 A 版呈现均已实施；独立复核完成，发现的问题已修复。实际 Ratatui TestBackend 导出 34 份单元格及对应彩色图，覆盖两种语言、主题、宽窄屏和边界状态；初始 A/B/C 设计图保留为设计过程材料。

专项回归、严格 lint、工作区测试和全部 CI 分项已通过。完整命令先在既有 Tailwind 样式规则处失败；最小兼容修复经独立复核后，前端至扩展的剩余分项续跑通过，原始失败记录保留。fixture 基准记录主线程响应、历史查询与渲染成本；没有把更丰富布局的渲染成本增加宣称为加速。原生终端字体、冷盘查询和真实事务前后耗时尚未验证。四个任务保留 in_progress，未提交或归档。实施证据及验收映射见 [整合验证](research/integration-validation.md) 和 [实施状态](research/implementation-status.md)。
