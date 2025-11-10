# CCR UI Backend 文档

欢迎查阅 CCR UI 后端完整文档！本目录包含了从架构设计到部署运维的全方位指南。

## 📢 重要更新

**v1.2.0 (2025-01-15)**: 后端框架已从 Actix Web 迁移到 Axum

- ✅ 更好的性能和更低的内存占用
- ✅ 更强的类型安全和更简洁的代码
- ✅ 完整的 Agents 和 Commands 文件夹分组支持
- ✅ 智能 Markdown 文件解析（支持有/无 frontmatter）

详细信息请查看 **[Axum 迁移说明](./MIGRATION_AXUM.md)**。

## 📚 文档导航

### 🏗️ 架构与设计

- **[架构设计](./architecture.md)** - 后端整体架构、模块划分、数据流
  - 系统架构图
  - 技术栈总览
  - 项目结构
  - 核心模块设计

- **[技术栈详解](./tech-stack.md)** - 使用的技术和工具链
  - Axum Web 框架
  - Tokio 异步运行时
  - SQLx 数据库访问
  - Tracing 日志系统
  - 代码示例和最佳实践

### 🔧 开发指南

- **[开发指南](./development.md)** - 完整的开发流程和规范
  - 环境搭建
  - 项目结构
  - 开发工作流
  - 代码规范
  - 测试策略

- **[错误处理](./error-handling.md)** - 错误处理机制和最佳实践
  - 错误类型定义
  - 错误处理策略
  - 日志记录
  - 监控告警

### 🚀 部署与运维

- **[部署指南](./deployment.md)** - 部署方案和最佳实践
  - 本地部署
  - Docker 容器化
  - Kubernetes 部署
  - 云平台部署
  - 生产环境优化

### 📡 API 文档

- **[API 文档](./api.md)** - 完整的 REST API 接口文档
  - API 概览
  - 配置管理接口
  - MCP 服务器管理
  - Slash Commands 管理
  - Agents 管理
  - 插件管理
  - 系统信息接口

### 🔄 迁移指南

- **[Axum 迁移说明](./MIGRATION_AXUM.md)** ⭐ **必读**
  - 迁移概述
  - 代码对照表
  - 核心改进
  - 修复的问题
  - 性能提升数据

## 🎯 快速开始

### 新手开发者

如果你是第一次接触本项目，建议按以下顺序阅读：

1. **[架构设计](./architecture.md)** - 了解整体架构
2. **[开发指南](./development.md)** - 搭建开发环境
3. **[API 文档](./api.md)** - 熟悉 API 接口
4. **[技术栈详解](./tech-stack.md)** - 深入了解技术细节

### 维护人员

如果你负责维护和部署，重点关注：

1. **[Axum 迁移说明](./MIGRATION_AXUM.md)** - 了解最新变更
2. **[部署指南](./deployment.md)** - 部署和运维
3. **[错误处理](./error-handling.md)** - 错误监控和处理

## 🔑 核心特性

### 1. Agents 管理

支持从 `~/.claude/agents/` 目录读取 Agents 配置：

- ✅ 自动识别子文件夹（如 `kfc/`）
- ✅ 支持带/不带 YAML frontmatter 的 Markdown
- ✅ 文件夹分组展示
- ✅ 优雅降级到 settings.json

**API 响应示例**:
```json
{
  "success": true,
  "data": {
    "agents": [
      {
        "name": "bugfix",
        "folder": "",
        "tools": ["Read", "Edit"]
      },
      {
        "name": "spec-design",
        "folder": "kfc",
        "tools": []
      }
    ],
    "folders": ["kfc"]
  }
}
```

### 2. Slash Commands 管理

支持从 `~/.claude/commands/` 目录读取 Commands：

- ✅ 自动从 Markdown 内容提取描述
- ✅ 支持子文件夹分组
- ✅ 无需 YAML frontmatter

### 3. 配置管理

- ✅ 配置列表、切换、验证
- ✅ 操作历史记录
- ✅ 备份和恢复

### 4. 系统监控

- ✅ 健康检查
- ✅ 系统信息获取
- ✅ 版本管理

## 📊 技术栈一览

| 组件 | 技术 | 版本 |
|------|------|------|
| Web 框架 | Axum | 0.7 |
| 中间件 | Tower + Tower-HTTP | 0.5/0.6 |
| 异步运行时 | Tokio | 1.42 |
| 序列化 | Serde | 1.0 |
| 日志 | Tracing | 0.1 |
| 错误处理 | Anyhow + Thiserror | 1.0/2.0 |

## 🛠️ 开发工具

```bash
# 启动开发服务器
cargo run --release

# 运行测试
cargo test

# 代码检查
cargo clippy

# 代码格式化
cargo fmt

# 构建发布版本
cargo build --release
```

## 📞 获取帮助

- **Issue 反馈**: [GitHub Issues](https://github.com/your-org/ccr/issues)
- **文档问题**: 提交 PR 修改本文档
- **技术讨论**: 查看 [Discussions](https://github.com/your-org/ccr/discussions)

## 🤝 贡献指南

欢迎贡献文档和代码！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 提交 Pull Request

### 文档贡献

如果发现文档错误或需要改进：

1. 直接编辑对应的 Markdown 文件
2. 确保示例代码可以正常运行
3. 保持文档风格一致
4. 提交 PR 并说明改进内容

## 📅 更新日志

### v1.2.0 (2025-01-15)

- 🔄 从 Actix Web 迁移到 Axum
- ✨ 完整支持 Agents 文件夹分组
- ✨ 完整支持 Slash Commands 文件夹分组
- 🐛 修复子文件夹文件读取问题
- 🐛 修复 Markdown 无 frontmatter 解析问题
- 📝 完善文档和迁移指南

## 📄 许可证

本项目采用 MIT License 开源许可证

---

**最后更新**: 2025-01-15  
**维护者**: CCR Team  
**文档版本**: v1.2.0

