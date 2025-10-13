# 文档更新日志

## 2025-10-13 - v1.1.2 跨平台打包指南

### 🆕 新增内容

#### 1. 跨平台打包指南
- **新建**: `guide/packaging.md` - 完整的跨平台打包文档
  - 📦 智能打包系统概述 (Linux/macOS/Windows)
  - 🎯 一键智能打包命令
  - 🐧 Linux 打包详解 (.deb + .rpm)
  - 🍎 macOS 打包详解 (.app + .dmg)
  - 🪟 Windows 打包详解 (.msi + .nsis)
  - 🔧 高级配置和自定义打包格式
  - 🌍 多平台构建策略 (本地/CI/CD/交叉编译)
  - 📊 性能数据和优化效果
  - 🐛 常见问题解答
  - ✅ 测试清单和发布流程

### ✏️ 更新内容

#### 1. README.md
- **更新文档列表**: 添加"跨平台打包指南"条目
- **更新目录结构**: 标注 `packaging.md` 新增文档

---

## 2025-10-13 - v1.1.2 功能增强 (之前)

### 🆕 新增内容

#### 1. Web 调试模式文档
- **新建**: `guide/web-debug-mode.md` - Web 调试模式完整指南
  - 双模式运行对比 (桌面模式 vs Web 调试模式)
  - 快速开始和服务管理
  - 系统架构和 API 适配器设计
  - 4 个主要使用场景 (WSL、远程开发、前端调试、API 测试)
  - 故障排查指南
  - 最佳实践和安全注意事项

#### 2. WSL 环境故障排查文档
- **新建**: `troubleshooting/wsl-issues.md` - WSL 专项故障排查
  - WSLg 架构说明
  - 6 个常见问题的详细解决方案:
    1. 鼠标滚轮无法滚动 (三层修复方案)
    2. libEGL 和 Mesa 警告
    3. 窗口无法显示或黑屏
    4. 应用启动慢
    5. 字体显示模糊
    6. 无法访问配置目录
  - WSL 开发环境最佳实践
  - WSL1 vs WSL2 对比

### ✏️ 更新内容

#### 1. 首页 (`index.md`)
- **新增特性**:
  - 🌐 Web 调试模式
  - 🌍 跨平台 - 特别标注 WSL 环境优化
- **更新主要功能**:
  - 双模式运行 (桌面 + Web)
  - WSL 优化
- **更新快速体验**:
  - 添加桌面模式和 Web 调试模式的代码示例
  - 新增 `just dev-wsl` 命令

#### 2. 快速开始指南 (`guide/getting-started.md`)
- **新增运行模式**:
  - 桌面模式 (推荐)
  - WSL 优化模式 (`just dev-wsl`)
  - Web 调试模式 (`just dev-web`)
- **新增提示框**:
  - WSL 环境提示 (滚轮修复说明)
  - Web 调试模式提示 (使用场景和命令)
- **新增故障排查章节**:
  - WSL 鼠标滚轮问题 (完整解决方案)
  - libEGL 和 Mesa 警告
  - Web 调试模式问题
  - 通用问题

#### 3. 开发指南 (`development/getting-started.md`)
- **更新开发模式**:
  - 三种模式的代码示例 (桌面、WSL、Web)
  - Web 调试模式的详细说明
- **新增 Web 调试模式优势**:
  - 技术实现说明
  - API 适配器架构

#### 4. 架构设计 (`architecture/overview.md`)
- **新增双模式运行章节**:
  - 桌面模式 vs Web 调试模式
  - API 适配器自动识别机制
- **更新前端层**:
  - 新增 `api/index.ts` - 双模式 API 适配
  - 新增 `scroll-fix.ts` - WSL 滚轮修复 polyfill
  - 更新 `style.css` - 包含 WSL 滚轮修复
- **新增 API 适配器章节**:
  - 代码示例和设计优点
- **新增 WSL 滚轮修复章节**:
  - 三层修复方案 (环境变量、CSS、JavaScript)
  - 详细代码示例
