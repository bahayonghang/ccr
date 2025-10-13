# WSL 环境故障排查

本小姐专门为 WSL 用户准备的故障排查指南！(￣▽￣)ゞ

## 🐧 WSL 环境概述

CCR Desktop 在 WSL (Windows Subsystem for Linux) 环境中运行时，可能会遇到一些特殊问题。这是因为 WSL 使用 WSLg (WSL GUI) 来支持图形应用，而 WSLg 与原生 Linux 桌面环境有一些差异。

### WSLg 架构

```
┌───────────────────────────────────────────────────┐
│  Windows 宿主机                                    │
│  ┌─────────────────────────────────────────────┐  │
│  │  WSLg (Weston 合成器)                        │  │
│  │  • X11 服务器                                │  │
│  │  │  Wayland 合成器                           │  │
│  └──┴────────────────────────────────────────────┘  │
│     ▲                                              │
│     │ 通过虚拟化通道                                │
│     ▼                                              │
│  ┌──────────────────────────────────────────────┐  │
│  │  WSL2 Linux 环境                              │  │
│  │  ┌────────────────────────────────────────┐  │  │
│  │  │  CCR Desktop (Tauri + WebKit2GTK)      │  │  │
│  │  │  • Vue 3 前端                          │  │  │
│  │  │  • Rust 后端                           │  │  │
│  │  └────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

## ⚠️ 常见问题

### 问题 1: 鼠标滚轮无法滚动

**症状**: 
- 应用界面正常显示
- 可以点击按钮和操作
- 但鼠标滚轮无法滚动配置列表或任何可滚动区域

**原因**:
WSL 环境中 WebKit2GTK 的滚轮事件处理存在 bug：
1. GTK 默认禁用了核心设备事件
2. WebKit 没有正确处理 XInput2 事件
3. 滚轮事件被阻止，无法传递到 DOM

**解决方案 1: 使用 WSL 优化启动脚本** (推荐)

```bash
cd ccr-tauri
just dev-wsl
```

这个命令会自动设置必要的环境变量和启用滚轮修复。

**解决方案 2: 永久配置环境变量**

1. 编辑 Fish shell 配置 (如果使用 Fish):
   ```bash
   vim ~/.config/fish/config.fish
   ```

   添加以下内容：
   ```fish
   # CCR Tauri WSL 滚轮修复
   set -gx GDK_CORE_DEVICE_EVENTS 1
   set -gx MOZ_USE_XINPUT2 1
   set -gx WEBKIT_ENABLE_SMOOTH_SCROLLING 1
   ```

2. 或编辑 Bash 配置 (如果使用 Bash):
   ```bash
   vim ~/.bashrc
   ```

   添加以下内容：
   ```bash
   # CCR Tauri WSL 滚轮修复
   export GDK_CORE_DEVICE_EVENTS=1
   export MOZ_USE_XINPUT2=1
   export WEBKIT_ENABLE_SMOOTH_SCROLLING=1
   ```

3. 重新加载配置：
   ```bash
   # Fish shell
   source ~/.config/fish/config.fish
   
   # Bash shell
   source ~/.bashrc
   ```

4. 验证环境变量：
   ```bash
   echo $GDK_CORE_DEVICE_EVENTS         # 应该输出 1
   echo $MOZ_USE_XINPUT2                # 应该输出 1
   echo $WEBKIT_ENABLE_SMOOTH_SCROLLING # 应该输出 1
   ```

5. 重启应用：
   ```bash
   cd ccr-tauri
   cargo tauri dev
   # 或
   just dev
   ```

**解决方案 3: 使用 Web 调试模式**

如果环境变量方法不起作用，可以使用 Web 调试模式：

```bash
cd ccr-tauri
just dev-web
```

然后在 Windows 浏览器中访问 http://localhost:5173，滚轮将正常工作。

**技术细节**:

CCR Desktop 实现了**三层滚轮修复方案**：

1. **环境变量层** (`dev-wsl.sh`):
   - `GDK_CORE_DEVICE_EVENTS=1`: 启用 GTK 核心设备事件
   - `MOZ_USE_XINPUT2=1`: 启用 XInput2 扩展
   - `WEBKIT_ENABLE_SMOOTH_SCROLLING=1`: 启用 WebKit 平滑滚动

2. **CSS 层** (`src-ui/src/style.css`):
   ```css
   * {
     -webkit-overflow-scrolling: touch;
   }
   
   .scrollable {
     overflow-y: auto !important;
     -webkit-overflow-scrolling: touch !important;
     overscroll-behavior-y: contain !important;
     touch-action: pan-y !important;
     pointer-events: auto !important;
   }
   ```

3. **JavaScript 层** (`src-ui/src/scroll-fix.ts`):
   ```typescript
   // 监听滚轮事件，手动处理滚动
   window.addEventListener('wheel', (e) => {
     const scrollable = findScrollableParent(e.target)
     if (scrollable) {
       scrollable.scrollTop += e.deltaY
       e.preventDefault()
     }
   }, { passive: false })
   ```

**相关文件**:
- `ccr-tauri/dev-wsl.sh` - WSL 启动脚本
- `ccr-tauri/src-ui/src/style.css` - 全局 CSS 滚轮修复
- `ccr-tauri/src-ui/src/scroll-fix.ts` - JavaScript 滚轮 polyfill
- `ccr-tauri/src-ui/src/App.vue` - 组件级滚轮样式
- `ccr-tauri/tauri.conf.json` - CSP 配置 (允许内联脚本)

---

### 问题 2: libEGL 和 Mesa 警告

**症状**:

启动应用时终端输出大量警告：

```
libEGL warning: wayland-egl: could not open /dev/dri/renderD128 (No such file or directory)
MESA: error: ZINK: failed to choose pdev
glx: failed to create drisw screen
```

**原因**:

WSLg 尝试使用硬件加速 (GPU)，但 WSL2 虚拟机与 Windows 宿主机的 GPU 共享机制不完全兼容，导致 EGL (OpenGL ES 接口) 初始化失败。

**影响**:

**这些警告是无害的！** 应用会自动回退到软件渲染模式，功能完全正常，只是可能略微降低性能 (对于 CCR Desktop 这样的配置管理应用，差异可以忽略不计)。

**解决方案 1: 抑制警告** (推荐)

使用 WSL 优化脚本自动抑制警告：

```bash
cd ccr-tauri
just dev-wsl
```

**解决方案 2: 手动设置环境变量**

```bash
export LIBGL_ALWAYS_SOFTWARE=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
```

然后正常启动应用：

```bash
cargo tauri dev
```

**解决方案 3: 永久配置**

在 `~/.config/fish/config.fish` (Fish shell) 或 `~/.bashrc` (Bash) 中添加：

```bash
# 抑制 WSL 图形警告
export LIBGL_ALWAYS_SOFTWARE=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
```

**解决方案 4: 使用 Web 调试模式**

Web 调试模式完全不依赖图形界面，没有这些警告：

```bash
just dev-web
```

---

### 问题 3: 窗口无法显示或黑屏

**症状**:
- 应用启动，终端显示正在运行
- 但窗口不显示，或显示黑屏

**原因**:
WSLg 服务未正确启动，或 DISPLAY 环境变量未设置。

**解决方案 1: 检查 DISPLAY 环境变量**

```bash
echo $DISPLAY
# 应该输出类似 :0 或 :1
```

如果没有输出，设置 DISPLAY：

```bash
export DISPLAY=:0
```

**解决方案 2: 重启 WSL**

在 Windows PowerShell 中：

```powershell
wsl --shutdown
wsl
```

然后重新启动 CCR Desktop。

**解决方案 3: 检查 WSLg 状态**

在 WSL 中：

```bash
ps aux | grep -E "(Xwayland|weston)"
```

应该看到 Xwayland 和 weston 进程正在运行。如果没有，重启 WSL。

**解决方案 4: 使用 Web 调试模式**

```bash
cd ccr-tauri
just dev-web
```

---

### 问题 4: 应用启动慢

**症状**:
- 在 WSL 中启动应用需要很长时间 (>30 秒)
- 首次编译 Rust 代码需要 10+ 分钟

**原因**:
1. WSL2 的文件系统性能 (特别是跨文件系统访问)
2. Rust 编译需要大量 I/O

**解决方案 1: 确保项目在 Linux 文件系统中**

**不要**把项目放在 Windows 文件系统中 (`/mnt/c/...`)，而应该放在 Linux 文件系统中 (`~/...`):

```bash
# ❌ 慢 (跨文件系统)
cd /mnt/c/Users/YourName/Projects/ccr

