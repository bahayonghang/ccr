# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ✨ 新功能

- **TUI 鼠标支持**：
  - 新增 `SelectAt(usize)` 动作，支持鼠标点击直接选中列表项
  - 实现 `list_hit_test` / `tab_hit_test` 命中检测辅助函数（纯函数，可测试）
  - 主应用与 CodexAuth 子应用均支持鼠标左键点击交互
  - 支持鼠标点击标签栏（Tab bar）切换平台
  - 使用 `Cell<Option<Rect>>` 缓存布局区域以提升命中精度

- **Codex Auth 导入增强**：
  - `ccr codex auth import` 新增 `--force` 选项
  - 支持在合并模式下强制覆盖已存在的账号
  - 导入结果显示被覆盖账号的详细列表
  - 添加文件权限检查，防止覆盖只读文件
  - 新增 5 个测试用例覆盖强制导入功能

### 🔧 改进

- **后端架构优化**：
  - `config` / `stats` handler 移除对 executor 的直接依赖，改用 Service 层
  - `sync_status` 实现真实平台状态查询并补全全部平台列表

- **前端交互升级**：
  - 新增智能路由过渡动画系统，提升页面切换体验（基于 CSS transition + Vue router hooks）

### 📦 依赖更新

- `toml`、`uuid`、`sysinfo`、`clap` 等核心依赖升级至最新版本

---

## [4.1.1] - 2026-02-18

### ✨ 新功能

- **CLI 版本检测**：
  - 新增后端 CLI 版本检测 API（`GET /api/cli/version`）
  - 前端首页和 Codex 视图集成 CLI 版本检测与展示
  - 支持自动检测本地安装的 CCR 版本信息

- **市场模块完善**：
  - MarketView 全面重构，新增安装弹窗、卸载功能和统计面板
  - 完善安装状态与卸载状态的完整生命周期管理
  - `marketplace` handler 提取辅助函数，提升代码可维护性

- **国际化（i18n）扩展**：
  - 添加市场模块完整国际化文案（中英文）
  - 添加 CLI 版本相关国际化文案

### 🔧 改进

- **前端样式统一**：
  - 各平台视图统一使用 `AnimatedBackground` 组件，强化视觉一致性
  - 优化暗色模式背景效果
  - 全局背景控制重构：移入 `MainLayout`，支持路由级 `hideGlobalBackground` 配置
  - `BaseSlashCommands` 布局与组件主题变量统一重构

- **构建工具**：
  - justfile 新增完整 CI 流程命令集（format、clippy、test、build 一键运行）

### 🐛 修复

- **ccr-types**：修复 Hooks 配置反序列化兼容性问题，新增 MCP URL 模式支持

---

## [4.1.0] - 2026-02-15

### ✨ 新功能

- **Codex 完整设置管理**：
  - 新增 Codex 设置读取与写入 API（`GET/PUT /api/codex/settings`）
  - 前端 Codex Settings 配置页完整实现
  - 支持 API Key、模型选择、Provider 端点等核心参数的可视化配置

---

## [4.0.9] - 2026-02-14

### ✨ 新功能

- **Claude Code 全局设置管理**：
  - 新增 Claude Code 全局 `settings.json` 读取与写入 API
  - 前端 Claude Settings 配置页完整实现
  - 支持全局设置的可视化编辑与保存

---

## [4.0.8] - 2026-02-14

### 🔧 改进

- **历史记录存储迁移**：
  - 将历史记录存储从 JSON 文件全面迁移至 SQLite 数据库
  - 提升历史查询性能，降低大量历史记录时的内存占用
  - 支持更复杂的历史记录过滤与统计分析

---

## [4.0.4] - 2026-02-14

### ✨ 新功能

- **使用量分析 V2（Usage Dashboard V2）**：
  - 全新 Usage Dashboard V2 页面，集成 ApexCharts 可视化图表
  - 添加 Dashboard V2 国际化文案与面包屑动态导航
  - 新增 Usage V2 聚合 API，支持按日/周/月维度统计

### ⚡ 性能优化

