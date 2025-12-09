# platform - 多平台管理

管理平台注册表 (`~/.ccr/config.toml`)，支持列出、切换、查看、初始化平台。

**支持版本**：v3.6.0+

## 子命令

### list

列出所有支持的平台，并标记当前平台。

```bash
ccr platform list [--json]
```

- `--json`：以 JSON 输出，便于脚本。

### switch

切换当前使用的平台（不会修改各平台 profiles 内容，只更新注册表指针）。

```bash
ccr platform switch <platform>
```

示例：
```bash
ccr platform switch codex
ccr platform switch gemini
```

### current

查看当前平台。

```bash
ccr platform current [--json]
```

### info

查看指定平台的详细信息（实现状态、路径等）。

```bash
ccr platform info <platform> [--json]
```

### init

为指定平台创建目录结构与 `profiles.toml` 模板（若不存在）。

```bash
ccr platform init <platform>
```

## 平台标识

- 已实现：`claude`、`codex`、`gemini`
- 预留/Stub：`qwen`、`iflow`

## 常用组合

```bash
ccr platform list
ccr platform switch claude
ccr add                     # 在当前平台新增 profile
ccr list && ccr switch xxx  # 针对当前平台的 profile 列表与切换
```
