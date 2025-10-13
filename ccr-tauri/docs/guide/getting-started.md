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

```bash
cargo tauri dev

# 或使用 justfile
just dev
```

第一次运行需要编译 Rust 代码，可能需要几分钟时间。后续启动会快很多！(^_^)b

::: warning 首次启动较慢
Rust 的首次编译需要下载并编译所有依赖，这可能需要 5-10 分钟。请耐心等待～
:::

## 验证安装

成功启动后，你应该看到：

1. **Vite 开发服务器** 在 `http://localhost:5173` 启动
2. **Tauri 应用窗口** 自动打开，显示 CCR Desktop 界面
3. **终端日志** 显示编译和运行信息

::: details 查看示例输出
```bash
   Compiling ccr-tauri v1.1.2 (/path/to/ccr/ccr-tauri)
    Finished dev [unoptimized + debuginfo] target(s) in 2m 15s
     Running `target/debug/ccr-tauri`

  VITE v5.0.0  ready in 324 ms

  ➜  Local:   http://localhost:5173/
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

## 主题切换

点击右上角的 **🌙/☀️** 图标，在深色和浅色主题间切换。

主题偏好会自动保存到本地存储，下次启动时自动应用。

## 下一步

恭喜！你已经成功运行了 CCR Desktop！(￣▽￣)／

现在你可以：

- 📚 阅读 [架构设计](/architecture/overview) 了解内部实现
- 🛠️ 查看 [开发指南](/development/structure) 学习如何添加新功能
- 🔧 配置 [Tauri 权限](/config/permissions) 自定义文件访问范围
- 📦 学习如何 [构建发布版本](/guide/build)

::: tip 提示
遇到问题？查看 [常见问题](/troubleshooting/faq) 或 [调试技巧](/development/debugging)
:::

---

**Made with ❤️ by 哈雷酱**

哼，这么简单的教程本小姐都写得这么详细了，笨蛋你要是还搞不定的话...(,,><,,)
