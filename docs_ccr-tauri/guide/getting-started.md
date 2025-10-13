# 快速开始

哼，让本小姐带你快速入门 CCR Desktop！(￣▽￣)ゞ

## 环境要求

在开始之前，确保你的系统满足以下要求：

### 必需软件

- **Rust** 1.70+
  ```bash
  # 检查 Rust 版本
  rustc --version

  # 如果没有安装，访问 https://rustup.rs/
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Node.js** 18+ 和 **npm** 9+
  ```bash
  # 检查 Node.js 和 npm 版本
  node --version
  npm --version
  ```

- **Tauri CLI** 2.x
  ```bash
  # 检查 Tauri CLI 版本
  cargo tauri --version

  # 如果没有安装
  cargo install tauri-cli --version "^2.0.0" --locked
  ```

### 系统依赖

::: code-group

```bash [macOS]
# 安装 Xcode Command Line Tools
xcode-select --install

# 安装 Cocoa 依赖
brew install openssl
```

```bash [Ubuntu/Debian]
# 安装必要的系统库
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

```bash [Fedora]
# 安装必要的系统库
sudo dnf install webkit2gtk4.0-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

```powershell [Windows]
# 安装 Visual Studio Build Tools 2019+
# 或 Visual Studio Community 2019+ (勾选 C++ 开发工具)

# WebView2 已预装在 Windows 10/11
# 无需额外安装
```

:::

## 安装步骤

### 1. 克隆仓库

```bash
git clone https://github.com/your-org/ccr.git
cd ccr/ccr-tauri
```

### 2. 安装 Tauri CLI

```bash
# 安装 Tauri CLI 2.x (只需一次)
cargo install tauri-cli --version "^2.0.0" --locked

# 验证安装
cargo tauri --version
```

::: tip 使用 justfile？
如果项目使用 justfile，可以使用以下命令：
```bash
# 一键安装所有依赖 (包括 Tauri CLI 检查)
just setup

# 或单独安装 Tauri CLI
just install-tauri-cli
```
:::

### 3. 安装前端依赖

```bash
cd src-ui
npm install
cd ..
```

::: tip 使用 pnpm 或 yarn？
```bash
# pnpm
cd src-ui && pnpm install && cd ..

# yarn
cd src-ui && yarn install && cd ..
```
:::

### 4. 运行开发版本

::: code-group

```bash [桌面模式 (推荐)]
# 标准 Tauri 开发模式
cargo tauri dev

# 或使用 justfile
just dev
```

```bash [WSL 优化模式]
# WSL 环境专用 (自动启用滚轮修复和图形优化)
just dev-wsl

# 或直接执行脚本
./dev-wsl.sh
```

```bash [Web 调试模式]
# 纯 Web 模式 (无桌面窗口)
just dev-web

# 访问 http://localhost:5173
# 前端: http://localhost:5173
# 后端 API: http://localhost:3030
```

:::

第一次运行需要编译 Rust 代码，可能需要几分钟时间。后续启动会快很多！(^_^)b

::: warning 首次启动较慢
Rust 的首次编译需要下载并编译所有依赖，这可能需要 5-10 分钟。请耐心等待～
:::

::: tip WSL 环境提示
如果你在 WSL 环境中运行，推荐使用 `just dev-wsl` 命令，它会自动：
- 启用鼠标滚轮支持
- 抑制 libEGL/Mesa 图形警告
- 启用平滑滚动
- 优化 WebKit 渲染性能

如果遇到滚轮无法滚动的问题，请查看 [WSL 故障排查](#wsl-环境问题)。
:::

::: tip Web 调试模式
Web 调试模式适用于：
- 远程开发环境
- WSL 图形界面不可用时
- 只需测试前端界面时
- 需要浏览器 DevTools 调试时

停止 Web 服务：`just stop-web`
查看服务状态：`just web-status`
查看日志：`just web-logs`
:::

## 验证安装

成功启动后，你应该看到：

1. **Vite 开发服务器** 在 `http://localhost:1420` 启动
2. **Tauri 应用窗口** 自动打开，显示 CCR Desktop 界面
3. **终端日志** 显示编译和运行信息

