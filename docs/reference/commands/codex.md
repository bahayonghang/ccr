# codex - Codex 多账号管理

`ccr codex` 是 Codex 平台的专项命令组，当前重点能力是 `auth` 子命令。

## 用法

```bash
ccr codex
ccr codex auth <ACTION> [OPTIONS]
```

## 当前支持的子命令

### `ccr codex`

不带子命令时，会进入 Codex 相关的默认交互路径；在启用 TUI 特性时，可作为 Codex 账号管理入口。

### `ccr codex auth`

| 子命令 | 说明 |
|--------|------|
| `save <name>` | 保存当前 `~/.codex/auth.json` 为命名账号 |
| `list` | 列出已保存账号 |
| `switch <name>` | 切换到指定账号 |
| `delete <name>` | 删除已保存账号 |
| `current` | 显示当前账号信息 |
| `export` | 导出账号到 JSON |
| `import` | 从 JSON 导入账号 |

## 常见示例

```bash
# 保存当前登录
ccr codex auth save work

# 带描述和到期时间
ccr codex auth save personal -d "Personal GitHub account" --expires-at 2026-02-01T00:00:00Z

# 查看与切换
ccr codex auth list
ccr codex auth switch work
ccr codex auth current

# 导入导出
ccr codex auth export --no-secrets
ccr codex auth import --replace
```

## 何时使用

- 一个开发者维护多个 GitHub / Codex 登录身份
- 团队共享机器，需要显式切换账号
- 需要导入导出 Codex 登录状态做迁移或备份

## 相关文档

- [平台支持](/reference/platforms/)
- [UI 模块地图](/guide/ui-modules)
