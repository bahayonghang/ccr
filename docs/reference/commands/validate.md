# validate - 验证配置

`ccr validate` 只读检查 Claude、Codex 和 Grok 的 profile 文档、认证模式和已有运行配置。命令不会初始化文件、修复 current 标记、创建备份或创建日志目录。

## 用法

```bash
ccr validate
```

## 检查范围

- 注册表：`<CCR_ROOT>/config.toml`。
- Profile：`<CCR_ROOT>/platforms/<platform>/profiles.toml`。
- Claude settings：按 `ClaudeRuntimePaths` 的环境变量优先级解析。
- Codex runtime：按 Codex 配置目录解析 `config.toml`。
- Grok runtime：`$GROK_HOME/config.toml`，默认 `~/.grok/config.toml`。

每个平台调用与 apply 相同的领域 validator。Claude subscription 不要求 API-key 字段；API-key 配置按 Claude 规则校验。Codex 使用 OpenAI ChatGPT、OpenAI API key、provider env key、provider bearer token 或 no-auth 对应的规则。Grok official session 和第三方 API key/env key 分开校验。

合法 profile 与可激活状态分别报告。普通禁用 profile 可以保留，产生 `disabled` 警告；记录为 current 的禁用 profile 产生错误。未声明 current 的简化 TOML 和已存在的空文档保持 inactive。

Claude 按有效文件标记、有效注册表标记的顺序选择 current。一个标记悬空或两处标记冲突时，只要存在有效候选，诊断使用该候选并报告 `warning/invalid`，不会修复标记。选中的现有 profile 被禁用时仍返回 `90`，不会转选另一个启用项。存在显式标记但所有候选都不存在时返回 `62`。Codex/Grok 保留各自的既有 current 规则。

Runtime 文件检查不依赖 profile 是否存在。已有 runtime 文件损坏或不可读时仍报告错误；有效 current 的 runtime 文件先完成读取和语法分类，再检查领域运行状态。

## 分类与退出码

| 状态 | 分类 | 命令退出码 |
| --- | --- | --- |
| 合法且没有错误 | `valid` | `0` |
| 未配置 profile、缺少 runtime settings、inactive、普通禁用 profile | `missing` / `inactive` / `disabled` 警告 | `0` |
| Profile 领域校验失败或 disabled current | `invalid` / `disabled` 错误 | `90` |
| Profile TOML 语法或类型损坏 | `corrupt` | `14` |
| Profile/runtime 文件读取失败 | `unreadable` | `51` |
| 已存在的 runtime JSON / TOML 损坏 | `corrupt` | `40` / `41` |
| 显式 current 目标不存在且没有领域规则允许的有效候选 | `invalid` | `62` |

其他错误保留 `CcrError` 的既有数值映射。多个错误同时出现时，按照注册表、Claude、Codex、Grok 的稳定顺序返回第一个错误码。只有警告时返回 `0`。错误报告返回成功退出的旧行为已修复。

`missing` 表示路径不存在。权限拒绝、共享句柄拒绝、路径指向目录等读取失败不会被归类为未配置。

## 使用示例

```bash
ccr validate
ccr validate || echo "CCR validation failed"
```

终端报告显示 severity、category、平台和目标。报告不输出 token、损坏文档原文或被拒绝的字段值。诊断不会调用 apply；修复配置后，由用户明确运行平台的 `profile switch`。

需要全局配置、运行认证、冲突或可选在线检查时，使用 [`ccr doctor`](./doctor)。Grok 官方会话由 Grok 负责，离线诊断不证明真实登录成功。Gemini/Droid 保留兼容 adapter，未扩展 auth/profile 命令支持。
