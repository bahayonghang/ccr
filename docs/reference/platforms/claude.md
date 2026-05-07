# Claude 平台指南

Claude 是当前显式 runtime/profile 模型中的一等平台。

## 当前主路径

```bash
ccr claude auth current
ccr claude profile list
ccr claude profile switch <name>
ccr claude profile current
ccr claude profile off
```

## 模型说明

- `ccr claude auth ...`：管理 official auth 账号与登录态
- `ccr claude profile ...`：把某个 profile 应用到 `~/.claude/settings.json`
- `ccr claude profile off`：退出 profile mode，回到 official auth runtime

## 关键路径

- Runtime settings：`~/.claude/settings.json`
- Profiles：`~/.ccr/platforms/claude/profiles.toml`
- Registry pointer：`~/.ccr/config.toml` 中 `[claude].current_profile`

## 迁移提醒

以下旧路径不再推荐：

- `ccr platform switch claude`
- `ccr platform current`
- `ccr switch <name>`

改用：

- `ccr current`
- `ccr claude profile switch <name>`
- `ccr claude profile off`