::: details 查看示例输出
```bash
   Compiling ccr-tauri v1.1.2 (/path/to/ccr/ccr-tauri)
    Finished dev [unoptimized + debuginfo] target(s) in 2m 15s
     Running `target/debug/ccr-tauri`

  VITE v5.0.0  ready in 324 ms

  ➜  Local:   http://localhost:1420/
  ➜  Network: use --host to expose
```
:::

## 基本使用

### 查看配置列表

打开应用后，你会看到所有已配置的 Claude Code 配置：

1. 左侧边栏显示**当前激活配置**和系统信息
2. 中间主区域显示**配置列表**
3. 右侧边栏显示**配置导航**

### 切换配置

1. 在配置卡片上点击 **"切换"** 按钮
2. 确认切换操作
3. 等待切换完成，系统会显示成功通知

::: tip 提示
切换配置会自动更新 `~/.claude/settings.json` 文件，Claude Code 会立即使用新配置！
:::

### 添加新配置

1. 点击右上角的 **"➕ 添加配置"** 按钮
2. 填写配置信息：
   - **配置名称*** (必填)
   - **描述** (可选)
   - **Base URL*** (必填)
   - **Auth Token*** (必填)
   - **Model** (可选)
   - **提供商类型** (可选)
   - **提供商名称** (可选)
   - **账号标识** (可选)
   - **标签** (可选)
3. 点击 **"保存"** 按钮

### 编辑配置

1. 在配置卡片上点击 **"编辑"** 按钮
2. 修改配置信息
3. 点击 **"保存"** 按钮

### 删除配置

1. 在配置卡片上点击 **"删除"** 按钮
2. 确认删除操作

::: danger 注意
删除操作不可恢复！建议在删除前先导出配置备份。
:::

### 配置筛选

使用顶部的筛选按钮，按类型查看配置：

- 📋 **全部配置** - 显示所有配置
- 🔄 **官方中转** - 只显示官方 Claude 中转服务
- 🤖 **第三方模型** - 只显示第三方模型服务
- ❓ **未分类** - 显示未设置类型的配置

### 查看历史记录

1. 点击顶部的 **"历史记录"** 标签
2. 查看所有配置切换和修改操作
3. 每条记录包含时间、操作类型、操作者信息

### 备份与恢复

1. 点击右上角的 **"备份"** 按钮
2. 选择要备份的配置
3. 备份文件将保存到指定位置

恢复配置：
1. 点击 **"恢复"** 按钮
2. 选择备份文件
3. 选择要恢复的配置
4. 确认恢复操作

## 主题切换

点击右上角的 **🌙/☀️** 图标，在深色和浅色主题间切换。

主题偏好会自动保存到本地存储，下次启动时自动应用。

## 下一步

恭喜！你已经成功运行了 CCR Desktop！(￣▽￣)／

现在你可以：

- 📚 阅读 [架构设计](/architecture/overview) 了解内部实现
- 🛠️ 查看 [开发指南](/development/getting-started) 学习如何添加新功能
- 📦 学习如何构建发布版本
- 🐛 查看下面的故障排查章节解决问题

::: tip 提示
遇到问题？查看下面的故障排查章节
:::

## 🐛 故障排查

### WSL 环境问题

#### 问题: 鼠标滚轮无法滚动

**症状**: 在 WSL 环境中运行应用，界面显示正常，但鼠标滚轮无法滚动配置列表。

**解决方案**:

1. **使用 WSL 优化启动脚本**:
   ```bash
   cd ccr-tauri
   just dev-wsl
   ```