# ✅ 快 (Linux 文件系统)
cd ~/Projects/ccr
```

如果项目在 Windows 文件系统中，迁移到 Linux 文件系统：

```bash
# 复制项目到 Linux 文件系统
cp -r /mnt/c/Users/YourName/Projects/ccr ~/Projects/

# 在新位置构建
cd ~/Projects/ccr/ccr-tauri
cargo build
```

**解决方案 2: 使用缓存**

Rust 的构建缓存会显著加快后续编译：

```bash
# 首次构建 (慢)
cargo build

# 后续构建 (快)
cargo build  # 只重新编译修改的部分
```

**解决方案 3: 使用 sccache**

安装 sccache 可以缓存 Rust 编译结果：

```bash
cargo install sccache

# 配置 Cargo 使用 sccache
export RUSTC_WRAPPER=sccache

# 构建
cargo build
```

**解决方案 4: 增加 WSL 资源**

编辑 `%USERPROFILE%\.wslconfig` (Windows 路径)：

```ini
[wsl2]
memory=8GB
processors=4
```

重启 WSL：

```powershell
wsl --shutdown
wsl
```

---

### 问题 5: 字体显示模糊或丑陋

**症状**:
- 应用中的文字显示模糊
- 字体渲染质量差

**原因**:
WSLg 的字体渲染设置不够好，或缺少字体。

**解决方案 1: 安装更好的字体**

```bash
# Ubuntu/Debian
sudo apt install fonts-noto fonts-noto-cjk fonts-noto-color-emoji

