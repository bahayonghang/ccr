# OpenCode Auth 遗留清理

## 目标与范围

父任务 09-08-tui-grok-auth-replacement 的清理交付。当前主 TUI 已有 Grok Auth，不重复删除已不存在的旧子应用。依据父任务 research/codebase.md 清理剩余专属代码和测试。本会话已按用户明确要求开始实施。

## 需求

- R1：删除 OpencodeAuth 枚举、as_str、load 过滤和仅验证旧兼容的用例。
- R2：删除无调用方的 OpenCodePaths、专属配色/helper/断言；保留 CodexPaths、有效 OpenCode 消费者。
- R3：混合测试只替换旧 fixture，保留语言/主题/Usage/有效排序/通用错误处理；更新当前规范和 README 过时宣传。

## 验收

- [ ] AC1（R1）：有效源码不再定义旧 tab 标识，默认六页与合法自定义排序通过；旧 opencode_auth 走既有未知枚举错误和整体默认回退，初次加载不写盘。
- [ ] AC2（R2）：OpenCodePaths 与专属 theme helper 删除，无悬空引用；其他路径/主题测试通过，有效 OpenCode 配置和 usage 消费者不变。
- [ ] AC3（R3）：语言、主题、Usage fixture 使用有效 tab 后原行为仍通过；当前规范说明旧配置影响，历史记录不重写。

## 非目标、依赖与状态

不实现 Grok 账号服务/页面，不删除 Usage 兼容，不批量移除 OpenCode 关键词。无前置依赖；其 theme 结果是 TUI 子任务集成输入。已进入实施。本轮按用户要求不运行测试，只清理/维护测试源码并静态检查引用；动态验收保持未验证，不操作当前 Grok Build。
