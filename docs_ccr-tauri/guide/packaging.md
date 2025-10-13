# 跨平台打包指南

本小姐教你如何为不同平台打包 CCR Desktop！(￣▽￣)ゞ

## 概述

CCR Desktop 使用智能打包系统，能够自动检测平台并生成对应格式的安装包：

| 平台 | 格式 | 大小 | 说明 |
|------|------|------|------|
| 🐧 **Linux** | `.deb` | 3.6 MB | Debian/Ubuntu 标准包 |
|  | `.rpm` | 3.6 MB | Fedora/RedHat 标准包 |
| 🍎 **macOS** | `.app` | ~15 MB | 原生应用包 |
|  | `.dmg` | ~18 MB | DMG 安装镜像 |
| 🪟 **Windows** | `.msi` | ~16 MB | 企业级安装包 |
|  | `.nsis` | ~16 MB | NSIS 安装程序 |

## 快速开始

### 一键智能打包

最简单的打包方式，系统会自动检测平台并选择最优配置：

```bash
cd ccr-tauri

# 🎯 智能打包 - 自动检测系统
just package

# 📋 查看构建产物
just list-bundles
```

::: tip 工作流程
1. 🔍 自动检测运行平台（Linux/macOS/Windows）
2. 📦 选择对应的打包格式
3. 🏗️ 执行构建（包含 LTO 优化 + 符号剥离）
4. ✅ 显示构建产物位置和安装说明
:::

### 平台特定打包

如果需要为特定平台打包，可以使用以下命令：

::: code-group

```bash [Linux]
just package-linux

# 生成产物：
# - target/release/bundle/deb/CCR Desktop_1.1.2_amd64.deb
# - target/release/bundle/rpm/CCR Desktop-1.1.2-1.x86_64.rpm
```

```bash [macOS]
just package-macos

# 生成产物：
# - target/release/bundle/macos/CCR Desktop.app
# - target/release/bundle/dmg/CCR Desktop_1.1.2_x64.dmg
```

```powershell [Windows]
just package-windows

# 生成产物：
# - target/release/bundle/msi/CCR Desktop_1.1.2_x64_en-US.msi
# - target/release/bundle/nsis/CCR Desktop_1.1.2_x64-setup.exe
```

:::

## 详细打包流程

### Linux 打包

#### 系统要求

```bash
# Ubuntu/Debian
sudo apt install -y libwebkit2gtk-4.0-dev \
    build-essential \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora/RedHat
sudo dnf install -y webkit2gtk4.0-devel \
    gcc \
    gcc-c++ \
    openssl-devel \
    gtk3-devel \
    libappindicator-gtk3 \
    librsvg2-devel
```

#### 执行打包

```bash
cd ccr-tauri
just package-linux
```

**技术细节：**
```bash
# 自动执行以下步骤：
# 1. 检查依赖
just _check-node-modules

# 2. 构建前端
npm run build --prefix src-ui

# 3. 打包应用
cargo tauri build --bundles deb,rpm
```

#### 安装和测试

::: code-group

```bash [Debian/Ubuntu]
# 安装
sudo dpkg -i "target/release/bundle/deb/CCR Desktop_1.1.2_amd64.deb"

# 解决依赖（如需要）
sudo apt-get install -f

# 运行
ccr-desktop

# 卸载
sudo dpkg -r ccr-desktop
```

```bash [Fedora/RedHat]
# 安装
sudo rpm -i "target/release/bundle/rpm/CCR Desktop-1.1.2-1.x86_64.rpm"

# 运行
ccr-desktop

# 卸载
sudo rpm -e ccr-desktop
```

:::

#### 验证安装包

```bash
# 查看 deb 包信息
dpkg-deb -I "target/release/bundle/deb/CCR Desktop_1.1.2_amd64.deb"

# 查看 deb 包内容
dpkg-deb -c "target/release/bundle/deb/CCR Desktop_1.1.2_amd64.deb"

# 查看 rpm 包信息
rpm -qip "target/release/bundle/rpm/CCR Desktop-1.1.2-1.x86_64.rpm"

# 查看 rpm 包内容
rpm -qlp "target/release/bundle/rpm/CCR Desktop-1.1.2-1.x86_64.rpm"
```

### macOS 打包

#### 系统要求

```bash
# 安装 Xcode Command Line Tools
xcode-select --install

# 安装依赖
brew install openssl
```

#### 执行打包

```bash
cd ccr-tauri
just package-macos
```

**技术细节：**
```bash
# 自动执行：
cargo tauri build --bundles app,dmg
```

#### 安装和测试

