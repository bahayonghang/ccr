# GPUI 03：轻盈玻璃设计系统与原生外壳：执行计划

## Preconditions

01 工程接口可消费，开放的平台/性能验收依父 implement.md 独立跟踪；02 AC2.1–2.5 已独立通过。用户已批准父子方案并明确完成 02 后继续 03，无需重复审批。只有实质改变产品范围/兼容行为/安全边界的变化才另行评审。

2026-10-18 最新验证指令：聚焦核心功能，不执行缩放/窗口尺寸/DPI 矩阵。相关未运行项记 user-deferred，不能标 pass，也不作为阻止已批准后续工程的理由。仍实现原有自适应设计边界；本阶段验证真实导航/状态、读取筛选/错误恢复、键盘/输入与取消退出，避免无关检查。

## 视觉增量顺序（待审阅和恢复）

2026-10-27：用户明确要求“请继续实施gpui 这个trellis任务”，本次恢复包含当前已落盘的轻盈玻璃增量。已正常激活03，沿用原任务树；先保存旧release产物及环境/source/fixture哈希，再按能力探针结果实施。锁定渲染栈未提供应用内backdrop blur接口，本轮使用合同允许的实色降级，真实glass不记通过。原DPI/窗口尺寸延期继续保留。

按父 [airy-glass-plan](../10-17-gpui-desktop/research/airy-glass-plan.md) 先固定旧 GPUI release 基线，再核查锁定依赖/内景模糊和系统偏好能力。通过探针后更新 DESIGN.md → theme.rs → shell，随后复核已经接入的配置/转换/收藏/历史/设置及确认弹层。保存 glass 与 solid 的原生截图、复合对比度和 S1/S2 数据；新增材质能力选择的行为测试，不能只测 token 字面值。失败先降 solid，图形依赖升级需重新评审。历史业务批准不自动授权本次视觉实施；原暂停和延期矩阵保持。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 按视觉增量顺序更新 ccr-desktop/DESIGN.md，随后实现 light/dark、系统字体、数字对齐和单一语义 token 映射。
3. 稳定环境入口、功能分组、主列表/详情、状态区；导航动作与键盘焦点有真实 state。 原生输入、中文 IME、Tab/Shift+Tab/Enter/Escape、返回、错误/空/加载/禁用状态，中文和英文文案。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- just desktop-test-ui；just desktop-test；just desktop-lint
- 原生窗口 1280×800、960×800、1280×640、960×640 矩阵：user-deferred，本轮不执行
- 100/150/200% DPI：user-deferred；light/dark、zh-CN/en、键盘/IME/屏幕阅读器仍分别记录，不拿 headless 代替 native
- 对比度计算及实际焦点/hover/pressed/disabled/error 渲染审查

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
