# current - 运行时状态总览

`ccr current` 现在展示的是并列的 Claude Runtime 与 Codex Runtime，而不是单一“当前平台”。

## 用法

```bash
ccr current
ccr current --verbose
ccr current --json
```

## 输出模型

默认输出：

- Claude Runtime 状态卡片
- Codex Runtime 状态卡片
- 每个平台的当前 profile / provider / auth / health 摘要

`--verbose` 额外显示：

- registry 目标信息
- 对应平台路径
- 当前 profile 详情
- 环境变量与设置细节

`--json` 输出：

- `schema_version`
- `generated_at`
- `claude`
- `codex`

> 顶层不再输出 `current_platform`。

## 适用场景

```bash
ccr current
ccr current --verbose
```

- 确认 Claude / Codex 哪个 runtime 正在就绪
- 观察 profile mode、official auth、provider key 等状态
- 给 VS Code、脚本或调试流程提供统一 runtime 总览

## 相关页面

- [platform](./platform)
- [validate](./validate)
- [迁移指南](/reference/migration)
