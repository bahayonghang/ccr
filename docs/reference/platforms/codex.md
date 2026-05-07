# Codex 平台配置指南

Codex 平台当前采用“账号面”和“运行时面”分离的设计。

## 当前主路径

```bash
ccr codex auth current
ccr codex auth list
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## `auth` 与 `profile`

- `ccr codex auth ...`：保存 / 切换 / 导入导出 official auth 账号
- `ccr codex profile ...`：把 profile 写入 `~/.codex/config.toml` 与 `~/.codex/auth.json`
- `ccr codex profile off`：退出 profile mode，恢复到 official auth runtime

## 关键路径

- Runtime config：`~/.codex/config.toml`
- Runtime auth：`~/.codex/auth.json`
- Profiles：`~/.ccr/platforms/codex/profiles.toml`
- Registry pointer：`~/.ccr/config.toml` 中 `[codex].current_profile`

## 历史同步补充

`ccr codex sync-history ...` 仍用于修复 provider namespace 切换后旧历史不可见的问题。

## 迁移提醒

以下旧路径不再推荐：

- `ccr switch <profile>`
- `ccr platform switch codex`
- `ccr platform current`

改用：

- `ccr current`
- `ccr codex profile switch <profile>`
- `ccr codex profile off`