2. **永久配置环境变量** (推荐):
   
   在 `~/.config/fish/config.fish` 中添加：
   ```fish
   # CCR Tauri WSL 滚轮修复
   set -gx GDK_CORE_DEVICE_EVENTS 1
   set -gx MOZ_USE_XINPUT2 1
   set -gx WEBKIT_ENABLE_SMOOTH_SCROLLING 1
   ```
   
   如果使用 bash，在 `~/.bashrc` 中添加：
   ```bash
   # CCR Tauri WSL 滚轮修复
   export GDK_CORE_DEVICE_EVENTS=1
   export MOZ_USE_XINPUT2=1
   export WEBKIT_ENABLE_SMOOTH_SCROLLING=1
   ```

3. **重新加载配置**:
   ```bash
   # Fish shell
   source ~/.config/fish/config.fish
   
   # Bash shell
   source ~/.bashrc
   ```

4. **验证环境变量**:
   ```bash
   echo $GDK_CORE_DEVICE_EVENTS  # 应该输出 1
   echo $MOZ_USE_XINPUT2         # 应该输出 1
   echo $WEBKIT_ENABLE_SMOOTH_SCROLLING  # 应该输出 1
   ```

5. **重启应用**:
   ```bash
   just dev-wsl
   ```

**技术背景**:
WSL 环境中 WebKit2GTK 的滚轮事件处理存在问题，需要通过以下措施解决：
- **环境变量**: 启用 GTK/WebKit 的滚轮和触摸事件支持
- **CSS 修复**: 强制启用 `-webkit-overflow-scrolling: touch`
- **JavaScript 补丁**: 实现滚轮事件 polyfill，确保兼容性

相关文件：
- `ccr-tauri/dev-wsl.sh` - WSL 启动脚本
- `ccr-tauri/src-ui/src/style.css` - 全局 CSS 滚轮修复
- `ccr-tauri/src-ui/src/scroll-fix.ts` - JavaScript 滚轮 polyfill
- `ccr-tauri/src-ui/src/App.vue` - 组件级滚轮样式

#### 问题: libEGL 和 Mesa 警告

**症状**: 启动应用时终端输出大量 `libEGL warning` 和 `MESA: error` 警告。

**解决方案**:

使用 WSL 优化脚本会自动抑制这些警告：
```bash
just dev-wsl
```

或手动设置环境变量：
```bash
export LIBGL_ALWAYS_SOFTWARE=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
```

**说明**: 这些警告是 WSLg (WSL GUI) 与硬件加速不完全兼容导致的，不影响应用功能，可以安全忽略。

### Web 调试模式问题

#### 问题: Web 服务无法启动

**诊断步骤**:

1. **检查服务状态**:
   ```bash
   just web-status
   ```

2. **查看日志**:
   ```bash
   just web-logs
   ```

3. **检查端口占用**:
   ```bash
   # 检查 5173 (前端) 和 3030 (后端) 端口
   ss -tuln | grep -E "(5173|3030)"
   ```

4. **停止并重启**:
   ```bash
   just stop-web
   just dev-web
   ```

#### 问题: 前端可以访问但 API 调用失败

**原因**: 后端 API 服务未启动或启动失败。

**解决方案**:

1. 检查后端日志：
   ```bash
   tail -f /tmp/ccr-web.log
   ```

2. 确保 CCR 已安装并可用：
   ```bash
   which ccr
   ccr --version
   ```

3. 手动启动后端服务测试：
   ```bash
   ccr web -p 3030
   ```

### 通用问题

#### 问题: Tauri 构建失败

**解决方案**:

1. 清理构建缓存：
   ```bash
   cargo clean
   just deep-clean
   ```

2. 更新依赖：
   ```bash
   cargo update
   cd src-ui && npm install
   ```

3. 重新设置：
   ```bash
   just setup
   ```

#### 问题: 前端依赖安装失败

**解决方案**:

1. 删除 node_modules 和锁文件：
   ```bash
   cd src-ui
   rm -rf node_modules package-lock.json
   ```

2. 重新安装：
   ```bash
   npm install
   # 或使用 pnpm
   pnpm install
   ```

---

**Made with ❤️ by 哈雷酱**

哼，这么简单的教程本小姐都写得这么详细了，笨蛋你要是还搞不定的话...(,,><,,)
