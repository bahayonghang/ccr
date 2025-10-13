# CCR Desktop - 开发和构建指南

## 🏗️ 快速构建命令

```bash
# 1. 在根项目安装 Rust 依赖 (只需一次)
cd /path/to/ccr
cargo check

# 2. 进入 Tauri 子项目
cd ccr-tauri

# 3. 安装 Tauri CLI (只需一次)
cargo install tauri-cli --version "^2.0.0" --locked
# 或者使用 justfile: just install-tauri-cli

# 4. 安装前端依赖 (只需一次)
cd src-ui
npm install
cd ..

# 5. 开发模式运行
cargo tauri dev

# 6. 构建生产版本
cargo tauri build

# 7. 🚀 智能打包（推荐）
just package          # 自动检测系统并打包
just package-linux    # Linux 平台 (.deb + .AppImage)
just package-macos    # macOS 平台 (.app + .dmg)
just package-windows  # Windows 平台 (.msi + .exe)
```

## 🔧 详细步骤

### 步骤 1: 环境准备

确保已安装：
- Rust 1.70+ (`rustup update`)
- Node.js 18+ (`node --version`)
- npm 9+ (`npm --version`)
- Tauri CLI 2.x (`cargo tauri --version`)

**安装 Tauri CLI：**
```bash
# 安装最新的 Tauri 2.x CLI
cargo install tauri-cli --version "^2.0.0" --locked

# 验证安装
cargo tauri --version
```

::: tip 提示
如果您使用 justfile，可以运行 `just setup` 自动安装所有依赖，包括 Tauri CLI。
:::

### 步骤 2: 系统依赖

#### macOS

```bash
xcode-select --install
```

#### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

#### Windows

WebView2 (Windows 10/11 自带，无需额外安装)

### 步骤 3: 前端依赖安装

```bash
cd ccr-tauri/src-ui
npm install
```

这将安装：
- Vue 3
- TypeScript
- Vite
- Tauri API 包
- 其他前端依赖

### 步骤 4: 开发模式

#### 方式 1: 使用 Cargo (推荐)

```bash
cd ccr-tauri
cargo tauri dev
```

#### 方式 2: 使用 npm

```bash
cd ccr-tauri/src-ui
npm run tauri:dev
```

**开发模式特性：**
- ✅ Rust 代码热重载 (需重新编译)
- ✅ Vue 3 热模块替换 (HMR)
- ✅ TypeScript 类型检查
- ✅ Vue DevTools
- ✅ Tauri DevTools (F12)

### 步骤 5: 生产构建

```bash
cd ccr-tauri
cargo tauri build
```

**构建产物位置：**

```
ccr-tauri/target/release/bundle/
├── macos/
│   ├── CCR Desktop.app           # macOS 应用包
│   └── dmg/
│       └── CCR Desktop_1.1.2_x64.dmg    # macOS 安装镜像
├── appimage/
│   └── ccr-desktop_1.1.2_amd64.AppImage # Linux AppImage
├── deb/
│   └── ccr-desktop_1.1.2_amd64.deb      # Debian 包
└── msi/
    └── CCR Desktop_1.1.2_x64_en-US.msi  # Windows 安装包
```

## 📦 智能打包系统

### 一键打包（推荐）

使用 `just package` 命令会自动检测当前系统并执行对应的打包：

```bash
cd ccr-tauri
just package
```

**工作流程：**

1. 🔍 自动检测运行平台（Linux/macOS/Windows）
2. 📦 选择对应的打包配置
3. 🏗️ 执行 Tauri 构建（包含 LTO 优化 + 符号剥离）
4. ✅ 显示构建产物位置和安装说明

### 分平台打包

#### 🐧 Linux 打包

```bash
just package-linux
```

**生成产物：**
- ✅ `.deb` - Debian/Ubuntu 安装包 (3.6 MB)
- ✅ `.rpm` - Fedora/RedHat 安装包 (3.6 MB)

**安装方法：**
```bash
# Debian/Ubuntu
sudo dpkg -i target/release/bundle/deb/CCR\ Desktop_*.deb
sudo apt-get install -f  # 解决依赖

# Fedora/RedHat
sudo rpm -i target/release/bundle/rpm/CCR\ Desktop-*.rpm
```

**技术细节：**
```bash
# 自动执行：
cargo tauri build --bundles deb,rpm
```

#### 🍎 macOS 打包

```bash
just package-macos
```

**生成产物：**
- ✅ `.app` - macOS 应用包 (~15 MB)
- ✅ `.dmg` - DMG 安装镜像 (~18 MB)

**安装方法：**
1. 双击打开 `.dmg` 文件
2. 拖动 CCR Desktop.app 到 Applications 文件夹
3. 首次运行需右键点击「打开」（如未签名）

**技术细节：**
```bash
# 自动执行：
cargo tauri build --bundles app,dmg
```

**代码签名（可选）：**
```bash
# 需要 Apple Developer 证书
codesign --force --deep --sign "Developer ID Application: Your Name" \
  target/release/bundle/macos/CCR\ Desktop.app
```

#### 🪟 Windows 打包

```bash
just package-windows
```

