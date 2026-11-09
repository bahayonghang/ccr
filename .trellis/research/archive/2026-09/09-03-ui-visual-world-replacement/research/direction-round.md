# 视觉方向轮决策记录（2025-09-03）

> impeccable new-work「替换视觉世界」流程。seed key `19fe1fa0`，mode=operate，code-led（本机无图像生成工具，`.impeccable/config.json` 记 `buildPath: code`）。

## 产品事实与文化母土

- 机制一句话：CCR UI 把多个 AI CLI 工具链的配置、认证、用量与运行状态收进一个可信桌面控制台。
- 受众实景：中文为主的高级开发者，暗光下长时间盯守多平台请求/令牌/成本/事件。
- 类目套路（rut，排除出候选）：AI SaaS 深色仪表板 + 霓虹辉光；其可预测反面 = 暖奶油编辑风 + 陶土橙（恰是现任 clay 世界）。

## 我的 7 个 grounded 候选（按共鸣排序）

1. Bloomberg/路透行情终端 —— 产品本质就是专业数据终端
2. tmux/终端复用器网格 —— 受众日常母语
3. 航空运行手册/检查单 —— 风险下的冷静权威，对应安全语义
4. 机场/铁路翻牌信息板（Solari）—— 状态随时间翻转
5. 工程制图/技术标注 —— 配置即图纸的精确性
6. 广播调音台 —— 平台即通道、用量即 VU 表
7. 铁路列车运行图 —— 时间×状态的信息图形极致

骰子指派：**候选 7（运转图台）**。

## 挑战者裁决（两轴：受众认同 / 产品清晰）

| 挑战者 | 裁决 | 理由 |
|---|---|---|
| 示波器信号台 | competitive | 监控/事件流贴合；配置/认证/MCP 无母语 |
| 一位桌面（1984） | competitive | 认同感强；抖动纹理毁高密度可读性 |
| Factory 编号档案 | competitive | 设计素养向认同成立；以码代名违背状态真实 |
| 虹彩云边 | declined | 双轴皆输。**留存纪律：颜色只住状态发线** |
| 工业引号语法 | declined | 双轴皆输。**留存纪律：禁用/阻断态一眼可辨** |
| 拉班舞谱 | declined | 双轴皆输。**留存纪律：图表维度诚实（长度即数量）** |

## 最终选择

**用户点选「行情终端」（IMPECCABLE'S PICK 卡），覆盖骰子指派的运转图台。** 规则：用户钉死的决定永远优先于骰子。kind=pick。

方向契约已写入 surface brief：`ccr-ui/.impeccable/surfaces/ui-src-features-usage-dashboard-dashboardview-tsx.md`（primary target = DashboardView，related = AppSettingsView / MainLayoutChrome）。

核心承诺：
- 暖黑磷光终端：`#100f0c` 底 / `#1e1c16` 面板 / `#e6e1d5` 数据白；琥珀 `#f0a32b` 仅命令/激活/焦点；绿 `#5fa05a` 红 `#c0503c` 仅状态。
- 等宽表格数字、发丝分隔线、功能键命令条；平台线色仅作身份 tick。
- 首页首屏：顶部行情带、左功能键命令列、主区固定行高用量面板（图表高度有界）、右栏事件日志与行动队列、底部命令状态条。
- 三条落选者纪律作为全场约束：颜色只住状态发线；禁用态一眼可辨；图表维度诚实。

构建状态：`build-phase.mjs start --direction 19fe1fa0 --kind pick` 已记录（choice recorded，code-led 无 phase gates）。
