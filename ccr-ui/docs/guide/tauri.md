# 🖥️ Tauri 桌面应用

CCR UI 提供了基于 Tauri 2.x 的原生桌面应用程序，为用户提供更流畅的使用体验和更好的系统集成。

## 什么是 Tauri？

Tauri 是一个使用 Web 技术构建轻量级、安全且跨平台桌面应用的框架。CCR UI 的桌面版本结合了：

- **前端**: Vue 3 + Vite（与 Web 版本共享代码）
- **后端**: Rust + Tauri API（系统级交互）
- **打包**: 原生安装包（.dmg, .exe, .deb, .AppImage）

### 🌟 主要优势

- ⚡ **轻量级**: 相比 Electron，体积小 80%+，内存占用低 50%+
- 🔒 **安全**: Rust 底层保证内存安全，严格的权限控制
- 🚀 **性能**: 原生 WebView，启动速度快，运行流畅
- 📦 **原生体验**: 系统托盘、通知、文件操作等原生功能
- 🌍 **跨平台**: 一套代码，支持 Windows、macOS、Linux

## 系统要求

### macOS

- **最低系统版本**: macOS 10.13 (High Sierra) 或更高
- **开发依赖**:
  - Xcode Command Line Tools: `xcode-select --install`
  - Rust 1.70+
  - Node.js 18+

### Windows

- **最低系统版本**: Windows 7 或更高（推荐 Windows 10/11）
- **开发依赖**:
  - Microsoft C++ Build Tools
  - WebView2 Runtime（Windows 11 自带）
  - Rust 1.70+
  - Node.js 18+

### Linux

- **支持的发行版**: Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch Linux
- **开发依赖**:
  ```bash
  # Ubuntu/Debian
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

  # Fedora
  sudo dnf install webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel

  # Arch Linux
  sudo pacman -Syu
  sudo pacman -S webkit2gtk-4.1 \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    appmenu-gtk-module \
    libappindicator-gtk3 \
    librsvg
  ```

## 开发模式

### 🚀 快速启动（使用 Just - 推荐）

```bash
cd ccr-ui

# 启动 Tauri 开发模式
just tauri-dev

# 或者使用简化命令
cd frontend
npm run tauri:dev
```

开发模式会：
1. 启动 Vite 开发服务器（http://localhost:5173）
2. 自动编译 Rust 代码
3. 打开 Tauri 桌面窗口
4. 支持热重载（修改代码自动刷新）

### 📝 手动启动

```bash
cd ccr-ui/frontend

# 1. 安装依赖（首次运行）
npm install

# 2. 启动开发模式
npm run tauri:dev
```

### 🔍 开发工具

```bash
# 检查 Tauri 环境和配置
just tauri-check
# 或
npm run tauri:check

# 检查 Rust 代码
just tauri-check-rust
# 或
npm run tauri:check

# 运行 Clippy（Rust linter）
just tauri-clippy
# 或
npm run tauri:clippy

# 格式化 Rust 代码
just tauri-fmt
# 或
npm run tauri:fmt

# 运行测试
just tauri-test
# 或
npm run tauri:test
```

### 🐛 调试技巧

**前端调试**:
- 桌面窗口中右键 → "检查元素" → 打开 DevTools
- 或按 `F12` / `Cmd+Option+I` (macOS)

**后端调试**:
```bash
# 在 Rust 代码中添加调试输出
println!("Debug: {:?}", variable);

# 或使用 dbg! 宏
dbg!(&variable);

# 启动时会在终端显示输出
```

## 构建生产版本

### 🏗️ 标准构建

```bash
cd ccr-ui

# 使用 Just（推荐）
just tauri-build

# 或手动构建
cd frontend
npm run build:desktop
```

构建过程：
1. ✅ 构建 Vue 前端（生产优化）
2. ✅ 编译 Rust 代码（Release 模式）
3. ✅ 打包成原生安装包
4. ✅ 自动清理构建缓存（仅 macOS，避免弹窗）

