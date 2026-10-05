# T08 实施顺序与验证

## 启动条件

- [x] 用户于 2026-09-28 批准父任务及全部子任务的最新规划；执行前仍须读取 applicable AGENTS 与 specs。
- [ ] 核对前置任务：无子任务前置；仍须用户批准规划后才可实施。
- [ ] 记录当前 commit、工作区和既有失败；不回退或删除他人修改。
- [ ] 依 Trellis 流程在前置契约满足后单独激活本子任务；当前状态见 task.json。

## 有序实施

- [ ] 1. 固化 R1/AC1 的旧失败反例：以真实 settings mapper 和 mock transport 捕获请求，后端 fixture 应用 patch 后比对非目标路径；加入 unknown enum/字段 round-trip。
- [ ] 1.1 在明确 owner 内实现机制：SettingsConfig 保留原始 typed snapshot，按 dirtyKeys 构造 patch；不把未表示的 union 压成 boolean。提供明确 unset 操作，区别未修改、清空和删除；无损 mapper 单独测试。
- [ ] 1.2 运行行为断言并验证 AC1，保留兼容成功路径。
- [ ] 2. 固化 R2/AC2 的旧失败反例：渲染完整 BaseSettings 并触发用户交互，断言 DOM disabled、说明文案、请求 payload；后端拒绝仍有错误反馈。
- [ ] 2.1 在明确 owner 内实现机制：将 load 结果扩展为 values、managedLocks、layer/token metadata；BaseSettings 消费平台 descriptor 提供的能力，FieldControl 支持 disabled/reason 和 current unknown option，禁止平台名分支。
- [ ] 2.2 运行行为断言并验证 AC2，保留兼容成功路径。
- [ ] 3. 固化 R3/AC3 的旧失败反例：路由级 smoke 覆盖入口可见、保存/冲突、local/WSL/SSH 禁用；后续 web 交互与原生 smoke 分别记录，不以截图替代行为测试。 Grok fixture 对 typed/raw 保存和 invalid/stale 等失败路径比较目标与备份目录清单，断言无新增备份，并断言 no-backup notice 可见。
- [ ] 3.1 在明确 owner 内实现机制：descriptor 提供 raw callbacks、content token、layer notices 和 environment policy，复用 ConfigSourcePanel；不新增各平台专用重复页面，不更改市场终端视觉方向。 ConfigSourcePanel 的领域中性 composite 移入现有明确共享层后供 Base 复用，避免 features/platform 直接跨域依赖 features/editor，不扩大全域 import 豁免。Grok 保留无备份、policy layer 和 Profiles/off 恢复说明。原始文本仅停留编辑会话，不入全局 store/localStorage/log。
- [ ] 3.2 运行行为断言并验证 AC3，保留兼容成功路径。

## 验证命令

下列命令在实施后运行；当前规划未预先执行。测试过滤器必须匹配实际用例，执行零个用例不能判为通过。涉及生成物的命令只在获批实现和隔离工作区使用。

```text
cd ccr-ui && bun run test:smoke -- tests/platforms tests/configs
cd ccr-ui && bun run type-check
cd ccr-ui && bun run lint:ci
cd ccr-ui && bun run build
git diff --check
```

## 交付与集成

- [ ] 同步本任务拥有的规范和命令/DTO 生成物，JSONL 仅引用 spec/research。
- [ ] 依 Trellis 实现/检查角色完成独立检查，修复本次引入的问题；无关基线失败保留原始证据。
- [ ] 更新父任务 requirement-to-evidence ledger：测试、运行环境、commit、剩余风险。
- [ ] 对应子任务通过后交 T10 做跨域集成；T10 自身直接回到父任务集成审查。
- [ ] UI 改动做相关 Web 行为/视觉验证并另列 native 限制；OS/权限/进程改动做原生平台验证。

## 失败和回滚

mapper、descriptor 与组件同批回退代码，保持原 wire 格式；不以旧表单数据覆盖用户新配置。禁止为 Grok 等无备份域自动保存前镜像或复制敏感原文到发布、研究或历史产物；其他配置仅遵守所属领域已经批准的备份策略，不新增备份动作。

正式 gate 失败不得以排除文件的诊断结果替代。未通过的验收保持未勾选；不得据此完成或归档。
