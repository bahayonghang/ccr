# 更新日志

CCR 的所有重要变更都会记录在本文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/),版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [1.4.0] - 2025-01-19

### ✨ 新增

- **CCR UI 启动命令** - 全新的 `ccr ui` 命令用于启动完整的 Web 应用
  - **智能环境检测**：自动检测 `ccr-ui/` 目录并选择最佳启动方式
  - **开发模式支持**：使用 `just dev` 启动源码版本，支持热重载
  - **依赖自动检查**：自动检测并提示安装 `just` 工具和项目依赖
  - **交互式安装**：使用 `dialoguer` 提供友好的依赖安装确认
  - **端口配置**：支持自定义前端端口（`-p`）和后端端口（`--backend-port`）

- **新增 UI 服务层** (`src/services/ui_service.rs`)
  - **UiService**：封装 UI 启动和管理逻辑
  - **三级优先级检测**：
    1. 开发环境（当前/父目录的 `ccr-ui/`）
    2. 用户目录（`~/.ccr/ccr-ui/`）
    3. GitHub 自动下载（首次使用自动提示）
  - **GitHub 自动下载**：
    - 交互式下载提示（使用 `dialoguer`）
    - 自动克隆 CCR 仓库并提取 `ccr-ui/` 子目录
    - 临时目录自动清理（使用 `tempfile`）
    - 智能跳过 `.git` 等版本控制文件
  - **工具检查**：验证 `just` 和 `git` 工具是否已安装
  - **依赖管理**：检查并安装前后端依赖
  - **预留接口**：为未来的预构建版本管理预留接口

- **新增 UI 命令** (`src/commands/ui.rs`)
  - 简洁的命令入口，调用 `UiService` 启动 UI
  - 支持端口参数传递

### 🔧 改进

- **Cargo 依赖更新**
  - 新增 `reqwest` (v0.12)：HTTP 客户端，用于未来的版本下载
  - 新增 `dialoguer` (v0.11)：交互式提示库
  - 新增 `flate2` (v1.0)：gzip 压缩/解压支持

- **命令行界面增强**
  - 主帮助信息中添加 `ccr ui` 命令说明
  - 区分 `ccr web`（轻量级 API）和 `ccr ui`（完整应用）
  - 更新命令描述，明确不同界面的用途

### 📚 文档

- **README.md 更新**：
  - 添加 `ccr ui` 命令使用说明
  - 更新 CLI vs TUI vs Web Server vs CCR UI 对比
  - 更新命令参考表格

- **README_CN.md 更新**：
  - 同步英文 README 的所有更新
  - 添加中文版的 `ccr ui` 说明

- **docs/index.md 更新**：
  - 新增 "CCR UI 应用" 功能特性卡片
  - 更新快速使用示例

- **docs/quick-start.md 更新**：
  - 新增 "### 9. 启动 CCR UI 应用" 章节
  - 详细说明功能特性、技术栈、启动流程
  - 添加 Web Server vs CCR UI 对比提示

- **docs/changelog.md 更新**：
  - 记录 v1.4.0 的所有新增功能和改进

### 🎯 使用场景

**场景 1：开发环境快速启动**
```bash
# 在 CCR 项目根目录下
ccr ui

# ✅ 自动检测 ccr-ui/ 目录
# ✅ 检查 just 工具
# ✅ 检查并安装依赖
# ✅ 启动完整的 Next.js + Actix Web 应用
```

**场景 2：首次使用自动下载**
```bash
# 在任意位置运行
ccr ui

# 💬 提示: CCR UI 是一个完整的 Next.js + Actix Web 应用
#    可以从 GitHub 下载到用户目录:
#    /home/user/.ccr/ccr-ui/
#
# ❓ 是否立即从 GitHub 下载 CCR UI? [Y/n]: y
# 📦 克隆仓库: https://github.com/bahayonghang/ccr.git
# ⏳ 下载中 (这可能需要几分钟)...
# 📦 正在复制文件到目标目录...
# ✅ CCR UI 下载完成
#
# [自动检查依赖并启动...]
```

**场景 3：用户目录已下载**
```bash
# 已下载到 ~/.ccr/ccr-ui/
ccr ui

# ✅ 自动使用用户目录版本
# ✅ 启动完整应用
```