### 📦 构建产物

构建完成后，安装包位于 `frontend/src-tauri/target/release/bundle/`:

**macOS**:
- `macos/CCR Desktop.app` - 应用程序包（可直接运行）
- `dmg/CCR Desktop_2.5.0_aarch64.dmg` - 磁盘镜像（可分发）

**Windows**:
- `msi/CCR Desktop_2.5.0_x64_en-US.msi` - MSI 安装程序
- `nsis/CCR Desktop_2.5.0_x64-setup.exe` - NSIS 安装程序（可选）

**Linux**:
- `deb/ccr-desktop_2.5.0_amd64.deb` - Debian/Ubuntu 包
- `appimage/ccr-desktop_2.5.0_amd64.AppImage` - 通用 Linux 包

### ⚡ 调试构建（更快，带调试符号）

```bash
# 调试构建（编译速度快 3-5 倍）
just tauri-build-debug
# 或
npm run tauri:build:debug
```

适用场景：
- 测试打包流程
- 快速验证功能
- 调试特定问题

⚠️ **注意**: 调试版本体积更大，性能较差，不适合分发。

## 平台特定说明

### 🍎 macOS

#### 代码签名

默认配置为**未签名**，适合个人使用。如需分发，需要配置代码签名：

```json
// frontend/src-tauri/tauri.conf.json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "providerShortName": "TEAM_ID",
      "entitlements": "path/to/entitlements.plist"
    }
  }
}
```

#### 公证（Notarization）

macOS 10.15+ 需要公证才能正常分发：

```bash
# 1. 构建应用
just tauri-build

# 2. 公证（需要 Apple Developer 账号）
xcrun notarytool submit \
  "frontend/src-tauri/target/release/bundle/dmg/CCR Desktop_2.5.0_aarch64.dmg" \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# 3. 装订公证票据
xcrun stapler staple \
  "frontend/src-tauri/target/release/bundle/dmg/CCR Desktop_2.5.0_aarch64.dmg"
```

#### 已知问题与解决方案

**问题**: 构建后弹出 DMG 安装窗口

**原因**: macOS 的 `bundle_dmg.sh` 脚本会自动挂载 DMG 进行验证。

**解决方案**: ✅ 已在 `ccr-ui/justfile:980` 添加自动卸载逻辑
```bash
@hdiutil detach "/Volumes/CCR Desktop" 2>/dev/null || true
```

现在使用 `just tauri-build` 不会再弹出安装窗口！

#### Universal Binary（通用二进制）

支持 Intel (x86_64) 和 Apple Silicon (aarch64) 的通用包：

```bash
# 需要安装两个 Rust target
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# 修改 tauri.conf.json
{
  "bundle": {
    "macOS": {
      "targets": ["universal"]
    }
  }
}

# 构建
just tauri-build
```

### 🪟 Windows

#### 安装程序选择

CCR UI 默认生成 MSI 安装程序，也可配置 NSIS:

```json
// frontend/src-tauri/tauri.conf.json
{
  "bundle": {
    "targets": ["msi", "nsis"],
    "windows": {
      "wix": {
        "language": ["zh-CN", "en-US"]
      }
    }
  }
}
```

#### WebView2 Runtime

