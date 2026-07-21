# CCR 运行时架构图实施计划

## 实施步骤

- [x] 读取相关 Trellis 规范和 Archify architecture schema/完整示例，确认字段与验证命令。
- [x] 用当前代码证据复核入口、IPC、子进程、持久化、网络调用和安全写入边界。
- [x] 在任务目录创建 `ccr-runtime-architecture.architecture.json`，控制组件数量与主路径，使用 cards 承载次要说明。
- [x] 运行 Archify `inspect` 与 `validate architecture ... --json`，按 Suggested fix 消除越界、重叠和标签冲突。
- [x] 渲染到 `docs/public/architecture/ccr-runtime-architecture.html`。
- [x] 运行 `archify check` 和 Cardinal Rule 自检，确认主题/导出脚本、SVG class、边界与 legend 完整。
- [x] 尝试在浏览器中打开成品；`file://` 被浏览器安全策略拒绝，未绕过策略，改以 Archify check、结构检查和 VitePress build 作为可执行验证。
- [x] 运行 `git diff --check` 并检查最终 diff，只保留任务规划、任务内 JSON 和 HTML。

## 验证命令

```powershell
node bin\archify.mjs inspect architecture <task-json>
node bin\archify.mjs validate architecture <task-json> --json
node bin\archify.mjs render architecture <task-json> <output-html>
node bin\archify.mjs check <output-html>
rg 'fill="(#|rgb)|stroke="(#|rgb)' <output-html>
git diff --check
```

## 风险与回滚点

- CJK 标签较宽：优先缩短标签或增大节点宽度，不通过减小字号硬塞。
- 边界嵌套可能扩大画布：先 inspect 计算 boxes，再调 viewBox/坐标。
- 连接过多会破坏主叙事：仅保留跨边界或非显然数据流，其余写入 cards。
- 渲染器输出不得手改；任何视觉修复都回到 JSON 后重新生成。

## 验证结果

- Archify `validate architecture --json`：通过。
- Archify `check`：通过（单 SVG、有限坐标、正交箭头、legend clearance）。
- Cardinal Rule：未发现 SVG 内硬编码 `fill` / `stroke` 色值；toolbar、主题选择器和导出脚本存在。
- `docs/` 下 `bun run build`：通过。
- `docs/` 下 `bun run audit`：通过。
- `git diff --check`：通过。
- 浏览器视觉检查：未取得，原因是 Browser 安全策略阻止 `file://`，且禁止通过 localhost 或其他浏览器方式绕过。