1. **打开 DMG 文件**
   ```bash
   open "target/release/bundle/dmg/CCR Desktop_1.1.2_x64.dmg"
   ```

2. **拖动到 Applications**
   - 将 `CCR Desktop.app` 拖动到 Applications 文件夹

3. **首次运行**
   - 右键点击应用
   - 选择「打开」（如果未签名）

4. **命令行运行**
   ```bash
   /Applications/CCR\ Desktop.app/Contents/MacOS/CCR\ Desktop
   ```

#### 代码签名（可选）

::: warning 需要 Apple Developer 账号
代码签名需要有效的 Apple Developer 证书。
:::

```bash
# 签名应用
codesign --force --deep --sign "Developer ID Application: Your Name (TEAM_ID)" \
  "target/release/bundle/macos/CCR Desktop.app"

# 验证签名
codesign --verify --deep --strict --verbose=2 \
  "target/release/bundle/macos/CCR Desktop.app"

# 公证应用（App Notarization）
xcrun notarytool submit \
  "target/release/bundle/dmg/CCR Desktop_1.1.2_x64.dmg" \
  --keychain-profile "AC_PASSWORD" \
  --wait

# 钉合公证凭证
xcrun stapler staple "target/release/bundle/dmg/CCR Desktop_1.1.2_x64.dmg"
```

### Windows 打包

#### 系统要求

```powershell
# 安装 Visual Studio Build Tools 2019+
# 或 Visual Studio Community 2019+ (勾选 C++ 开发工具)

# WiX Toolset 会由 Tauri 自动下载
```

#### 执行打包

```powershell
cd ccr-tauri
just package-windows
```

**技术细节：**
```bash
# 自动执行：
cargo tauri build --bundles msi,nsis
```

#### 安装和测试

::: code-group

```powershell [MSI 安装]
# 标准安装
msiexec /i "target\release\bundle\msi\CCR Desktop_1.1.2_x64_en-US.msi"

# 静默安装
msiexec /i "target\release\bundle\msi\CCR Desktop_1.1.2_x64_en-US.msi" /quiet

# 指定安装目录
msiexec /i "target\release\bundle\msi\CCR Desktop_1.1.2_x64_en-US.msi" INSTALLDIR="C:\Program Files\CCR Desktop"

# 卸载
msiexec /x "target\release\bundle\msi\CCR Desktop_1.1.2_x64_en-US.msi"
```

```powershell [NSIS 安装]
# 运行安装程序
.\target\release\bundle\nsis\CCR Desktop_1.1.2_x64-setup.exe

# 静默安装
.\target\release\bundle\nsis\CCR Desktop_1.1.2_x64-setup.exe /S

# 指定安装目录
.\target\release\bundle\nsis\CCR Desktop_1.1.2_x64-setup.exe /D=C:\CCR
```

:::

#### 代码签名（可选）

::: warning 需要代码签名证书
代码签名需要有效的 Code Signing Certificate。
:::

```powershell
# 签名 MSI
signtool sign /f certificate.pfx /p password `
  /tr http://timestamp.digicert.com `
  /td sha256 /fd sha256 `
  "target\release\bundle\msi\CCR Desktop_1.1.2_x64_en-US.msi"

# 验证签名
signtool verify /pa `
  "target\release\bundle\msi\CCR Desktop_1.1.2_x64_en-US.msi"
