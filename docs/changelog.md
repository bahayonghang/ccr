# 更新日志

CCR 的所有重要变更都会记录在本文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.2.3] - 2025-01-10

### ✨ 新增

- **配置初始化 (Init)**: 快速创建配置文件
  - 从内置模板自动创建 `~/.ccs_config.toml`
  - 包含 8 个常用 API 服务的预配置模板
  - 智能检测现有配置，避免意外覆盖
  - 交互式确认机制
  - 自动备份现有配置
  - 正确的文件权限设置（Unix: 644）
  
- **配置导出 (Export)**: 导出配置到文件
  - **默认包含 API 密钥**，方便备份和迁移 🔑
  - 自动生成带时间戳的文件名
  - 支持 `--no-secrets` 参数导出不含密钥的配置
  - TOML 格式输出，易于编辑
  - 完美适用于备份、迁移和团队协作
  
- **配置导入 (Import)**: 从文件导入配置
  - 支持两种导入模式：
    - **合并模式** (`--merge`): 保留现有配置，添加新的
    - **替换模式** (默认): 完全替换现有配置
  - 导入前自动备份现有配置（可选 `--no-backup` 禁用）
  - 配置验证和完整性检查
  - 详细的导入摘要报告

### 🔧 改进

- **导出默认行为优化**: 
  - 从默认不包含密钥改为默认包含密钥
  - 参数从 `--include-secrets` 改为 `--no-secrets`
  - 更符合用户备份和迁移的实际需求
  
- **文档体系完善**:
  - 新增 `docs/FEATURES.md` - 完整功能说明（12KB）
  - 新增 `docs/INIT_IMPORT_EXPORT.md` - 详细的导入导出指南（11KB）
  - 新增 `docs/README.md` - 文档中心索引
  - 更新所有相关文档

### 📝 新增命令

```bash
ccr init                    # 初始化配置文件
ccr export                  # 导出配置（含密钥）
ccr export --no-secrets     # 导出配置（不含密钥）
ccr import <file> --merge   # 合并导入
ccr import <file>           # 替换导入
```

### 🧪 测试

- 新增 6 个单元测试，全部通过
  - `init::tests` - 3 个测试
  - `export::tests` - 2 个测试
  - `import::tests` - 1 个测试

### 📦 文件变更

**新增文件**:
- `src/commands/init.rs` (140 行)
- `src/commands/export.rs` (120 行)
- `src/commands/import.rs` (190 行)
- `docs/FEATURES.md`
- `docs/INIT_IMPORT_EXPORT.md`
- `docs/README.md`

**修改文件**:
- `src/commands/mod.rs` - 导出新命令
- `src/main.rs` - 集成新子命令
- `README.md` - 更新功能说明
- `README_CN.md` - 更新中文文档
- `CLAUDE.md` - 更新开发文档

## [0.2.2] - 2025-01-09

### ✨ 新增

- **自动更新功能 (Update)**: 一键更新到最新版本
  - 自动从 GitHub releases 检查更新
  - 智能版本比较算法
  - 自动识别当前平台并下载对应二进制
  - 支持多平台：
    - Linux: x86_64, aarch64
    - macOS: x86_64, Apple Silicon (aarch64)
    - Windows: x86_64
  - 原子更新机制，确保安全
  - 安装后自动验证
  - 下载进度显示

### 🔧 改进

- **依赖更新**:
  - 新增 `self_update` v0.42 - 自更新功能核心
  - 新增 `reqwest` v0.12.23 - HTTP 客户端
  - 新增 `tokio` v1 - 异步运行时

### 📝 新增命令

```bash
ccr update --check    # 检查更新
ccr update            # 执行更新
```

### 📦 文件变更

**新增文件**:
- `src/commands/update.rs` (240 行)

**修改文件**:
- `Cargo.toml` - 添加更新相关依赖
- `src/commands/mod.rs` - 导出 update 命令
- `src/main.rs` - 集成 update 子命令

## [0.2.0] - 2025-01-10

### ✨ 新增

- **Web 管理界面**: 完整的 Web UI，支持可视化配置管理
  - 现代化深色主题设计
  - 实时配置验证
  - 历史记录查看
  - RESTful API 接口
- **完整审计追踪**: 记录所有操作历史
  - UUID 唯一标识
  - 时间戳和操作者信息
  - 环境变量变更追踪
  - 敏感信息自动掩码
- **自动备份系统**: 切换配置前自动备份
  - 带时间戳的备份文件
  - 备份列表查看
  - 从备份恢复（预留功能）

### 🔧 改进

- **性能优化**: 编译优化配置
  - LTO (链接时优化)
  - 单编译单元
  - 符号剥离
- **错误处理**: 13 种详细错误类型
  - 统一错误码
  - 用户友好的错误消息
  - 致命错误识别
- **彩色输出**: 增强的终端体验
  - 成功/信息/警告/错误不同颜色
  - 进度指示
  - Banner 显示

### 🐛 修复

- 修复文件锁在某些情况下未正确释放的问题
- 修复配置验证时的边界条件处理
- 修复 Web 界面在某些浏览器的兼容性问题

## [0.1.0] - 2024-12-15

### ✨ 初始版本

- **核心功能**: 基本的配置管理
  - 列出配置
  - 切换配置
  - 查看当前状态
  - 配置验证
- **直接写入**: 操作 `~/.claude/settings.json`
- **文件锁机制**: 并发安全保证
- **TOML 配置**: 与 CCS 兼容的配置格式
- **命令行界面**: 基于 Clap 的 CLI
- **跨平台支持**: Linux / macOS / Windows

### 🔧 技术实现

- Rust 2024 Edition
- Serde 序列化
- TOML 解析
- File locking (fs4)
- Atomic file operations (tempfile)

---

## 未来计划

### [0.3.0] - 计划中

- [x] ~~配置导入/导出功能~~ (已在 v0.2.3 实现)
- [x] ~~配置初始化~~ (已在 v0.2.3 实现)
- [ ] 配置模板系统
- [ ] 批量操作支持
- [ ] 更详细的历史记录筛选
- [ ] 配置版本管理

### [0.4.0] - 规划中

- [ ] 插件系统
- [ ] 自定义命令
- [ ] API 扩展接口
- [ ] 性能监控
- [ ] 更多统计功能

### [1.0.0] - 稳定版

- [ ] 完整的测试覆盖
- [ ] 详细的文档
- [ ] 稳定的 API
- [ ] 长期支持

---

## 版本说明

- **主版本号 (Major)**: 不兼容的 API 变更
- **次版本号 (Minor)**: 向下兼容的功能性新增
- **修订号 (Patch)**: 向下兼容的问题修正

## 贡献

如果你发现了 Bug 或有新的功能建议，欢迎：

- 提交 [Issue](https://github.com/bahayonghang/ccs/issues)
- 创建 [Pull Request](https://github.com/bahayonghang/ccs/pulls)

## 感谢

感谢所有为 CCR 做出贡献的开发者！

---

[0.2.3]: https://github.com/bahayonghang/ccr/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/bahayonghang/ccr/compare/v0.2.0...v0.2.2
[0.2.0]: https://github.com/bahayonghang/ccr/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/bahayonghang/ccr/releases/tag/v0.1.0

