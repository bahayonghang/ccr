# AG03：观测、图表与运行状态页面：技术设计

## 继承合同
遵守父任务../10-26-ccr-ui-airy-glass/design.md，研究见父research/surface-inventory.md和validation-contracts.md。原始目录仅参考，CCR适配以批准后的父设计为准。

## 所有权与边界
路由所有权补充：本任务拥有`/commands/:client?`、`/codex/sessions`和`/agent-sessions`；`/slash-commands`、各平台slash-commands及`/opencode/commands`归AG02资源管理，不因共用commands目录混入本任务。

src/features/usage/、monitoring/、agent-sessions/、commands/、checkin/、tray/，各平台Dashboard与相关CSS，src/styles/components/home.css；精准选择展示文件，不改事件协议/数据聚合/SQL。

只修改与验收直接相关的展示/样式/守卫；读取相关spec后再实施。所有代码引用是候选范围，不是必须逐个修改的清单。公共热点串行变更；其他代理可能同时工作，不覆盖其改动。

## 前置和实现顺序
AG01基础通过，AG02公共组件稳定；共享组件变更先协调所有者。

1. 建立数据与异步状态fixture，检查rendered page而非redirect/placeholder。
2. 迁移首页/Insights及平台概览，保留布局密度、有界图表和底部状态栏。
3. 迁移Usage/预算/价格和会话/日志，先固定底板，再校对数据色和legend/table。
4. 迁移commands/checkin/tray展示；不改订阅、取消和恢复逻辑。
5. 运行图表/会话/作业烟测，采集滚动/图表切换对照证据并交AG04。

## 风险及回滚
强制绿紫图表会损坏平台区分；加padding会破坏虚拟高度；条件卸载会导致chart重建。

记录本阶段文件差异与基线，只逆转自己改动；保留用户与其他任务工作。规范、tokens、消费者、测试一起回退。出现行为偏差优先恢复原行为，不通过删除保护断言解决。
