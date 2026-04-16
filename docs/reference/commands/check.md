# check - 配置冲突检测

检测多平台环境变量/配置项的潜在冲突，避免互相覆盖。

如果你想要先看一份统一诊断总览，请使用 [`ccr doctor`](./doctor)；`check` 只负责局部冲突扫描。

**支持版本**：v3.6.0+

## 子命令

### conflicts

扫描平台配置并提示可能的键冲突。

```bash
ccr check conflicts
```

输出示例：
```
⚠️ 检测到冲突:
- CLAUDE_API_KEY 同时出现在 claude / codex
```

> 建议在多平台注册和迁移后执行，确保 key 命名一致或按平台区分。

## 与其他诊断命令的边界

- [`doctor`](./doctor)：统一体检入口，会把冲突扫描纳入总览
- [`validate`](./validate)：静态校验配置与 settings
- [`provider`](./provider)：在线/连通性导向的 Provider 健康检查
