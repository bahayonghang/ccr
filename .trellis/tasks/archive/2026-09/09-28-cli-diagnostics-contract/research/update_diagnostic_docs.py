from pathlib import Path

documents = {
    'docs/reference/commands/validate.md': '''# validate - 验证配置

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

## 分类与退出码

| 状态 | 分类 | 命令退出码 |
| --- | --- | --- |
| 合法且没有错误 | `valid` | `0` |
| 未配置 profile、缺少 runtime settings、inactive、普通禁用 profile | `missing` / `inactive` / `disabled` 警告 | `0` |
| Profile 领域校验失败或 disabled current | `invalid` / `disabled` 错误 | `90` |
| Profile TOML 语法或类型损坏 | `corrupt` | `14` |
| Profile/runtime 文件读取失败 | `unreadable` | `51` |
| 已存在的 runtime JSON / TOML 损坏 | `corrupt` | `40` / `41` |
| 记录的 current profile 已不存在 | `invalid` | `62` |

其他错误保留 `CcrError` 的既有数值映射。多个错误同时出现时，按照注册表、Claude、Codex、Grok 的稳定顺序返回第一个错误码。只有警告时返回 `0`。错误报告返回成功退出的旧行为已修复。

`missing` 表示路径不存在。权限拒绝、共享句柄拒绝、路径指向目录等读取失败不会被归类为未配置。

## 使用示例

```bash
ccr validate
ccr validate || echo "CCR validation failed"
```

终端报告显示 severity、category、平台和目标。报告不输出 token、损坏文档原文或被拒绝的字段值。诊断不会调用 apply；修复配置后，由用户明确运行平台的 `profile switch`。

需要全局配置、运行认证、冲突或可选在线检查时，使用 [`ccr doctor`](./doctor)。Grok 官方会话由 Grok 负责，离线诊断不证明真实登录成功。Gemini/Droid 保留兼容 adapter，未扩展 auth/profile 命令支持。
''',
    'docs/en/reference/commands/validate.md': '''# validate - Configuration Validation

`ccr validate` reads Claude, Codex, and Grok profile documents and existing runtime settings. The command does not initialize files, repair current markers, create backups, or create log directories.

## Usage

```bash
ccr validate
```

## Scope

Profiles use `<CCR_ROOT>/platforms/<platform>/profiles.toml`. The registry uses `<CCR_ROOT>/config.toml`. Runtime paths follow the existing Claude, Codex, and Grok path resolvers.

Each platform uses the same domain validator as profile application. Claude subscription profiles do not require API-key fields. Codex validates the selected OpenAI or provider auth mode. Grok validates official session profiles and third-party API-key or environment-key profiles separately.

A disabled profile can remain in the document. The command reports a warning for an inactive disabled profile and an error for a disabled current profile. A simplified document without a current marker and an existing empty document remain inactive.

## Exit Codes

| Condition | Category | Exit code |
| --- | --- | --- |
| Valid configuration, or warnings only | `valid`, `missing`, `inactive`, `disabled` | `0` |
| Invalid profile or disabled current profile | `invalid`, `disabled` | `90` |
| Invalid profile TOML syntax or field type | `corrupt` | `14` |
| Profile or runtime read failure | `unreadable` | `51` |
| Invalid existing runtime JSON or TOML | `corrupt` | `40` or `41` |
| Recorded current profile does not exist | `invalid` | `62` |

Other failures retain the existing `CcrError` code. When several errors occur, the first error in registry, Claude, Codex, Grok order determines the exit code. Error reports no longer return a successful process status.

A missing path differs from denied access, a sharing violation, or a directory at a file path. Read errors are not treated as an unconfigured profile.

## Output and Boundaries

The terminal report contains severity, category, platform, and target. Reports omit credentials, source documents, and rejected field values. Diagnostics do not apply profiles.

Use [doctor](./doctor) for the environment, runtime auth, conflicts, and optional online checks. Grok owns official session authentication. Offline diagnostics do not prove that a session can log in. Gemini and Droid retain compatibility adapters; their auth/profile command support has not expanded.
''',
}
for name, content in documents.items():
    Path(name).write_text(content, encoding='utf-8', newline='\n')

path = Path('docs/reference/commands/doctor.md')
text = path.read_text(encoding='utf-8')
text = text.replace('全局 CCR 状态 + 当前平台深度检查', '全局 CCR 状态 + 已配置 Claude/Codex/Grok 平台检查')
text = text.replace('全局 + 当前平台，本地只读检查', '全局 + 已配置 Claude/Codex/Grok，本地只读检查')
text = text.replace('- 当前平台是否可解析且为已实现平台', '- 平台能力是否属于当前支持、保留的 legacy adapter 或未实现')
text = text.replace('- 有 `failed` 项时返回非 0', '- 有 `failed` 项时返回 `1`，保留既有 doctor 退出码')
text = text.replace('> `--platform` 与 `--all-platforms` 互斥。', '> `--platform` 与 `--all-platforms` 互斥。平台取值从共享 `Platform` 枚举派生，包括 `grok`。\n\nClaude/Codex/Grok 支持 auth/profile 命令。Gemini/Droid 保留已有只读兼容检查，报告 `legacy_adapter`，没有新增 writer 调用或 auth/profile 命令。Qwen 报告未实现。Grok 配置检查覆盖 TOML 和受管路由一致性；官方会话真实性仍由 Grok 负责。')
path.write_text(text, encoding='utf-8', newline='\n')

path = Path('docs/en/reference/commands/doctor.md')
text = path.read_text(encoding='utf-8').replace('configured Claude/Codex runtime targets rather than a legacy global current platform', 'configured Claude/Codex/Grok runtime targets')
text += '\n## Capability and Exit Contract\n\nPlatform choices come from the shared `Platform` enum and include `grok`. Claude, Codex, and Grok support auth/profile commands. Gemini and Droid retain read-only compatibility checks with a `legacy_adapter` label. Qwen remains unimplemented. No additional legacy writer is called.\n\nGrok checks cover config TOML and managed route consistency. Grok owns official session authentication; offline checks do not prove a valid login.\n\nFailed checks return `1`. Warnings alone return `0`. The binary chooses process status from the typed report. Embedded services never terminate the host process. Local diagnostics do not create log directories or mutate configuration files.\n'
path.write_text(text, encoding='utf-8', newline='\n')
