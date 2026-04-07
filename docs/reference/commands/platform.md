# platform - 平台注册表

管理 `~/.ccr/config.toml` 中的平台状态、当前平台指针和平台初始化。

## 用法

```bash
ccr platform <ACTION> [OPTIONS]
```

## 子命令

### list

```bash
ccr platform list [--json]
```

列出当前已知平台和状态。

### switch

```bash
ccr platform switch <platform>
```

切换当前活动平台，但不会修改其他平台的 profile 内容。

### current

```bash
ccr platform current [--json]
```

显示当前平台。

### info

```bash
ccr platform info <platform> [--json]
```

查看指定平台的状态、路径与说明。

### init

```bash
ccr platform init <platform>
```

为平台创建目录结构与模板文件。

## 当前平台键

| 平台键 | 状态 | 备注 |
|--------|------|------|
| `claude` | 已实现 | 默认主线平台 |
| `codex` | 已实现 | 支持 `ccr codex auth` |
| `gemini` | 已实现 | Unified Mode 管理 |
| `droid` | 已实现 | 写入 `~/.factory/settings.json` |
| `qwen` | 预留 / Stub | 当前核心实现返回未支持 |

## 常见命令

```bash
ccr platform list
ccr platform switch claude
ccr platform info droid
ccr platform init gemini
```

## 相关文档

- [平台支持](/reference/platforms/)
- [快速开始](/guide/quick-start)