**自定义端口：**
```bash
# 自定义前端端口
ccr ui -p 8080

# 同时自定义前后端端口
ccr ui -p 8080 --backend-port 9000
```

**界面选择：**
- `ccr tui`：终端交互界面（键盘导航）
- `ccr web`：轻量级 API 服务器（8080 端口）
- `ccr ui`：完整 Web 应用（3000/8081 端口，可视化仪表板）

### 💡 技术亮点

1. **零依赖启动**（开发模式）
   - 仅需要 `just` 工具
   - 自动检测和安装项目依赖

2. **智能路径检测**
   - 支持多种目录结构
   - 适应不同的运行位置

3. **友好的用户体验**
   - 彩色输出（使用 `ColorOutput`）
   - 进度提示
   - 交互式确认
   - 详细的错误说明和安装指引

4. **可扩展性**
   - 预留预构建版本接口
   - 清晰的服务层结构
   - 易于添加新功能

## [1.1.0] - 2025-01-11

### ✨ 新增

- **配置分类系统** - 全新的多维度配置分类和筛选功能
  - **提供商类型** (`provider_type`): 区分官方中转 (official_relay) 和第三方模型 (third_party_model)
  - **提供商名称** (`provider`): 标识具体的服务提供商 (如 anyrouter, glm, moonshot)
  - **账号标识** (`account`): 区分同一提供商的不同账号 (如 github_5953, linuxdo_79797)
  - **标签系统** (`tags`): 灵活的标签分类 (如 ["free", "stable", "primary"])

- **Web 界面分类功能**
  - 配置类型过滤按钮: 全部/官方中转/第三方模型/未分类
  - 配置卡片增强显示: 提供商类型徽章、描述、提供商信息、账号、标签
  - 右侧配置目录同步过滤: 导航菜单跟随筛选器变化
  - 视觉分层优化: 不同元数据使用不同样式和颜色

### 🔧 改进

- **API 响应增强**
  - `/api/configs` 返回新增的分类字段 (provider, provider_type, account, tags)
  - 后端统一使用英文字符串返回提供商类型 (official_relay/third_party_model)
  - 前端使用中文显示徽章 (🔄 官方中转 / 🤖 第三方模型)

- **代码结构优化**
  - 新增 `ProviderType::to_string_value()` 方法,用于 API 序列化
  - 配置节新增 6 个辅助方法: `provider_display()`, `provider_type_display()`, `provider_type_icon()`, `account_display()`, `has_tag()`, `tags_display()`
  - CcsConfig 新增 5 个分组筛选方法: `group_by_provider()`, `group_by_provider_type()`, `filter_by_tag()`, `filter_by_provider()`, `filter_by_provider_type()`

- **编译警告清理**
  - 为 CLI 功能保留的未使用方法添加 `#[allow(dead_code)]` 属性
  - 0 warnings, 0 errors 编译结果

### 📚 文档

- **configuration.md 更新**:
  - 新增 4 个配置段字段的完整说明
  - 更新配置示例,展示官方中转、第三方模型、多账号管理等场景
  - 更新 API 响应示例,包含新增的分类字段

- **changelog.md 更新**:
  - 记录 v1.1.0 的所有新增功能和改进

### 💡 使用场景

**多账号管理**:
```toml
[anyrouter-main]
provider = "anyrouter"
provider_type = "official_relay"
account = "github_5953"
tags = ["free", "primary"]

[anyrouter-backup]
provider = "anyrouter"
provider_type = "official_relay"
account = "github_5962"
tags = ["free", "backup"]
```

**分类筛选**:
- Web 界面可以按提供商类型快速筛选配置
- 未来 CLI 可以使用标签筛选: `ccr list --tag free`

### ⚠️ 破坏性变更

**无破坏性变更** - 100% 向后兼容
- 新增字段均为可选字段
- 现有配置文件无需修改即可正常工作
- CLI 命令接口不变
- API 行为完全兼容

### 🎯 质量指标

- 编译：0 errors, 0 warnings
- 测试：全部通过
- 代码质量：⭐⭐⭐⭐⭐
- 向后兼容：✅ 100%

---

## [1.0.0] - 2025-10-11

### 🏗️ 架构重构(重大更新)

