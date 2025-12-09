# check - 配置冲突检测

检测多平台环境变量/配置项的潜在冲突，避免互相覆盖。

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
