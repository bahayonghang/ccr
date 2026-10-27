# AG01：轻盈玻璃规范、令牌与应用外壳：技术设计

## 继承合同
遵守父任务../10-26-ccr-ui-airy-glass/design.md，研究见父research/surface-inventory.md和validation-contracts.md。原始目录仅参考，CCR适配以批准后的父设计为准。

## 所有权与边界
ccr-ui/DESIGN.md、ccr-ui/AGENTS.md、ccr-ui/.impeccable/design.json、.trellis/spec/ccr-ui/frontend/theme-token-contracts.md；src/styles/tokens.css、core.css、components/surfaces.css、utilities/utilities.css、shell-critical.css；src/shell/MainLayout.tsx、shell.css；index.html、src/utils/nativeWindowAppearance.ts、src/features/configs/lib/flavorPreview.ts；对应theme tests。共享技能仅改被跟踪源文件。

只修改与验收直接相关的展示/样式/守卫；读取相关spec后再实施。所有代码引用是候选范围，不是必须逐个修改的清单。公共热点串行变更；其他代理可能同时工作，不覆盖其改动。

## 前置和实现顺序
父任务最终方案获用户批准。必须先收集before基线再改造。

性能harness首次校准归本任务：先检查React选择器、真实目标页面/滚动距离、错误分支，再采成功before；必要的现有scripts/perf最小修复属于本任务。不能等AG04才建立方法。

1. 审计工作树与验证命令隐含安装，记录before和路由登记。
2. 完成新DESIGN适配与暗色/状态/图表底板候选配色，先算对比度；同步AGENTS/sidecar及spec。
3. 记录token名字/消费者基线；登记唯一纸面别名，保留共享实底角色。
4. 迁移外壳/首帧/flavor预览/macOS底色；浮层取消二次blur。
5. 定向迁移旧外观守卫，补合成/运行时层数/降级；跑主题与shell烟测及四组合代表页。

## 风险及回滚
全局token连带影响所有页面；字段/图表底板不能透明化。性能before缺失时不得向后宣称无回归。

记录本阶段文件差异与基线，只逆转自己改动；保留用户与其他任务工作。规范、tokens、消费者、测试一起回退。出现行为偏差优先恢复原行为，不通过删除保护断言解决。
