# 执行与验证

## 进入实现前

- [x] 用户在最终规划摘要之后批准实现；保持 status=planning 直到批准。
- [x] 完成独立规划检查、修正阻断项；运行结构和引用预检。
- [x] `implement.jsonl` / `check.jsonl` 含真实 spec/research，无 `_example`。
- [x] 运行 `task.py start`，加载 Phase 2.1 上下文；派发 `trellis-implement`，主线程仅协调与更新规划/spec。

2026-09-08：下列实施步骤已完成，AC1–AC5 及全部必需门禁通过；具体证据见 `checks.md` 与 `checks-review.md`。保留未提交状态，不执行自动提交归档。

## 实施顺序

1. 记录 `git status` 并保留已有修改。读取 TUI spec 与本任务研究。
2. 在 `codex_auth/app.rs` 与 `ui.rs` 的现有测试模块加入可失败用例：归属成功说明误着色、仅 preview cache + Idle 的详情缺失、明确缺失窗口被画满、窄窗口统计被长说明挤掉。测试全部注入临时目录及合成记录，不调用真实 auth/usage/配额接口。先记录至少一次失败。
3. 完成 D1/D2 状态与展示修复，不改账本过滤或配额 service。直接复用现有 attribution_state 分类样式。
4. 完成 D3/D4 布局、条形、数字与中英文文案；两个真实入口都经主 App 的 draw_embedded，基于此路径复用内容。仅在主 tui/ui.rs 测试模块补全实际组合渲染，不重构旧独立 draw。修改所有受影响断言，不删除范围语义检查。
5. 派发 `trellis-check` 独立检查，限定本任务文件；修复本任务引入的问题，其他失败单独报告。
6. 主线程把稳定契约补入 TUI spec：剩余配额、缺失窗口、账号说明/全局回退、紧凑布局与测试矩阵。记录 checks.md 并更新 AC 勾选；不在缺少证据时勾选。

## 回归矩阵

| 验证对象 | Fixture 与判据 |
| --- | --- |
| AC1 配额 | 0/10/50/80/100%；Some(true)/Some(false)/None；无缓存等待/加载/错误；仅 preview 缓存且 Idle；带缓存刷新/失败；条形和百分比同源、缓存时间可辨、列表详情一致 |
| AC2 范围 | A/B 交错账本及额外历史：A 的数值不含 B；成功说明非 warning 且不推断历史损坏；无账号 ID/空账本无命中/虚拟账号依现有全局回退且保留警告；原有时间窗边界测试继续通过 |
| AC3 布局 | 英文/中文 × 80×24/100×22/100×30/120×22/140×40/180×50，测试 draw_embedded 有效内容区及主 App 实际组合渲染，核对独立命令同样路由到主 App；数字、范围、两窗口、单行错误落在内边框；100×22 验证列表3+详情9、120×22验证右侧合并卡；另测 60×18 只要求无 panic、无无范围数字，并明确省略提示 |
| AC4 信息 | 0、999、1K、999950 的单位晋级、百万/十亿/万亿；CJK 账号、长模型、长重置、长错误带省略号；请求和 token 按列对齐，保留累计及主要模型优先级 |
| AC5 交互 | 已有选择/刷新/分页/overlay 测试通过；两语言切换不改变账号选择；主题测试不修改全局 ACTIVE，使用既有纯 palette helper 或读取稳定主题 |

## 命令

规划阶段仅运行前两类，不启动产品测试或修改产品代码。

```powershell
python .trellis/scripts/task.py validate .trellis/tasks/09-08-codex-auth-tui-presentation
python C:/Users/lyh/.agents/skills/trellis-plan-review/scripts/plan_precheck.py .trellis/tasks/09-08-codex-auth-tui-presentation --include-descendants
rtk cargo test -p ccr-tui codex_auth -- --test-threads=1
rtk cargo test -p ccr-tui -- --test-threads=1
rtk cargo test -p ccr -- --test-threads=1
rtk cargo test -p ccr-usage
just version-check
just fmt-check
just lint-strict
```

`just` 作为未声明 RTK 支持的命令直接运行。一次一条检查退出码，失败不由后续命令遮盖。若范围扩大到跨 crate 生产代码或宣称发布就绪，追加 `just ci`；本任务不修改其他 crate，ccr 测试作为 TUI feature surface 的必需集成检查。

## 回退与收尾

- 风险文件只有 app/ui 及测试；若归属数字变化，回退该变化而非接受视觉修复掩盖统计回归。
- 产品测试结果与真实终端视觉验证分别记录。无个人运行时采样，不对截图真实统计完整性背书。
- 当前授权含规划、拟议实现与检查；实现仍需最终规划确认。提交、推送、发布、archive 自动提交均未授权，不执行。