- **后端**：新增 `usage_daily_agg` 预聚合表，大幅提升使用量查询性能
- **前端**：`StatsView` 串行请求改为 `Promise.all` 并行请求，加速页面加载

### 🔧 改进

- **API 清理与重构**：
  - 移除旧 V1 `usage` / `records` 端点及相关前端调用
  - 移除 stats 快捷端点，逻辑内联到 `stats_summary`
  - 清理已废弃的前端 `UsageView` 组件

---

## [4.0.3] - 2026-02-14

### ⚡ 性能优化

- **后端**：
  - 优化签到模块架构与性能，减少不必要的数据库操作
  - 技能模块添加服务端缓存，降低重复 I/O 开销

- **前端**：
  - 签到前端状态管理重构，减少不必要的组件重渲染
  - 技能模块前端性能优化
  - `AddSkillView` 安装状态计算逻辑优化

---

## [4.0.2] - 2026-02-13

### ✨ 新功能

- **跨平台统一技能管理中心**：
  - 新增统一技能管理 API，支持 Claude / Codex / Gemini 等多平台
  - 前端新增技能管理中心，支持跨平台技能浏览与一键安装
  - 新增技能多源导入 API（GitHub、本地、市场）与市场增强功能
  - 新增技能添加页面（`AddSkillView`），支持从市场直接安装

- **签到功能增强（CDK 与 OAuth）**：
  - 后端扩展签到支持 CDK 充值码与 Cloudflare WAF 绕过
  - 前端新增签到 OAuth 向导与 CDK 配置界面

---

## [4.0.1] - 2026-02-13

### 🔧 改进

- **代码质量**：
  - 清理死代码和冗余 `#[allow(dead_code)]` 属性标注
  - 消除所有 `unwrap()` / `expect()` 调用，符合严格 Clippy 检查（`-W clippy::unwrap_used`）

---

## [4.0.0] - 2026-02-11

> **重大版本发布** — v4.x 系列正式启动。

此版本将 v3.x 开发分支合并至 main，包含 v3.x 系列所有成熟特性，并作为 v4.x 主线的起始版本对外发布。详细功能参见 [3.19.0] 及更早版本记录。

---

## [3.19.0] - 2026-01-10

### ✨ 新功能

- **Codex 多账号管理**：
  - 新增 `ccr codex auth` 命令组，支持多账号切换
  - `ccr codex auth save <名称>` - 保存当前登录到命名账号
  - `ccr codex auth list` - 列出所有已保存的账号
  - `ccr codex auth switch <名称>` - 切换到指定账号
  - `ccr codex auth delete <名称>` - 删除指定账号
  - `ccr codex auth current` - 显示当前账号信息
  - `ccr codex` - 启动交互式 TUI 账号管理界面
- **Token 新鲜度指示**：
  - 自动解析 `last_refresh` 时间戳
  - 显示 Token 状态：🟢 新鲜 (<1天)、🟡 陈旧 (1-7天)、🔴 过期 (>7天)
- **进程检测警告**：
  - 切换账号前检测运行中的 Codex 进程
  - 显示警告提示用户关闭进程后再切换
- **安全特性**：
  - 邮箱脱敏显示 (如 `use***@example.com`)
  - Unix 系统下 auth 文件权限设置为 0600
  - 备份轮转机制，保留最近 10 个备份

### 🐛 修复

- **TUI 交互优化**：修复 `ccr codex` TUI 切换账号后未自动退出的问题

### 🧪 测试

- 新增 26 个单元测试覆盖 Codex Auth 服务层
- 测试覆盖：邮箱脱敏、账号名验证、Token 新鲜度计算、JWT 解析、工作流集成

---

## [3.18.0] - 2026-01-10

### ✨ 新功能

- **Droid 平台完整支持**：
  - 添加 Factory Droid 平台核心支持
  - 实现后端 Droid 管理器和 API 端点
  - 添加前端 Custom Models、Profiles 和 Droids 管理界面
  - 添加 Droid 中英文国际化支持
