# AG03：观测、图表与运行状态页面

## 目标
让数据密集与异步运行页面融入新风格，仍能快速、准确理解数量、未知状态与下一步操作。

## 前置与范围
AG01基础通过，AG02公共组件稳定；共享组件变更先协调所有者。

首页和Insights、各平台概览、Usage/预算/价格、监控、Agent Sessions/Transcript、命令、签到与独立tray。

## 验收标准
- [ ] AC1：观测路由与下半屏Insights完整覆盖，未知/空/零/加载/失败不混淆。
- [ ] AC2：有界图表、平台/来源色、精确表格/图例/刻度保留，新底色上文字与标识可读。
- [ ] AC3：Usage visitedTabs/hidden保持挂载，options/series稳定性、数据比例和数值格式不变。
- [ ] AC4：会话列表/Transcript虚拟化估高/overscan/末项可达、日志滚动和筛选不退化。
- [ ] AC5：命令/签到的忙碌、取消、恢复、错误、余额四态可辨识；tray使用紧凑实底，保持原交互。

## 不在范围
不改后端数据、IPC、权限和业务机制，不处理并行GPUI改动，不新增主题体系，不安装/提交/推送/发行。

## 需求追溯与状态
父任务：../10-26-ccr-ui-airy-glass/prd.md；对应R2 R3 R5 R6 R7。父design.md和verification.md的兼容/验收合同适用于本任务。已批准实施；展示实现和检查记录已补齐，当前保持in_progress，最终验收缺口见AG04 integration-summary。