**生成产物：**
- ✅ `.msi` - MSI 安装包 (~16 MB)
- ✅ `.nsis` - NSIS 安装程序 (~16 MB)

**安装方法：**
```powershell
# 标准安装
msiexec /i "CCR Desktop_*.msi"

# 静默安装
msiexec /i "CCR Desktop_*.msi" /quiet
```

**技术细节：**
```bash
# 自动执行：
cargo tauri build --bundles msi,nsis
```

**代码签名（可选）：**
```powershell
# 需要 Code Signing Certificate
signtool sign /f certificate.pfx /p password /tr http://timestamp.digicert.com /td sha256 /fd sha256 "CCR Desktop_*.msi"
```

### 查看构建产物

```bash
just list-bundles
```

输出示例：
```
▶ 构建产物列表
  target/release/bundle/deb/ccr-desktop_1.1.3_amd64.deb (12.5M)
  target/release/bundle/appimage/ccr-desktop_1.1.3_amd64.AppImage (15.2M)
```

## 🎯 常用命令

### 开发命令

```bash
# 前端开发服务器 (仅 UI，无后端)
cd src-ui && npm run dev

# 前端类型检查
cd src-ui && npm run build

# Rust 代码检查
cd ccr-tauri && cargo check

# Rust 代码格式化
cd ccr-tauri && cargo fmt

# Rust 代码 Lint
cd ccr-tauri && cargo clippy
```

### 清理命令

```bash
# 清理 Rust 构建缓存
cd ccr-tauri && cargo clean

# 清理前端构建产物
cd src-ui && rm -rf dist node_modules

# 完全清理 (重新开始)
cd ccr-tauri
cargo clean
cd src-ui
rm -rf dist node_modules package-lock.json
npm install
```

## 📦 发布清单

发布新版本前的检查项：

- [ ] 更新版本号 (4 个文件):
  - [ ] `ccr-tauri/Cargo.toml`
  - [ ] `ccr-tauri/src-ui/package.json`
  - [ ] `ccr-tauri/tauri.conf.json`
  - [ ] 根项目 `Cargo.toml`
- [ ] 运行所有测试: `cargo test --all`
- [ ] 构建成功: `cargo tauri build`
- [ ] 在 3 个平台测试 (macOS, Linux, Windows)
- [ ] 更新 CHANGELOG.md
- [ ] 创建 Git tag: `git tag -a ccr-tauri-v1.1.2`
- [ ] 发布到 GitHub Releases

## 🔍 调试技巧

### Rust 后端调试

1. **启用详细日志：**

```bash
export RUST_LOG=ccr_tauri=trace,ccr=debug,tauri=debug
cargo tauri dev
```

2. **使用 LLDB/GDB 调试器：**

```bash
# macOS/Linux
rust-lldb target/debug/ccr-tauri

# Windows
rust-gdb target/debug/ccr-tauri.exe
```

### 前端调试

1. **浏览器 DevTools (F12)**
   - Console: 查看日志和错误
   - Network: 查看 Tauri Command 调用
   - Vue DevTools: 查看组件状态

2. **Vite 开发服务器日志：**

```bash
cd src-ui
npm run dev
# 查看 http://localhost:5173
```

### 性能分析

```bash
# 构建带调试信息的 Release 版本
cargo tauri build --debug

# 使用系统性能分析工具
# macOS: Instruments
# Linux: perf
# Windows: Windows Performance Analyzer
```

## 🚨 故障排查

### 问题: `cargo tauri dev` 启动失败

**可能原因 1: Tauri CLI 未安装**

```bash
# 错误信息: error: no such command: `tauri`
# 解决方法: 安装 Tauri CLI
cargo install tauri-cli --version "^2.0.0" --locked

# 或使用 justfile
just install-tauri-cli
```

**可能原因 2: 端口占用**

```bash
# 检查 5173 端口
lsof -i :5173
# 或修改 vite.config.ts 中的端口号
```

**可能原因 3: 前端未构建**

```bash
# 错误信息: The `frontendDist` configuration is set to `"src-ui/dist"` but this path doesn't exist
# 解决方法: 构建前端
cd src-ui
npm run build
cd ..
```

**可能原因 4: 依赖未安装**

```bash
cd src-ui
rm -rf node_modules package-lock.json
npm install
```

### 问题: 构建时找不到 CCR 核心库

确保在 Workspace 根目录：

```bash
cd ccr  # 根项目目录
cargo check --all
cd ccr-tauri
cargo tauri build
```

### 问题: Tauri Command 调用失败

1. 检查 Command 注册 (`src/main.rs`)
2. 检查参数类型匹配
3. 查看控制台错误信息:

```javascript
// 前端 src-ui/src/api/index.ts
try {
  await invoke('my_command', { param })
} catch (error) {
  console.error('Command failed:', error)
}
```

## 📚 参考资源

- [Tauri 官方文档](https://tauri.app/v2/guides/)
- [Vue 3 文档](https://vuejs.org/)
- [Vite 文档](https://vitejs.dev/)
- [CCR 主项目文档](../CLAUDE.md)

---

**祝你构建顺利！有问题随时查看这份文档～ (￣▽￣)／**
