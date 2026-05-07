# switch - 已退休的旧 profile 切换入口

`ccr switch <config_name>` 曾经会根据全局 `current_platform` 推断目标平台。这个入口现在已经退休。

## 当前行为

运行：

```bash
ccr switch <config_name>
```

会返回迁移错误，提示你改用显式平台命令。

## 替代路径

```bash
ccr claude profile switch <config_name>
ccr codex profile switch <config_name>
```

## 为什么退休

- 全局 `current_platform` / `default_platform` 心智模型容易误导用户
- Claude 与 Codex 现在都需要显式 runtime state
- VS Code、doctor、validate、`ccr current` 都已经迁移到按平台表达状态

## 迁移示例

| 旧命令 | 新命令 |
|---|---|
| `ccr switch work` | `ccr claude profile switch work` |
| `ccr switch proxy` | `ccr codex profile switch proxy` |
| `ccr work` | 对应平台显式 `profile switch` |

## 相关页面

- [current](./current)
- [platform](./platform)
- [迁移指南](/reference/migration)