Windows 7/8/10 需要预装 WebView2 Runtime：
- Windows 11 已内置
- 自动检查并提示安装
- 离线安装包: [Microsoft WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

#### 管理员权限

如需管理员权限，修改 `tauri.conf.json`:

```json
{
  "bundle": {
    "windows": {
      "allowDowngrades": true,
      "wix": {
        "installMode": "perMachine"
      }
    }
  }
}
```

### 🐧 Linux

#### 选择打包格式

推荐使用 `.deb` (Debian/Ubuntu) 或 `.AppImage` (通用):

```json
// frontend/src-tauri/tauri.conf.json
{
  "bundle": {
    "targets": ["deb", "appimage"],
    "linux": {
      "deb": {
        "depends": [
          "libwebkit2gtk-4.1-0",
          "libayatana-appindicator3-1"
        ]
      }
    }
  }
}
```

#### 安装

```bash
# Debian/Ubuntu
sudo dpkg -i ccr-desktop_2.5.0_amd64.deb

# AppImage（无需安装）
chmod +x ccr-desktop_2.5.0_amd64.AppImage
./ccr-desktop_2.5.0_amd64.AppImage

# 创建桌面快捷方式（AppImage）
./ccr-desktop_2.5.0_amd64.AppImage --appimage-extract
sudo cp squashfs-root/usr/share/applications/*.desktop /usr/share/applications/
```

## 配置说明

### Tauri 配置文件

**位置**: `ccr-ui/frontend/src-tauri/tauri.conf.json`

**核心配置**:

```json
{
  "productName": "CCR Desktop",
  "version": "2.5.0",
  "identifier": "com.ccr.desktop",

  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },

  "app": {
    "windows": [{
      "title": "CCR Desktop - Claude Code Configuration Manager",
      "width": 1200,
      "height": 800,
      "minWidth": 800,
      "minHeight": 600,
      "resizable": true,
      "center": true
    }]
  },

  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/icon.png"],
    "category": "DeveloperTool"
  }
}
```

### Cargo 配置

**位置**: `ccr-ui/frontend/src-tauri/Cargo.toml`

**关键依赖**:

```toml
[dependencies]
tauri = { version = "2.9", features = ["devtools"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
tauri-build = { version = "2.9", features = [] }
```

### 图标配置

Tauri 需要多种尺寸的图标:

```
frontend/src-tauri/icons/
├── 32x32.png          # Windows taskbar
├── 128x128.png        # macOS Dock
├── icon.png           # 通用图标 (512x512)
├── icon@2x.png        # Retina 显示
└── icon.icns          # macOS (自动生成)
```

生成图标:
```bash
cd frontend/src-tauri
npm run tauri icon path/to/original-icon.png
```

## 常见问题

### Q: 开发模式启动失败？

**A**: 检查依赖和环境:

```bash
# 1. 检查 Tauri 环境
cd frontend
npx tauri info

# 2. 检查 Node 依赖
npm install

# 3. 清理缓存重试
rm -rf node_modules dist src-tauri/target
npm install
just tauri-dev
```

### Q: 构建时 Rust 编译错误？

**A**: 常见原因：

1. **Rust 版本过低**:
   ```bash
   rustup update
   rustc --version  # 应该 >= 1.70
   ```

2. **缺少系统依赖** (Linux):
   ```bash
   # 参考上面 "系统要求 → Linux" 安装依赖
   ```

3. **Cargo.lock 冲突**:
   ```bash
   cd frontend/src-tauri
   rm Cargo.lock
   cargo build
   ```

### Q: 构建的应用无法打开？

**macOS**:
```bash
# 允许来自身份不明开发者的应用
xattr -cr "CCR Desktop.app"

# 或在系统设置中允许
```

**Windows**:
- 安装 WebView2 Runtime
- 关闭杀毒软件重试

**Linux**:
```bash
# AppImage 需要可执行权限
chmod +x ccr-desktop_*.AppImage
```

### Q: 如何减小应用体积？

**A**: 优化策略:

1. **启用 LTO** (Link-Time Optimization):
   ```toml
   # frontend/src-tauri/Cargo.toml
   [profile.release]
   lto = true
   opt-level = "z"  # 优化体积
   codegen-units = 1
   ```

2. **剥离调试符号**:
   ```toml
   [profile.release]
   strip = true
   ```

3. **减少依赖**:
   ```bash
   cargo tree  # 查看依赖树
   # 移除不必要的 features
   ```

实际效果:
- 未优化: ~15 MB
- 优化后: ~8 MB
- gzip 压缩: ~3 MB

### Q: 如何添加系统托盘？

**A**: 修改 `src-tauri/src/main.rs`:

```rust
use tauri::Manager;
use tauri::SystemTray;
use tauri::SystemTrayMenu;
use tauri::SystemTrayMenuItem;

fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "显示窗口"))
        .add_item(CustomMenuItem::new("quit", "退出"));

    tauri::Builder::default()
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => std::process::exit(0),
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Q: 开发模式下前端样式错误？

**A**: 检查 Vite 配置:

```ts
// frontend/vite.config.ts
export default defineConfig({
  // 确保 clearScreen 为 false（避免终端被清空）
  clearScreen: false,

  server: {
    port: 5173,
    strictPort: true,
  },

  // Tauri 需要的环境变量
  envPrefix: ['VITE_', 'TAURI_'],
})
```

## 故障排除

### 完整的诊断流程

```bash
# 1. 检查环境
just check-prereqs

# 2. 检查 Tauri 配置
cd frontend
npx tauri info

# 3. 清理所有缓存
just tauri-clean
rm -rf node_modules dist
npm install

# 4. 重新构建
just tauri-dev

# 5. 查看详细日志
RUST_LOG=debug just tauri-dev
```

### 获取帮助

如果问题仍未解决:

1. **查看 Tauri 官方文档**: https://tauri.app/
2. **搜索 GitHub Issues**: https://github.com/tauri-apps/tauri/issues
3. **提交问题**:
   - CCR 项目: https://github.com/bahayonghang/ccr/issues
   - Tauri 项目: https://github.com/tauri-apps/tauri/issues
4. **Discord 社区**: https://discord.gg/tauri

### 日志位置

**开发模式**: 终端直接输出

**生产版本**:
- macOS: `~/Library/Logs/com.ccr.desktop/`
- Windows: `%APPDATA%\com.ccr.desktop\logs\`
- Linux: `~/.config/com.ccr.desktop/logs/`

## 性能优化

### 启动速度优化

1. **减少初始化代码**
2. **延迟加载重量级模块**
3. **启用增量编译**:
   ```toml
   # frontend/src-tauri/.cargo/config.toml
   [build]
   incremental = true
   ```

### 运行时性能

1. **使用原生 API** 而不是轮询
2. **避免频繁的 IPC 调用**
3. **合理使用 Web Workers**

### 内存优化

1. **及时释放大对象**
2. **使用流式处理大文件**
3. **监控内存使用**:
   ```rust
   use tauri::Manager;

   #[tauri::command]
   fn get_memory_usage() -> Result<String, String> {
       // 实现内存监控
   }
   ```

## 下一步

现在你已经了解了 Tauri 桌面应用的开发和构建，可以：

1. 📖 阅读 [Tauri 官方文档](https://tauri.app/) 了解更多高级特性
2. 🎨 查看 [前端开发指南](/frontend/development) 定制 UI
3. 🔧 学习 [Rust 开发](https://www.rust-lang.org/learn) 扩展后端功能
4. 📦 参考 [项目结构](/guide/project-structure) 理解代码组织

## 参考资源

- 🌐 [Tauri 官方网站](https://tauri.app/)
- 📚 [Tauri 官方文档](https://tauri.app/v1/guides/)
- 🐙 [Tauri GitHub](https://github.com/tauri-apps/tauri)
- 💬 [Tauri Discord](https://discord.gg/tauri)
- 📖 [Rust 官方教程](https://www.rust-lang.org/learn)

---

<div style="text-align: center; margin-top: 2rem; padding: 1rem; background: var(--vp-c-bg-soft); border-radius: 8px;">
  <p>🖥️ <strong>享受 CCR Desktop 原生桌面应用的强大功能！</strong></p>
  <p>轻量、快速、安全的配置管理工具，随时随地高效管理你的 AI CLI 配置。</p>
</div>
