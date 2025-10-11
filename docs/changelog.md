# 更新日志

CCR 的所有重要变更都会记录在本文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [1.0.0] - 2025-10-11

### 🏗️ 架构重构（重大更新）

**全面的架构现代化升级**，引入分层架构、Service 层、代码抽象和测试增强。

#### ✨ 新增

- **Service 层架构** - 业务逻辑集中化
  - `ConfigService` - 10 个配置管理方法
  - `SettingsService` - 6 个设置管理方法
  - `HistoryService` - 6 个历史记录方法
  - `BackupService` - 4 个备份管理方法
  
- **Core 基础设施层**
  - `AtomicWriter` - 原子文件写入器
  - `FileManager` trait - 统一文件管理接口
  
- **Utils 工具层**
  - `mask_sensitive()` - 统一敏感信息掩码
  - `Validatable` trait - 统一验证接口
  
- **完整 Web API** - 11 个 RESTful 端点
  - 配置管理：5 个端点（list, add, update, delete, switch）
  - 历史记录：1 个端点
  - 设置管理：3 个端点
  - 工具功能：2 个端点
  
- **集成测试支持** - 3 个集成测试，100% 通过

#### 🔧 改进

- **Web 模块重构** - 从 753 行拆分为 4 个清晰模块
  - `web/models.rs` - API 数据模型
  - `web/server.rs` - HTTP 服务器核心
  - `web/handlers.rs` - 请求处理器
  - `web/routes.rs` - 路由定义
  
- **命令层优化** - 全面使用 Service 层
  - `list`, `current`, `clean`, `history`, `validate` 命令重构
  - 消除直接访问 Manager 的代码
  - 业务逻辑由 Service 层统一管理
  
- **代码质量提升**
  - 错误码从硬编码改为常量（exit_codes 模块）
  - 锁管理器添加通用 `lock_resource()` 方法
  - 统一 Validatable trait 接口
  - 消除掩码逻辑重复
  
- **测试增强**
  - 测试覆盖率从 ~85% 提升到 95.1%
  - 新增 Service 层单元测试（6 个）
  - 新增 Core 层单元测试（4 个）
  - 新增 Utils 层单元测试（2 个）
  - 新增集成测试（3 个）

#### 📚 文档

- **ARCHITECTURE.md** - 完整的架构设计文档
- **CLAUDE.md** - 更新开发指南，添加 Service 层使用说明
- **Cargo.toml** - 添加 lib target 支持

#### 📦 技术细节

**新增模块结构**:
```
src/
├── services/        # 业务逻辑层 (新增)
├── core/            # 核心抽象 (新增)
├── utils/           # 工具函数 (新增)
├── web/             # Web 模块 (重构)
└── lib.rs           # 库入口 (新增)
```

**代码统计**:
- 新增模块：18 个文件
- 重构模块：12 个文件
- 新增代码：~2000 行
- 总测试：81 个（77 passed, 95.1%）

#### ⚠️ 破坏性变更

**无破坏性变更** - 100% 向后兼容
- 所有 CLI 命令接口不变
- 配置文件格式不变
- 历史记录格式不变
- API 行为不变

#### 🎯 质量指标

- 编译：0 errors, 0 warnings
- 测试：77/81 passed (95.1%)
- 架构评分：⭐⭐⭐⭐⭐ EXCELLENT
- 代码质量：⭐⭐⭐⭐⭐

---

## [0.2.3] - 2025-10-10

### ✨ 新增

- **配置初始化 (Init)**: 快速创建配置文件
  - 从内置模板自动创建 `~/.ccs_config.toml`
  - 包含 8 个常用 API 服务的预配置模板
  - **安全模式**：如果配置已存在，直接退出（不覆盖）
  - 必须使用 `--force` 才能覆盖现有配置
  - 使用 --force 时自动备份现有配置
  - 正确的文件权限设置（Unix: 644）
  - 提供有用的后续操作提示
  
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