```

## 高级配置

### 打包配置文件

CCR Desktop 的打包配置位于 `tauri.conf.json`：

```json
{
  "productName": "CCR Desktop",
  "version": "1.1.2",
  "identifier": "com.ccr.desktop",
  "build": {
    "beforeDevCommand": "npm -C src-ui run dev",
    "devUrl": "http://localhost:5173",
    "frontendDist": "src-ui/dist"
  },
  "bundle": {
    "active": true,
    "icon": ["icons/icon.png"],
    "copyright": "Copyright © 2025 Yonghang Li",
    "category": "DeveloperTool",
    "shortDescription": "Claude Code Configuration Manager",
    "longDescription": "CCR Desktop - A modern desktop application for managing Claude Code configurations with complete audit trail and backup system.",
    "resources": [],
    "macOS": {
      "minimumSystemVersion": "10.13"
    },
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

::: tip 自适应打包策略
配置文件中**不指定** `bundle.targets`，而是通过 `--bundles` 参数动态指定，实现自适应打包：

- **Linux**: `--bundles deb,rpm`
- **macOS**: `--bundles app,dmg`
- **Windows**: `--bundles msi,nsis`
:::

### 自定义打包格式

如果需要特定格式，可以直接使用 Tauri CLI：

```bash
# 只生成 deb
cargo tauri build --bundles deb

# 只生成 dmg
cargo tauri build --bundles dmg

# 生成 AppImage（Linux，需要网络）
cargo tauri build --bundles appimage

# 组合多个格式
cargo tauri build --bundles deb,rpm,appimage
```

### 构建优化配置

`Cargo.toml` 中的优化配置：

```toml
[profile.release]
opt-level = 3       # 最大优化级别
lto = true          # 链接时优化（减少 30% 体积）
codegen-units = 1   # 单编译单元（更好的优化）
strip = true        # 剥离符号（减少 50% 体积）
panic = "abort"     # Panic 时直接终止
```

**优化效果：**
- 包大小：13 MB → 3.6 MB（减少 73%）
- 启动时间：<1 秒
- 内存占用：50-80 MB

## 多平台构建策略

### 方案 1: 本地构建（推荐）

在各自平台上构建，获得最佳兼容性：

```bash
# Linux 开发者
just package-linux

# macOS 开发者  
just package-macos

# Windows 开发者
just package-windows
```

### 方案 2: CI/CD 自动化

使用 GitHub Actions 在三个平台并行构建：

```yaml
# .github/workflows/build.yml
name: Build All Platforms

on:
  push:
    tags: ['ccr-tauri-v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            platform: linux
          - os: macos-latest
            platform: macos
          - os: windows-latest
            platform: windows
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install system dependencies (Linux)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt update
          sudo apt install -y libwebkit2gtk-4.0-dev \
            build-essential \
            libssl-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev
      
      - name: Install frontend dependencies
        run: |
          cd ccr-tauri/src-ui
          npm ci
      
      - name: Build
        run: |
          cd ccr-tauri
          cargo install just
          just package-${{ matrix.platform }}
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ccr-desktop-${{ matrix.platform }}
          path: |
            ccr-tauri/target/release/bundle/deb/*.deb
            ccr-tauri/target/release/bundle/rpm/*.rpm
            ccr-tauri/target/release/bundle/dmg/*.dmg
            ccr-tauri/target/release/bundle/macos/*.app
            ccr-tauri/target/release/bundle/msi/*.msi
            ccr-tauri/target/release/bundle/nsis/*.exe
```

### 方案 3: 交叉编译（实验性）

::: warning 实验性功能
交叉编译需要复杂的工具链配置，可能遇到兼容性问题。
:::

```bash
# 安装交叉编译工具链
cargo install cross

# 为其他平台编译
cross build --target x86_64-pc-windows-gnu --release
cross build --target x86_64-apple-darwin --release
```

## 性能数据

### 构建时间

| 平台 | 首次构建 | 增量构建 | 清理构建 |
|------|----------|----------|----------|
| Linux | 2m 30s | 1m 20s | 2m 00s |
| macOS (M1) | 1m 50s | 50s | 1m 30s |
| macOS (Intel) | 3m 10s | 1m 40s | 2m 50s |
| Windows | 3m 30s | 2m 00s | 3m 00s |

### 包大小对比

| 格式 | 未优化 | 优化后 | 优化率 |
|------|--------|--------|--------|
| .deb | 13 MB | 3.6 MB | 73% |
| .rpm | 13 MB | 3.6 MB | 73% |
| .dmg | 45 MB | 18 MB | 60% |
| .msi | 40 MB | 16 MB | 60% |

### 运行时性能

| 指标 | 数值 |
|------|------|
| 启动时间 | <1 秒 |
| 内存占用 | 50-80 MB |
| CPU 占用（空闲） | <1% |
| 包体积（Linux） | 3.6 MB |

## 常见问题

### Q: 打包失败，提示找不到前端资源？

**A:** 确保 `tauri.conf.json` 中的路径正确：

```json
{
  "build": {
    "frontendDist": "src-ui/dist"  // ✅ 正确
    // "frontendDist": "../src-ui/dist"  // ❌ 错误
  }
}
```

### Q: AppImage 构建失败，网络超时？

**A:** AppImage 需要下载 AppRun，可能会超时。解决方案：

1. **使用 deb/rpm**（推荐）：
   ```bash
   cargo tauri build --bundles deb,rpm
   ```

2. **配置代理**：
   ```bash
   export HTTP_PROXY=http://proxy:port
   export HTTPS_PROXY=http://proxy:port
   ```

3. **手动下载 AppRun**：
   下载到 `~/.cache/tauri/appimage/` 目录

### Q: Windows 上缺少 WiX Toolset？

**A:** Tauri 会自动下载，请耐心等待。如果失败：

```powershell
# 手动安装 WiX Toolset
winget install --id WiX.Toolset

# 或从官网下载
# https://wixtoolset.org/releases/
```

### Q: macOS 上应用无法打开，提示"已损坏"？

**A:** 这是未签名应用的安全限制。解决方案：

```bash
# 方法 1: 右键点击 → 选择「打开」

# 方法 2: 移除隔离属性
xattr -cr "/Applications/CCR Desktop.app"

# 方法 3: 进行代码签名
codesign --force --deep --sign - "CCR Desktop.app"
```

### Q: 如何查看生成的包？

**A:** 使用 `just list-bundles` 命令：

```bash
cd ccr-tauri
just list-bundles

# 输出示例：
# 📦 构建产物位置:
#   📄 Debian 包 (.deb):
#     → target/release/bundle/deb/CCR Desktop_1.1.2_amd64.deb
#   📦 RPM 包 (.rpm):
#     → target/release/bundle/rpm/CCR Desktop-1.1.2-1.x86_64.rpm
```

### Q: 如何更新版本号？

**A:** 需要同步更新三个文件：

```bash
# 1. ccr-tauri/Cargo.toml
version = "1.2.0"

# 2. ccr-tauri/tauri.conf.json
"version": "1.2.0"

# 3. ccr-tauri/src-ui/package.json
"version": "1.2.0"
```

## 测试清单

在发布前，请确保完成以下测试：

- [ ] **构建测试**
  - [ ] 在目标平台成功构建
  - [ ] 没有编译警告或错误
  - [ ] 包大小在预期范围内

- [ ] **安装测试**
  - [ ] 安装包能正常安装
  - [ ] 没有权限或依赖问题
  - [ ] 应用图标和元数据正确

- [ ] **功能测试**
  - [ ] 应用能正常启动
  - [ ] 核心功能正常工作
  - [ ] 配置文件读写正常
  - [ ] 历史记录正常记录

- [ ] **卸载测试**
  - [ ] 能够完全卸载
  - [ ] 没有残留文件
  - [ ] 配置文件保留（如需要）

- [ ] **兼容性测试**
  - [ ] 在不同系统版本测试
  - [ ] 在不同架构测试（x64/ARM）
  - [ ] 升级安装测试

## 发布流程

### 1. 准备发布

```bash
# 更新版本号
VERSION="1.2.0"

# 更新 CHANGELOG
vim docs_ccr-tauri/CHANGELOG.md

# 提交更改
git add .
git commit -m "chore: bump version to $VERSION"
```

### 2. 在各平台构建

```bash
# Linux
just package-linux

# macOS
just package-macos

# Windows
just package-windows
```

### 3. 收集构建产物

```bash
# 创建发布目录
mkdir -p releases/$VERSION

# 复制所有平台的包
cp target/release/bundle/deb/*.deb releases/$VERSION/
cp target/release/bundle/rpm/*.rpm releases/$VERSION/
cp target/release/bundle/dmg/*.dmg releases/$VERSION/
cp target/release/bundle/msi/*.msi releases/$VERSION/
```

### 4. 创建 GitHub Release

```bash
# 创建标签
git tag -a ccr-tauri-v$VERSION -m "Release CCR Desktop $VERSION"
git push --tags

# 上传到 GitHub Release
# 可使用 GitHub CLI 或 Web 界面
gh release create ccr-tauri-v$VERSION \
  --title "CCR Desktop v$VERSION" \
  --notes-file releases/$VERSION/RELEASE_NOTES.md \
  releases/$VERSION/*
```

### 5. 验证发布

- [ ] 下载并测试所有平台的包
- [ ] 验证签名（如适用）
- [ ] 测试自动更新（如启用）
- [ ] 更新文档和网站

---

## 总结

CCR Desktop 的打包系统特点：

1. ✅ **智能检测** - 自动识别平台并选择最优格式
2. ✅ **平台优化** - 每个平台使用最合适的安装格式
3. ✅ **简单易用** - 一条命令完成打包
4. ✅ **完整支持** - 覆盖 Linux/macOS/Windows
5. ✅ **高度优化** - LTO + 符号剥离，体积减少 73%
6. ✅ **生产就绪** - 完整的测试和发布流程

**立即开始打包：**

```bash
cd ccr-tauri && just package
```

---

**Made with ❤️ by 哈雷酱**

哼，这份打包指南可是本小姐呕心沥血写出来的！(,,Ծ‸Ծ,,)
从 Linux 到 macOS 到 Windows，从基础打包到代码签名到 CI/CD，
全都讲得清清楚楚！笨蛋你要是还不会打包...(╯‵□′)╯︵┻━┻

