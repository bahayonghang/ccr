# AG02：公共控件、配置与资源页面：技术设计

## 继承合同
遵守父任务../10-26-ccr-ui-airy-glass/design.md，研究见父research/surface-inventory.md和validation-contracts.md。原始目录仅参考，CCR适配以批准后的父设计为准。

## 所有权与边界
路由所有权补充：`/slash-commands`、各平台slash-commands、`/opencode/commands`归本任务资源管理；动态`/commands/:client?`、`/codex/sessions`及`/agent-sessions`归AG03。按页面职责选文件，不能按commands/codex整个目录接管。

src/ui/、必要的src/components/profiles/共享表面；src/features/configs/、claude/、codex/、grok/、gemini/、opencode/、platform/、mcp/、sync/中的操作页面与领域CSS。按surface-inventory逐页选文件，不整目录重写；不改API/hooks/stores业务。

只修改与验收直接相关的展示/样式/守卫；读取相关spec后再实施。所有代码引用是候选范围，不是必须逐个修改的清单。公共热点串行变更；其他代理可能同时工作，不覆盖其改动。

## 前置和实现顺序
AG01令牌、外壳、四组合代表页和基线可用。

1. 将surface-inventory的操作页面分配到路由登记表，确认动态参数fixture。
2. 先迁移primitives/shared editor/profiles，覆盖控件状态与focus；不改提交请求。
3. 按配置→平台设置/认证→资源→同步/环境顺序迁移，复用shared Base。
4. 逐页核对草稿/错误/禁用/危险确认及Web不可用状态；保留Skillport迁移引导。
5. 运行目标域烟测和前端检查，补四组合共享控件及代表页面截图。

## 风险及回滚
展示重构引入mount/key或表单序列化变化会丢草稿/越过确认；本任务禁止这些变化。

记录本阶段文件差异与基线，只逆转自己改动；保留用户与其他任务工作。规范、tokens、消费者、测试一起回退。出现行为偏差优先恢复原行为，不通过删除保护断言解决。