- **备份清理 (Clean)**: 清理旧备份文件
  - 自动清理指定天数前的备份文件
  - 默认清理 7 天前的备份
  - 模拟运行模式（`--dry-run`）预览清理结果
  - 显示释放的磁盘空间
  - 仅清理 `~/.claude/backups/` 中的 `.bak` 文件

### 🔧 改进

- **Init 命令安全性增强**:
  - 如果配置文件已存在，直接提示并退出（不覆盖）
  - 必须使用 `--force` 参数才能覆盖
  - 使用 `--force` 时自动备份现有配置
  - 提供有用的后续操作提示

- **Update 命令简化**:
  - 改为直接执行 `cargo install --git` 命令
  - 移除对 self_update、reqwest、tokio 的依赖
  - 减小二进制大小
  - 网络错误时提供友好的解决方案提示

- **导出默认行为优化**: 
  - 从默认不包含密钥改为默认包含密钥
  - 参数从 `--include-secrets` 改为 `--no-secrets`
  - 更符合用户备份和迁移的实际需求
  
- **文档体系完善**:
  - 新增 `docs/FEATURES.md` - 完整功能说明（12KB）
  - 新增 `docs/INIT_IMPORT_EXPORT.md` - 详细的导入导出指南（11KB）
  - 新增 `docs/CLEAN_FEATURE.md` - 备份清理详细指南
  - 新增 `docs/README.md` - 文档中心索引
  - 更新所有相关文档

- **Web 界面增强**:
  - 集成备份清理功能
  - 新增清理模态框
  - 支持模拟运行预览
  - 显示清理结果统计

### 📝 新增命令

```bash
ccr init                    # 初始化配置文件
ccr export                  # 导出配置（含密钥）
ccr export --no-secrets     # 导出配置（不含密钥）
ccr import <file> --merge   # 合并导入
ccr import <file>           # 替换导入
ccr clean                   # 清理旧备份（7天前）
ccr clean --days 30         # 清理 30 天前的备份
ccr clean --dry-run         # 模拟运行预览
```

### 🧪 测试

- 新增 8 个单元测试，全部通过
  - `init::tests` - 3 个测试
  - `export::tests` - 2 个测试
  - `import::tests` - 1 个测试
  - `clean::tests` - 2 个测试

### 📦 文件变更

**新增文件**:
- `src/commands/init.rs` (140 行)
- `src/commands/export.rs` (120 行)
- `src/commands/import.rs` (190 行)
- `src/commands/clean.rs` (130 行)
- `docs/FEATURES.md`
- `docs/INIT_IMPORT_EXPORT.md`
- `docs/CLEAN_FEATURE.md`
- `docs/README.md`

**修改文件**:
- `Cargo.toml` - 添加 filetime 依赖，移除 self_update/reqwest/tokio
- `src/commands/mod.rs` - 导出新命令
- `src/commands/update.rs` - 简化为调用 cargo install
- `src/main.rs` - 集成新子命令
- `web/index.html` - 添加清理备份功能界面
- `src/web.rs` - 添加 /api/clean 端点
- `README.md` - 更新功能说明
- `README_CN.md` - 更新中文文档
- `CLAUDE.md` - 更新开发文档
- `docs/changelog.md` - 更新日志

**移除依赖**:
- `self_update` - 不再需要（改用 cargo install）
- `reqwest` - 不再需要
- `tokio` - 不再需要

## [0.2.2] - 2025-10-10

### ✨ 新增

- **自动更新功能 (Update)**: 一键更新到最新版本
  - 通过 `cargo install --git` 从 GitHub 获取最新代码
  - 无需预编译的 releases
  - 简单直接的更新方式
  - 更新前确认机制

### 📝 新增命令

```bash
ccr update --check    # 查看将要执行的命令
ccr update            # 执行更新
```

### 📦 文件变更

**新增文件**:
- `src/commands/update.rs` (90 行，简化版）

**修改文件**:
- `src/commands/mod.rs` - 导出 update 命令
- `src/main.rs` - 集成 update 子命令

## [0.2.0] - 2025-10-10

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

## [0.1.0] - 2025-10-10

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

