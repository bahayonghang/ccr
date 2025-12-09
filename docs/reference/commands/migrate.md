# migrate - 配置迁移 (Legacy → Unified)

将传统单文件模式（`~/.ccs_config.toml`）迁移到默认的 Unified 模式（`~/.ccr/` 注册表 + 平台目录）。

**支持版本**：v3.6.0+

## 用法

```bash
ccr migrate [--check] [--platform <name>]
```

- `--check`：仅检查可迁移性和差异，不写入。
- `--platform <name>`：只迁移指定平台（默认迁移已实现的所有平台）。

## 迁移流程

1) 读取 `~/.ccs_config.toml`  
2) 生成 `~/.ccr/config.toml`（含 current_platform）  
3) 为各平台创建 `platforms/<name>/profiles.toml`  
4) 保留原文件并生成备份，写入历史日志  

## 示例

```bash
# 仅查看需要迁移的内容
ccr migrate --check

# 只迁移 Claude
ccr migrate --platform claude

# 完整迁移
ccr migrate
```

> 提示：迁移后继续兼容 Legacy 模式，后续推荐使用 Unified 工作流（platform + profiles）。
