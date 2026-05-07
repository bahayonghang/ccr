# 迁移指南

本页说明从旧的全局平台 / profile 路由模型迁移到显式 Claude Runtime / Codex Runtime 模型。

## 命令迁移速查表

| 旧命令 | 当前做法 | 说明 |
|---|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` / `ccr codex profile switch <name>` | 不再隐式推断平台 |
| `ccr <name>` | 同上 | 快捷入口已退休 |
| `ccr platform switch <platform>` | 不再作为 auth/profile 主路径 | 改用显式 profile/auth 命令 |
| `ccr platform current` | `ccr current` | 查看双 runtime |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` | 平台级 profile 命令已显式分离 |

## registry 迁移

- 旧文件可继续包含 `default_platform` / `current_platform`
- 读取时仍兼容这些字段
- 但当前路由真相已经变为各平台条目的 `current_profile`

## 用户心智迁移

旧心智：

- 先选一个“当前平台”
- 再用通用 `switch` 切 profile

新心智：

- `ccr current` 先看 Claude / Codex 双 runtime 状态
- 直接进入 `ccr claude profile ...` 或 `ccr codex profile ...`
- official auth 与 profile routing 分开表达
