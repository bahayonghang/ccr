# AG01：轻盈玻璃规范、令牌与应用外壳

## 目标
建立唯一轻盈玻璃材质与主题基础，让首页、Profiles、Settings、Usage的代表页在亮暗四组合中稳定呈现。

## 前置与范围
父任务最终方案获用户批准。必须先收集before基线再改造。

设计规范、令牌治理、主外壳、首屏/原生视觉同步、浮层实底；不迁移领域业务。

## 验收标准
- [ ] AC1：改造前路由/数据/环境/代码指纹、截图和性能数据可复现；工具异常不被记成零耗时。
- [ ] AC2：新设计来源和迁移说明一致，令牌新增差集被登记，没有第二主题系统。
- [ ] AC3：同屏唯一纸面blur；普通workspace/modal/status实底；各降级条件blur=none、alpha=1。
- [ ] AC4：light/dark×neutral/clay、启动恢复、system/字体/custom accent结构和存储兼容全部保留。
- [ ] AC5：代表页portal、sticky、sidebar resize、滚动恢复可用；原生窗口不新增透明/权限。

## 不在范围
不改后端数据、IPC、权限和业务机制，不处理并行GPUI改动，不新增主题体系，不安装/提交/推送/发行。

## 需求追溯与状态
父任务：../10-26-ccr-ui-airy-glass/prd.md；对应R1 R2 R4 R6 R7 R8。父design.md和verification.md的兼容/验收合同适用于本任务。已批准实施；展示实现和检查记录已补齐，当前保持in_progress，最终验收缺口见AG04 integration-summary。