# Fedora
sudo dnf install google-noto-fonts google-noto-cjk-fonts google-noto-emoji-fonts
```

**解决方案 2: 配置字体渲染**

创建或编辑 `~/.config/fontconfig/fonts.conf`:

```xml
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
  <match target="font">
    <edit name="antialias" mode="assign">
      <bool>true</bool>
    </edit>
    <edit name="hinting" mode="assign">
      <bool>true</bool>
    </edit>
    <edit name="hintstyle" mode="assign">
      <const>hintslight</const>
    </edit>
    <edit name="rgba" mode="assign">
      <const>rgb</const>
    </edit>
    <edit name="lcdfilter" mode="assign">
      <const>lcddefault</const>
    </edit>
  </match>
</fontconfig>
```

重新加载字体配置：

```bash
fc-cache -fv
```

**解决方案 3: 使用 Web 调试模式**

Web 调试模式使用浏览器渲染，字体质量通常更好。访问 http://localhost:5173 （需要先启动 Web 调试模式）。

---

### 问题 6: 无法访问 ~/.claude/ 目录

**症状**:
- 应用提示无法找到配置文件
- 错误信息: "Permission denied" 或 "No such file or directory"

**原因**:
1. CCR 配置文件不存在
2. 文件权限问题

**解决方案 1: 初始化 CCR**

```bash
# 使用 CCR CLI 初始化
ccr init

# 创建第一个配置
ccr update anthropic \
  --base-url "https://api.anthropic.com" \
  --auth-token "your-token" \
  --model "claude-3-5-sonnet-20241022"
```

**解决方案 2: 检查文件权限**

```bash
ls -la ~/.ccs_config.toml
ls -la ~/.claude/

# 如果权限不对，修正
chmod 600 ~/.ccs_config.toml
chmod 700 ~/.claude/
```

**解决方案 3: 检查文件是否存在**

```bash
# 配置文件
ls -la ~/.ccs_config.toml

# Settings 文件
ls -la ~/.claude/settings.json

# 如果不存在，创建默认配置
mkdir -p ~/.claude
touch ~/.claude/settings.json
echo '{}' > ~/.claude/settings.json
```

---

## 💡 最佳实践

### 1. 推荐的 WSL 开发环境

```bash
# 1. 使用 Fish shell (可选，但推荐)
sudo apt install fish
chsh -s /usr/bin/fish

