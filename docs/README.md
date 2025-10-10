# CCR 文档中心

欢迎来到 CCR (Claude Code Configuration Switcher) 文档中心！

## 📚 文档索引

### 快速开始
- [主 README](../README.md) - 项目概述、快速开始和基本使用
- [中文 README](../README_CN.md) - 中文版项目文档

### 功能文档
- [**完整功能说明** (FEATURES.md)](./FEATURES.md) ⭐ **推荐阅读**
  - 所有功能的详细说明
  - 命令参考和使用场景
  - 最佳实践和故障排除

- [配置初始化、导入导出 (INIT_IMPORT_EXPORT.md)](./INIT_IMPORT_EXPORT.md)
  - init 命令详细说明
  - export/import 功能使用
  - 安全考虑和最佳实践

### 开发文档
- [开发指南 (CLAUDE.md)](../CLAUDE.md)
  - 项目架构和设计
  - 开发命令和测试
  - 模块关系和代码结构

## 📖 按主题浏览

### 配置管理
```bash
# 初始化配置
参见: FEATURES.md > 配置初始化

# 列出和切换配置
参见: FEATURES.md > 配置列表、配置切换

# 验证配置
参见: FEATURES.md > 配置验证
```

### 导入导出
```bash
# 详细的导入导出指南
参见: INIT_IMPORT_EXPORT.md

# 快速参考
参见: FEATURES.md > 配置导出、配置导入
```

### 高级功能
```bash
# 操作历史
参见: FEATURES.md > 操作历史

# Web 界面
参见: FEATURES.md > Web 界面

# 自动更新
参见: FEATURES.md > 自动更新
```

## 🎯 常见任务

### 首次使用 CCR
1. 阅读 [README.md](../README.md) 了解基本概念
2. 执行 `ccr init` 初始化配置
3. 参考 [INIT_IMPORT_EXPORT.md](./INIT_IMPORT_EXPORT.md) 配置服务

### 备份和迁移
1. 参考 [INIT_IMPORT_EXPORT.md](./INIT_IMPORT_EXPORT.md) > 使用场景
2. 使用 `ccr export` 导出配置
3. 使用 `ccr import` 导入配置

### 团队协作
1. 参考 [FEATURES.md](./FEATURES.md) > 使用场景 > 团队协作
2. 使用 `ccr export --no-secrets` 创建模板
3. 团队成员使用 `ccr import --merge` 导入

### 开发和贡献
1. 阅读 [CLAUDE.md](../CLAUDE.md) 了解项目架构
2. 参考开发命令进行本地开发
3. 运行测试确保代码质量

## 🔍 快速查找

### 命令帮助
```bash
# 查看所有命令
ccr --help

# 查看特定命令帮助
ccr <command> --help
```

### 按命令查找

| 命令 | 文档位置 |
|------|---------|
| `ccr init` | [FEATURES.md](./FEATURES.md#1-配置初始化-init) |
| `ccr list` | [FEATURES.md](./FEATURES.md#2-配置列表-list) |
| `ccr switch` | [FEATURES.md](./FEATURES.md#3-配置切换-switch) |
| `ccr current` | [FEATURES.md](./FEATURES.md#4-当前状态-current) |
| `ccr validate` | [FEATURES.md](./FEATURES.md#5-配置验证-validate) |
| `ccr export` | [FEATURES.md](./FEATURES.md#6-配置导出-export) + [INIT_IMPORT_EXPORT.md](./INIT_IMPORT_EXPORT.md#export-命令) |
| `ccr import` | [FEATURES.md](./FEATURES.md#7-配置导入-import) + [INIT_IMPORT_EXPORT.md](./INIT_IMPORT_EXPORT.md#import-命令) |
| `ccr history` | [FEATURES.md](./FEATURES.md#8-操作历史-history) |
| `ccr web` | [FEATURES.md](./FEATURES.md#9-web-界面-web) |
| `ccr update` | [FEATURES.md](./FEATURES.md#10-自动更新-update) |

## 📝 文档版本

| 文档 | 版本 | 最后更新 |
|------|------|---------|
| FEATURES.md | v0.2.3 | 2024-01-10 |
| INIT_IMPORT_EXPORT.md | v0.2.3 | 2024-01-10 |
| README.md | v0.2.3 | 2024-01-10 |
| CLAUDE.md | v0.2.3 | 2024-01-10 |

## 🆕 最新功能

### v0.2.3 新增功能
- ✅ **配置初始化** (`ccr init`) - 从模板快速创建配置
- ✅ **配置导出** (`ccr export`) - 默认包含密钥，方便迁移
- ✅ **配置导入** (`ccr import`) - 支持合并和替换模式

详细说明请参阅:
- [FEATURES.md](./FEATURES.md)
- [INIT_IMPORT_EXPORT.md](./INIT_IMPORT_EXPORT.md)

## 💡 提示

### 新用户
建议按以下顺序阅读：
1. [README.md](../README.md) - 了解 CCR 是什么
2. [FEATURES.md](./FEATURES.md) - 了解所有功能
3. [INIT_IMPORT_EXPORT.md](./INIT_IMPORT_EXPORT.md) - 学习配置管理

### 高级用户
直接跳转到：
- [FEATURES.md](./FEATURES.md) - 查看高级功能
- [CLAUDE.md](../CLAUDE.md) - 了解内部实现

### 开发者
重点阅读：
- [CLAUDE.md](../CLAUDE.md) - 项目架构
- [FEATURES.md](./FEATURES.md) - API 和命令参考

## 🔗 相关链接

- [GitHub 仓库](https://github.com/bahayonghang/ccr)
- [问题反馈](https://github.com/bahayonghang/ccr/issues)
- [贡献指南](../README.md#contributing)

## 📮 获取帮助

如果你在使用过程中遇到问题：

1. **查看文档**: 先查看相关文档
2. **命令帮助**: 运行 `ccr <command> --help`
3. **故障排除**: 参考 [FEATURES.md](./FEATURES.md#故障排除)
4. **提交 Issue**: 在 GitHub 上提交问题

---

**欢迎贡献！** 如果你发现文档有误或需要改进，请提交 PR。