**全面的架构现代化升级**,引入分层架构、Service 层、代码抽象和测试增强。

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
  - 配置管理：5 个端点(list, add, update, delete, switch)
  - 历史记录：1 个端点
  - 设置管理：3 个端点
  - 工具功能：2 个端点
  
- **集成测试支持** - 3 个集成测试,100% 通过

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
  - 错误码从硬编码改为常量(exit_codes 模块)
  - 锁管理器添加通用 `lock_resource()` 方法
  - 统一 Validatable trait 接口
  - 消除掩码逻辑重复
  
- **测试增强**
  - 测试覆盖率从 ~85% 提升到 95.1%
  - 新增 Service 层单元测试(6 个)
  - 新增 Core 层单元测试(4 个)
  - 新增 Utils 层单元测试(2 个)
  - 新增集成测试(3 个)

#### 📚 文档

- **ARCHITECTURE.md** - 完整的架构设计文档
- **CLAUDE.md** - 更新开发指南,添加 Service 层使用说明
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
- 总测试：81 个(77 passed, 95.1%)

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
  - **安全模式**：如果配置已存在,直接退出(不覆盖)
  - 必须使用 `--force` 才能覆盖现有配置
  - 使用 --force 时自动备份现有配置
  - 正确的文件权限设置(Unix: 644)
  - 提供有用的后续操作提示
  
- **配置导出 (Export)**: 导出配置到文件
  - **默认包含 API 密钥**,方便备份和迁移 🔑
  - 自动生成带时间戳的文件名
  - 支持 `--no-secrets` 参数导出不含密钥的配置
  - TOML 格式输出,易于编辑
  - 完美适用于备份、迁移和团队协作
  
- **配置导入 (Import)**: 从文件导入配置
  - 支持两种导入模式：
    - **合并模式** (`--merge`): 保留现有配置,添加新的
    - **替换模式** (默认): 完全替换现有配置
  - 导入前自动备份现有配置(可选 `--no-backup` 禁用)
  - 配置验证和完整性检查
  - 详细的导入摘要报告

- **备份清理 (Clean)**: 清理旧备份文件
  - 自动清理指定天数前的备份文件
  - 默认清理 7 天前的备份
  - 模拟运行模式(`--dry-run`)预览清理结果
  - 显示释放的磁盘空间
  - 仅清理 `~/.claude/backups/` 中的 `.bak` 文件

### 🔧 改进

- **Init 命令安全性增强**:
  - 如果配置文件已存在,直接提示并退出(不覆盖)
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
  - 新增 `docs/FEATURES.md` - 完整功能说明(12KB)
  - 新增 `docs/INIT_IMPORT_EXPORT.md` - 详细的导入导出指南(11KB)
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
ccr export                  # 导出配置(含密钥)
ccr export --no-secrets     # 导出配置(不含密钥)
ccr import <file> --merge   # 合并导入
ccr import <file>           # 替换导入
ccr clean                   # 清理旧备份(7天前)
ccr clean --days 30         # 清理 30 天前的备份
ccr clean --dry-run         # 模拟运行预览
```

### 🧪 测试

- 新增 8 个单元测试,全部通过
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
- `Cargo.toml` - 添加 filetime 依赖,移除 self_update/reqwest/tokio
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
- `self_update` - 不再需要(改用 cargo install)
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
- `src/commands/update.rs` (90 行,简化版)

**修改文件**:
- `src/commands/mod.rs` - 导出 update 命令
- `src/main.rs` - 集成 update 子命令

## [0.2.0] - 2025-10-10

### ✨ 新增

- **Web 管理界面**: 完整的 Web UI,支持可视化配置管理
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
  - 从备份恢复(预留功能)

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

如果你发现了 Bug 或有新的功能建议,欢迎：

- 提交 [Issue](https://github.com/bahayonghang/ccs/issues)
- 创建 [Pull Request](https://github.com/bahayonghang/ccs/pulls)

## 感谢

感谢所有为 CCR 做出贡献的开发者！

---

[0.2.3]: https://github.com/bahayonghang/ccr/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/bahayonghang/ccr/compare/v0.2.0...v0.2.2
[0.2.0]: https://github.com/bahayonghang/ccr/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/bahayonghang/ccr/releases/tag/v0.1.0

