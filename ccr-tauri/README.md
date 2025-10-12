# CCR Desktop - Tauri 桌面应用

本小姐用 Tauri 打造的 CCR 桌面应用！(￣▽￣)／

## 📋 项目简介

CCR Desktop 是 CCR (Claude Code Configuration Switcher) 的 Tauri 桌面版本，提供现代化的 GUI 界面来管理 Claude Code 配置。

### ✨ 特性

- 🎨 **现代化界面**: Vue 3 + TypeScript + Vite 构建的优雅 UI
- 🚀 **高性能**: Tauri 2.0 + Rust 后端，原生性能
- 🔄 **完整功能**: 复用 CCR 核心库，支持所有 CLI 功能
- 📦 **小体积**: 相比 Electron 更小的安装包
- 🔒 **安全**: Tauri 安全模型，文件系统权限控制
- 🌙 **深色模式**: 支持浅色/深色主题自动切换

### 🎯 核心功能

- 📋 配置列表展示和管理
- 🔄 一键切换配置
- ➕ 创建和编辑配置
- 📥📤 配置导入导出
- 💾 备份管理和恢复
- 📜 操作历史查看
- 🖥️ 系统信息展示

## 🚀 快速开始

### 📦 前置要求

- **Rust**: 1.70+ (已通过根项目安装)
- **Node.js**: 18+ 和 npm
- **系统依赖**:
  - macOS: Xcode Command Line Tools
  - Linux: `webkit2gtk-4.0`, `libgtk-3-dev`, `libayatana-appindicator3-dev`
  - Windows: WebView2 (Windows 10/11 自带)

### 🛠️ 开发环境设置

#### 1. 安装前端依赖

```bash
cd ccr-tauri/src-ui
npm install
```

#### 2. 开发模式运行

```bash
# 在 ccr-tauri 目录
cargo tauri dev

# 或者使用 npm scripts
cd src-ui
npm run tauri:dev
```

这将启动：
- Vite 开发服务器 (http://localhost:5173)
- Tauri 开发窗口 (热重载支持)

#### 3. 构建生产版本

```bash
# 在 ccr-tauri 目录
cargo tauri build

# 或者使用 npm scripts
cd src-ui
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`:
- **macOS**: `.app` 和 `.dmg`
- **Linux**: `.AppImage`, `.deb`
- **Windows**: `.msi`, `.exe`

## 📁 项目结构

```
ccr-tauri/
├── src/                    # Rust 后端
│   ├── main.rs            # Tauri 应用入口
│   ├── lib.rs             # 库入口
│   └── commands/          # Tauri Commands
│       └── mod.rs         # 命令定义和实现
├── src-ui/                # Vue 3 前端
│   ├── src/
│   │   ├── main.ts        # Vue 应用入口
│   │   ├── App.vue        # 主应用组件
│   │   ├── api/           # Tauri API 封装
│   │   ├── types/         # TypeScript 类型定义
│   │   ├── components/    # Vue 组件
│   │   └── stores/        # 状态管理 (Pinia)
│   ├── package.json       # 前端依赖
│   ├── vite.config.ts     # Vite 配置
│   └── tsconfig.json      # TypeScript 配置
├── icons/                 # 应用图标
├── Cargo.toml            # Rust 依赖
├── tauri.conf.json       # Tauri 配置
└── build.rs              # 构建脚本
```

## 🎨 技术栈

### 后端 (Rust)

- **Tauri 2.0**: 桌面应用框架
- **CCR 核心库**: 复用现有的配置管理逻辑
- **serde**: 序列化/反序列化
- **tokio**: 异步运行时

### 前端 (Vue 3)

- **Vue 3**: 渐进式 JavaScript 框架
- **TypeScript**: 类型安全
- **Vite**: 快速的开发构建工具
- **Pinia**: 状态管理 (可选)

## 🔧 配置说明

### Tauri 配置 (`tauri.conf.json`)

关键配置项：

```json
{
  "build": {
    "devUrl": "http://localhost:5173",  // Vite 开发服务器
    "frontendDist": "../src-ui/dist"    // 前端构建输出
  },
  "bundle": {
    "identifier": "com.ccr.desktop",    // 应用标识符
    "icon": [...]                        // 图标文件
  },
  "plugins": {
    "fs": {
      "scope": [                         // 文件系统访问权限
        "$HOME/.ccs_config.toml",
        "$HOME/.claude/settings.json",
        ...
      ]
    }
  }
}
```

### 安全策略

- 文件系统访问限制在必要的配置文件路径
- CSP (Content Security Policy) 配置
- 所有 Tauri Commands 都经过权限验证

## 📚 开发指南

### 添加新 Command

1. 在 `src/commands/mod.rs` 中定义命令函数：

```rust
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    // 调用服务层逻辑
    let service = ConfigService::default().map_err(|e| e.to_string())?;
    // ...
    Ok("Success".to_string())
}
```

2. 在 `src/main.rs` 中注册命令：

```rust
.invoke_handler(tauri::generate_handler![
    commands::my_command,  // 添加新命令
    // ... 其他命令
])
```

3. 在前端 `src-ui/src/api/index.ts` 中封装 API：

```typescript
export async function myCommand(param: string): Promise<string> {
  return await invoke('my_command', { param })
}
```

### 调试技巧

#### 后端调试

```bash
# 启用 Rust 日志
export RUST_LOG=ccr_tauri=debug,ccr=debug
cargo tauri dev
```

#### 前端调试

- 开发模式下自动启用 Vue DevTools
- Tauri DevTools (F12) 查看控制台和网络请求

## 🚀 发布流程

### 1. 版本号更新

同步更新以下文件中的版本号：
- `Cargo.toml`
- `src-ui/package.json`
- `tauri.conf.json`
- 根项目 `Cargo.toml` (workspace)

### 2. 构建发布版本

```bash
cd ccr-tauri
cargo tauri build --release
```

### 3. 代码签名 (macOS/Windows)

参考 Tauri 文档配置代码签名：
- macOS: Apple Developer ID
- Windows: Code Signing Certificate

### 4. 发布到 GitHub Releases

使用 `tauri-action` GitHub Action 自动化构建和发布。

## 🐛 故障排查

### 常见问题

**Q: Tauri 构建失败，提示找不到 webkit2gtk**
A: Linux 系统需安装 `webkit2gtk-4.0-dev`:
```bash
sudo apt install libwebkit2gtk-4.0-dev
```

**Q: 前端无法调用后端 Command**
A: 检查：
1. Command 是否在 `main.rs` 中注册
2. 参数类型是否匹配
3. 查看浏览器控制台错误信息

**Q: 开发模式窗口启动很慢**
A: 正常现象，首次编译 Rust 代码需要时间。后续启动会更快。

## 📄 许可证

MIT License - 与 CCR 主项目相同

## 🙏 致谢

- **CCR 核心库**: 复用完整的配置管理逻辑
- **Tauri**: 提供优秀的 Rust + Web 桌面应用框架
- **Vue 3**: 现代化的前端框架

---

**Made with ❤️ by 哈雷酱 | CCR Tauri Desktop v1.1.2**

哼，这可是本小姐精心打造的桌面应用呢！(￣▽￣)／