- **Tauri 桌面应用增强**：
  - 添加应用退出确认功能（原生对话框）
  - 侧边栏新增退出确认开关设置
  - 更新应用图标并添加生成脚本

### 🔧 改进

- **签到功能优化**：
  - 签到失败时显示警告样式和失败账号详情
  - 修复下拉菜单改为向上弹出避免被遮挡
  - 点击菜单项后自动关闭菜单
- **代码重构**：
  - 移除废弃的 migrate 命令及相关代码
  - 添加 AI Agent 工作流与代码规范规则

### 🐛 修复

- **CI 构建修复**：修复 `--no-default-features` 构建时的 unused 警告

### 📦 依赖更新

- blake3: 1.5.4 → 1.8.3
- indexmap: 2.12.1 → 2.13.0
- whoami: 2.0.0 → 2.0.2
- serde_json: 1.0.148 → 1.0.149

### 🔗 相关资源

- **GitHub Release**：[v3.18.0](https://github.com/bahayonghang/ccr/releases/tag/v3.18.0)
- **完整变更**：[v3.17.3...v3.18.0](https://github.com/bahayonghang/ccr/compare/v3.17.3...v3.18.0)

---

## [3.17.3] - 2026-01-05

### ✨ 新功能

- **UI/UX 全面升级**：
  - **Codex 增强**：重构 Codex Profiles 视图，采用 Neo-Terminal 风格，优化布局与交互体验
  - **组件库扩展**：新增 `Badge`、`BaseModal`、`Hooks`、`OutputStyles`、`Statusline` 等视图及详情页
  - **无障碍性 (A11y)**：全面增强模态框、按钮与导航的可访问性，支持屏幕阅读器与键盘导航
  - **视觉优化**：统一前端组件代码格式，扩展 Tailwind 设计令牌，优化 Web 界面视觉效果
- **核心功能扩展**：
  - **资源市场**：实现资源市场 (Marketplace) 后端与前端集成
  - **全栈配置增强**：新增 Hooks、OutputStyles 和 Statusline 的全栈支持 (API + Client + UI)
  - **多平台管理**：增强后端多平台配置管理能力，支持 Windows 环境变量
- **系统集成**：
  - **Tauri 深度集成**：集成后端 Sidecar 自动启动与生命周期管理
  - **健康检查**：新增后端健康检查与状态指示组件

### 🔧 改进

- **CI/CD 构建**：
  - 新增 Tauri 桌面应用多平台构建流程
  - 集成 `bun` 运行时，修复构建依赖问题
- **开发体验**：
  - 新增 `benchmark` 性能测试脚本
  - 优化开发脚本 (clean/run) 与进程管理
- **文档体系**：
  - 新增架构文档、API 参考、Composables 参考及迁移指南
  - 简化 Codex 配置示例文件

### 🐛 修复

- **平台兼容性**：修复 Windows 下环境变量获取与路径处理问题
- **构建修复**：解决 `--no-default-features` 构建错误及 Tauri 依赖缺失
- **代码质量**：修复 Rust Clippy 警告与前端组件硬编码颜色

### 📊 统计数据

- **版本跨度**：覆盖 v3.16.1 至 v3.17.3 的所有变更
- **主要重点**：前端架构重构、A11y 支持、Tauri 集成完善

### 🔗 相关资源

- **完整变更**：[v3.16.0...v3.17.3](https://github.com/bahayonghang/ccr/compare/v3.16.0...v3.17.3)

---

## [3.16.0] - 2025-12-26

### 🔧 核心重构

- **CLI 架构重构**：重构命令分发逻辑（`dispatch.rs`），提升代码可维护性和扩展性
- **原子写入模块**：新增 `atomic_writer` 和 `fileio` 模块，提供原子文件写入能力，确保数据一致性
- **管理器层扩展**：新增 `history`、`settings`、`temp_override` 管理器，完善数据管理架构
- **服务层增强**：新增 `backup_service`、`history_service`、`settings_service` 服务，提升业务逻辑封装
- **命令优化**：优化 `skills_cmd` 和 `provider_cmd` 命令实现，改进用户体验

### 🎨 UI/UX 优化

- **UI 命令改进**：优化 UI 命令定义与帮助信息，提升命令行交互体验
- **前端布局优化**：优化主布局与 WebSocket 组件，提升界面响应性
- **开发环境优化**：优化开发环境脚本配置，提升开发效率

### ✨ 新功能

- **签到仪表盘增强**：新增账户仪表盘数据展示功能，提供更直观的数据分析视图

### 📊 统计数据

- **代码变更**：新增 2,570 行，删除 957 行，净增 1,613 行
- **文件变更**：68 个文件
- **提交数量**：9 commits

### 🔗 相关资源

- **GitHub Release**：[v3.16.0](https://github.com/bahayonghang/ccr/releases/tag/v3.16.0)
- **对比 Diff**：[v3.15.0...v3.16.0](https://github.com/bahayonghang/ccr/compare/v3.15.0...v3.16.0)

---

## [3.15.0] - 2025-12-25

### ✨ 新增

- **通用配置缓存体系**：新增 `ConfigCache` 支持 TTL 过期与自动缓存的 `CachedSettingsManager`，减少重复 I/O，显著提升配置读取性能
- **平台基础抽象与统一响应**：新增 `platforms/base` 抽象层与 `response` 模块，平台 Handler 统一响应格式，并以宏简化 Manager 初始化与错误处理
- **签到功能扩展与 WAF 绕过**：内置中转站提供商、账号总额度/消耗字段与 Dashboard 分析视图，加入 WAF 绕过与日志管理基础模块及配套 UI
- **平台前端通用组件**：新增通用 `PlatformMcpView`、`PlatformPluginsView`、`ToastContainer`，引入组合式 `usePlatformMcp` / `usePlatformPlugins`，API 模块化提升复用
- **性能与工具**：新增后端性能测试脚本（`benchmark.ps1`/`benchmark.sh`），更新文档补充缓存层说明

### 🔧 改进

- **配置与平台重构**：配置文件模块化为 `config/` 目录，删除重复的配置转换与保存逻辑；Claude/Codex/Gemini 等平台基于统一抽象重构
- **交互与样式**：签到页面统计卡片、表格与 Dashboard 视觉优化，SVG 图标格式化提升可读性；通用组件与签到界面样式细节打磨
- **依赖与版本管理**：逐步升级版本至 v3.15.0，期间补齐 v3.14.1~v3.14.5 跨平台脚本与依赖优化

### 📊 统计数据

- **代码变更**：新增 19,163 行，删除 5,155 行，净增 14,008 行

### 🔗 相关资源

- **GitHub Release**：[v3.15.0](https://github.com/bahayonghang/ccr/releases/tag/v3.15.0)
- **详细文档**：详见 [docs/reference/changelog.md](docs/reference/changelog.md)
- **对比 Diff**：[3.15.0]

## [3.14.0] - 2025-12-22

### ✨ 新增

#### 🎫 API 中转站自动签到功能（核心特性）
- **完整的签到系统**：支持手动一键签到和自动定时签到
- **提供商管理**：管理多个 API 中转站提供商配置
- **账号管理**：支持多账号批量管理，每个账号独立配置
- **签到记录**：完整的签到历史记录，支持成功/失败状态追踪
- **余额快照**：自动记录余额变化，支持历史趋势查看
- **数据导入/导出**：JSON 格式配置导入导出，方便备份和迁移
- **数据保留策略**：90 天自动数据清理，避免历史数据过度积累
- **安全加密存储**：AES-256-GCM 加密存储 API Key，保障账号安全

**技术实现**：
- 后端：新增 129 个 REST API 端点，完整的签到服务架构
- 前端：Vue.js 3 签到管理界面，直观的操作体验
- 数据层：6 个管理器（Provider/Account/Record/Balance/Export/Crypto）
- 代码量：6200+ 行后端代码，180+ 行前端代码

#### 🎨 光明主题"光韵流光"动画系统
- **8 种核心动画关键帧**：lightPulse、lightFlow、lightRipple、lightGlow 等
- **光明主题专用 CSS 变量**：完整的光感配色系统
- **玻璃态组件增强**：卡片、按钮、导航、输入框的流畅动画效果
- **动态背景优化**：轻盈优雅的光感体验
- **主题对比**：与暗黑主题"赛博霜夜"形成完美美学对比

**技术实现**：
- 855 行 CSS 代码
- 响应式动画效果
- 无性能损失的流畅体验

### 🔧 改进

- **样式集成优化**：集成签到功能导航和界面样式，提升视觉一致性
- **组件样式适配**：适配新动画系统的组件样式优化
- **CI 流程改进**：添加自动格式化步骤，确保代码质量
- **代码重构**：消除 `unwrap_or_default` 模式并添加调试日志
- **构建优化**：改进 justfile 跨平台支持，提升开发体验

### 🐛 修复

- **CI 构建修复**：修复 CI 构建检查错误，确保持续集成稳定性

### 📊 统计数据

- **后端新增代码**：6200+ 行（签到功能）
- **前端新增代码**：855+ 行（动画系统 + 签到界面）
- **新增文件**：29 个（签到功能模块）
- **API 端点**：新增 129 个 REST API
- **测试覆盖率**：97 个测试通过，0 警告

### 🔗 相关资源

- **详细文档**：详见 [docs/reference/changelog.md](docs/reference/changelog.md)
- **签到功能说明**：详见项目 README
- **GitHub Release**：[v3.14.0](https://github.com/bahayonghang/ccr/releases/tag/v3.14.0)

---

## [3.13.0] - 2025-12-22

*详见 [docs/reference/changelog.md](docs/reference/changelog.md)*

---

## [3.11.0] - 之前版本

*详见 [docs/reference/changelog.md](docs/reference/changelog.md)*

---

## 版本说明

- **主版本号 (Major)**: 不兼容的 API 变更
- **次版本号 (Minor)**: 向下兼容的功能性新增
- **修订号 (Patch)**: 向下兼容的问题修正

## 贡献

如果你发现了 Bug 或有新的功能建议，欢迎：

- 提交 [Issue](https://github.com/bahayonghang/ccr/issues)
- 创建 [Pull Request](https://github.com/bahayonghang/ccr/pulls)


---

[4.1.1]: https://github.com/bahayonghang/ccr/compare/v4.1.0...v4.1.1
[4.1.0]: https://github.com/bahayonghang/ccr/compare/v4.0.9...v4.1.0
[4.0.9]: https://github.com/bahayonghang/ccr/compare/v4.0.8...v4.0.9
[4.0.8]: https://github.com/bahayonghang/ccr/compare/v4.0.7...v4.0.8
[4.0.4]: https://github.com/bahayonghang/ccr/compare/v4.0.3...v4.0.4
[4.0.3]: https://github.com/bahayonghang/ccr/compare/v4.0.2...v4.0.3
[4.0.2]: https://github.com/bahayonghang/ccr/compare/v4.0.1...v4.0.2
[4.0.1]: https://github.com/bahayonghang/ccr/compare/v4.0.0...v4.0.1
[4.0.0]: https://github.com/bahayonghang/ccr/releases/tag/v4.0.0
[3.19.0]: https://github.com/bahayonghang/ccr/compare/v3.18.0...v3.19.0
[3.18.0]: https://github.com/bahayonghang/ccr/compare/v3.17.3...v3.18.0
[3.17.3]: https://github.com/bahayonghang/ccr/compare/v3.16.0...v3.17.3
[3.16.0]: https://github.com/bahayonghang/ccr/compare/v3.15.0...v3.16.0
[3.15.0]: https://github.com/bahayonghang/ccr/compare/v3.14.0...v3.15.0
[3.14.0]: https://github.com/bahayonghang/ccr/compare/v3.13.0...v3.14.0
[3.13.0]: https://github.com/bahayonghang/ccr/releases/tag/v3.13.0
[3.11.0]: https://github.com/bahayonghang/ccr/releases/tag/v3.11.0