# 2. 安装完整的开发工具
sudo apt install build-essential curl wget git

# 3. 安装 Rust (如果还没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. 克隆项目到 Linux 文件系统
cd ~
git clone https://github.com/harleyqing/ccr.git
cd ccr/ccr-tauri

# 5. 配置环境变量 (Fish shell)
vim ~/.config/fish/config.fish
```

在 `~/.config/fish/config.fish` 中添加：

```fish
# CCR Tauri WSL 优化
set -gx GDK_CORE_DEVICE_EVENTS 1
set -gx MOZ_USE_XINPUT2 1
set -gx WEBKIT_ENABLE_SMOOTH_SCROLLING 1
set -gx LIBGL_ALWAYS_SOFTWARE 1
set -gx WEBKIT_DISABLE_COMPOSITING_MODE 1
set -gx WEBKIT_DISABLE_DMABUF_RENDERER 1

# Rust 环境
set -gx PATH $HOME/.cargo/bin $PATH
```

### 2. 选择合适的运行模式

| 场景 | 推荐模式 | 命令 |
|-----|---------|------|
| 日常开发 (WSL GUI 正常) | WSL 优化模式 | `just dev-wsl` |
| WSL GUI 有问题 | Web 调试模式 | `just dev-web` |
| 远程 SSH | Web 调试模式 | `just dev-web` |
| 前端 UI 测试 | 纯前端模式 | `just dev-web-frontend-only` |
| 性能测试 | 桌面模式 | `just dev` |

### 3. 性能优化建议

1. **项目位置**: 确保项目在 Linux 文件系统 (`~/Projects/`) 而不是 Windows 文件系统 (`/mnt/c/`)
2. **WSL 资源**: 分配足够的内存和 CPU 给 WSL2 (`.wslconfig`)
3. **使用缓存**: 利用 Cargo 的增量编译和 sccache
4. **Web 模式**: 对于频繁的前端开发，使用 Web 调试模式可以避免 Tauri 窗口的开销

### 4. 调试技巧

- **查看环境变量**: `printenv | grep -E "(GDK|MOZ|WEBKIT|LIBGL)"`
- **测试滚轮**: 在启动后立即测试滚轮功能，如果不工作，立即切换到 Web 模式
- **查看进程**: `ps aux | grep -E "(ccr|tauri|webkit)"` 检查进程状态
- **查看日志**: 使用 `RUST_LOG=debug cargo tauri dev` 查看详细日志

---

## 📊 WSL 版本对比

| 特性 | WSL1 | WSL2 |
|-----|------|------|
| GUI 支持 | ❌ 需要 X Server | ✅ 内置 WSLg |
| 性能 | 中等 | 高 |
| CCR Desktop 支持 | ⚠️ 需要配置 | ✅ 原生支持 |
| 推荐使用 | ❌ | ✅ |

**检查 WSL 版本**:

```powershell
# 在 Windows PowerShell 中
wsl --list --verbose
```

输出示例：
```
  NAME      STATE           VERSION
* Ubuntu    Running         2
```

**升级到 WSL2** (如果当前是 WSL1):

```powershell
wsl --set-version Ubuntu 2
```

---

## 🆘 仍然有问题？

如果以上解决方案都不起作用，可以：

1. **查看完整日志**:
   ```bash
   RUST_LOG=debug cargo tauri dev 2>&1 | tee ccr-debug.log
   ```

2. **使用 Web 调试模式作为替代方案**:
   ```bash
   just dev-web
   ```

3. **查看官方文档**:
   - [Tauri WSL 指南](https://tauri.app/v1/guides/getting-started/setup/linux#wsl)
   - [WSLg 文档](https://github.com/microsoft/wslg)

4. **提交 Issue**:
   - [CCR GitHub Issues](https://github.com/harleyqing/ccr/issues)
   - 包含详细的错误信息、系统版本、WSL 版本等

---

**Made with ❤️ by 哈雷酱**

哼，WSL 环境的问题可真是让人头疼呢！(,,><,,)
不过有了本小姐精心准备的这份指南，笨蛋你应该能解决大部分问题了吧！
实在不行的话，就用 Web 调试模式嘛，简单又好用！(￣▽￣)／