- **更新 Justfile 任务系统**:
  - 新增开发命令 (`dev-wsl`, `dev-web`)
  - 新增 Web 调试命令 (5 个新命令)

#### 5. VitePress 配置 (`.vitepress/config.ts`)
- **更新侧边栏**:
  - 快速开始 > 新增 "Web 调试模式"
  - 故障排查 > 新增 "WSL 环境问题"

### 🔑 关键技术点

#### 1. 双模式运行架构
```typescript
// API 适配器自动识别运行环境
const isTauriEnvironment = window.__TAURI__ !== undefined
if (isTauriEnvironment) {
  // 桌面模式: Tauri IPC
  result = await invoke('command')
} else {
  // Web 模式: HTTP API
  result = await fetch('http://localhost:3030/api/...').then(r => r.json())
}
```

#### 2. WSL 滚轮修复三层方案
1. **环境变量层**: `GDK_CORE_DEVICE_EVENTS=1`, `MOZ_USE_XINPUT2=1`
2. **CSS 层**: `-webkit-overflow-scrolling: touch`, `touch-action: pan-y`
3. **JavaScript 层**: 手动滚轮事件处理 polyfill

#### 3. Web 调试模式端点
- 前端: `http://localhost:5173` (Vite)
- 后端: `http://localhost:3030` (CCR Web API)

### 📋 新增 Justfile 命令

```bash
# WSL 优化模式
just dev-wsl              # WSL 环境优化启动

# Web 调试模式
just dev-web              # 启动 Web 调试模式
just stop-web             # 停止 Web 服务
just web-status           # 查看服务状态
just web-logs             # 查看日志
just web-logs-follow      # 实时跟踪日志
```

### 📊 文档统计

- **新建文档**: 2 个
- **更新文档**: 5 个
- **更新配置**: 1 个
- **新增章节**: 10+ 个
- **新增代码示例**: 20+ 个
- **新增命令**: 6 个

### 🎯 改进亮点

1. **全面的 WSL 支持**: 从环境设置到故障排查的完整覆盖
2. **灵活的运行模式**: 桌面、WSL 优化、Web 调试三种模式满足不同需求
3. **详细的故障排查**: 6 个 WSL 常见问题的完整解决方案
4. **技术深度**: 三层滚轮修复、API 适配器、双模式架构的详细说明
5. **实用的示例**: 大量代码示例和实际使用场景
6. **清晰的导航**: VitePress 侧边栏结构优化

### 📝 相关文件

#### 项目实现文件
- `ccr-tauri/dev-wsl.sh` - WSL 优化启动脚本
- `ccr-tauri/src-ui/src/api/index.ts` - API 适配器
- `ccr-tauri/src-ui/src/scroll-fix.ts` - 滚轮修复 polyfill
- `ccr-tauri/src-ui/src/style.css` - 全局 CSS 滚轮修复
- `ccr-tauri/src-ui/src/App.vue` - 组件级滚轮样式
- `ccr-tauri/tauri.conf.json` - CSP 配置
- `ccr-tauri/justfile` - 任务定义

#### 文档文件
- `docs_ccr-tauri/index.md`
- `docs_ccr-tauri/guide/getting-started.md`
- `docs_ccr-tauri/guide/web-debug-mode.md` (新建)
- `docs_ccr-tauri/development/getting-started.md`
- `docs_ccr-tauri/architecture/overview.md`
- `docs_ccr-tauri/troubleshooting/wsl-issues.md` (新建)
- `docs_ccr-tauri/.vitepress/config.ts`

### 🚀 下一步

用户可以：
1. 访问更新后的文档站点，查看新增内容
2. 使用 `just dev-wsl` 在 WSL 环境中运行
3. 使用 `just dev-web` 尝试 Web 调试模式
4. 遇到 WSL 问题时查阅故障排查文档

---

**Made with ❤️ by 哈雷酱**

哼，这次文档更新可是超级详细呢！从架构设计到故障排查，从快速开始到最佳实践，本小姐都给你整理得明明白白！(￣▽￣)／

