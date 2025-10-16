# TUI - 交互式终端界面

启动 CCR 的交互式终端用户界面（TUI），提供可视化配置管理功能。

## 基本用法

```bash
ccr tui [OPTIONS]
```

## 选项

| 选项 | 简写 | 说明 |
|------|------|------|
| `--yolo` | - | 启动时启用 YOLO 模式，跳过所有确认提示 |

## 功能概述

TUI 提供了三个功能齐全的标签页，用于管理 Claude Code 配置：

### 🖥️ 三个标签页

#### 1. 配置页（Configs Tab）

配置管理的核心界面：

**功能：**
- 📋 浏览所有可用配置
- ▶️ 切换到选中的配置
- 🗑️ 删除配置（需 YOLO 模式）
- 🎨 彩色编码标识：
  - **绿色 + ▶ 标记**：当前使用的配置
  - **青色 + ⭐ 标记**：默认配置
  - **白色**：其他配置

**操作：**
- 使用 `↑↓` 或 `j`/`k` 导航列表
- 按 `Enter` 切换到选中的配置
- 按 `d` 删除选中的配置（需要 YOLO 模式）

**显示信息：**
- 配置名称
- 配置描述
- 当前/默认状态标记

#### 2. 历史页（History Tab）

查看操作审计记录：

**功能：**
- 📜 显示最近 50 条操作历史
- ⏱️ 时间戳信息（月-日 时:分:秒）
- 🎯 操作类型（switch/backup/validate 等）
- 📊 结果指示器：
  - ✅ **绿色**：操作成功
  - ❌ **红色**：操作失败
  - ⚠️ **黄色**：警告

**显示格式：**
```
10-17 14:32:15 ✅ switch → anthropic
10-17 14:30:45 ✅ backup → N/A
10-17 14:25:10 ⚠️ validate → N/A
```

#### 3. 系统页（System Tab）

查看系统信息和配置：

**系统信息：**
- 🖥️ 主机名
- 👤 用户名
- 💻 操作系统

**CCR 信息：**
- 📦 版本号
- ▶️ 当前配置
- ⚡ YOLO 模式状态

**文件路径：**
- ⚙️ 配置文件：`~/.ccs_config.toml`
- 🎯 设置文件：`~/.claude/settings.json`
- 📚 历史文件：`~/.claude/ccr_history.json`

### ⌨️ 键盘快捷键

#### 标签页切换
- `1`：跳转到配置页
- `2`：跳转到历史页
- `3`：跳转到系统页
- `Tab`：切换到下一个标签页
- `Shift+Tab`：切换到上一个标签页

#### 列表导航
- `↑` / `k`：向上移动
- `↓` / `j`：向下移动（支持 Vim 风格）

#### 操作快捷键
- `Enter`：执行操作（切换配置）
- `d` / `D`：删除选中的配置（需 YOLO 模式）
- `y` / `Y`：切换 YOLO 模式
- `q` / `Q`：退出 TUI
- `Ctrl+C`：强制退出

### ⚡ YOLO 模式

YOLO (You Only Live Once) 模式用于跳过确认提示，提高操作效率。

**特性：**
- 🚨 跳过所有危险操作的确认提示
- 🗑️ TUI 中删除配置操作必须启用此模式
- 🔄 运行时可随时切换（按 `Y` 键）
- 📊 页脚实时显示状态：
  - 🔴 **YOLO** (红色) - 已启用
  - 🟢 **SAFE** (绿色) - 已禁用

**启用方式：**

1. **启动时启用：**
```bash
ccr tui --yolo
```

2. **运行时切换：**
```
按 Y 键 → 切换 YOLO 模式状态
```

::: warning 危险操作警告
YOLO 模式会跳过所有确认，使用时请务必小心！特别是在删除配置时。
:::

### 🎨 视觉特性

**彩色编码：**
- **绿色**：成功状态、当前配置
- **青色**：默认配置
- **红色**：错误消息、YOLO 模式启用
- **黄色**：警告消息、高亮选中项
- **白色**：普通配置和文本

**实时反馈：**
- ✅ 操作成功：绿色加粗消息
- ❌ 操作失败：红色加粗消息
- 💡 状态提示：实时显示在页脚上方

**标记符号：**
- ▶ 当前配置
- ⭐ 默认配置
- ✅ 成功操作
- ❌ 失败操作
- ⚠️ 警告信息
- 🔴 YOLO 模式启用
- 🟢 安全模式

## 使用示例

### 示例 1：日常配置切换

```bash
# 启动 TUI
ccr tui

# 在 TUI 中：
1. 按 '1' 进入配置页
2. 使用 ↑↓ 或 j/k 浏览配置列表
3. 选中需要的配置后按 Enter
4. 查看页脚的成功消息
5. 按 'q' 退出
```

### 示例 2：查看操作历史

```bash
# 启动 TUI
ccr tui

# 在 TUI 中：
1. 按 '2' 进入历史页
2. 使用 ↑↓ 或 j/k 浏览历史记录
3. 查看操作结果（✅ ❌ ⚠️）
4. 按 'q' 退出
```

### 示例 3：删除配置（YOLO 模式）

```bash
# 方式 1：启动时启用 YOLO 模式
ccr tui --yolo

# 在 TUI 中：
1. 按 '1' 进入配置页
2. 选择要删除的配置
3. 按 'd' 删除
4. 查看页脚的确认消息

# 方式 2：运行时启用 YOLO 模式
ccr tui

# 在 TUI 中：
1. 按 'Y' 启用 YOLO 模式
2. 页脚显示 🔴 YOLO 状态
3. 按 '1' 进入配置页
4. 选择要删除的配置
5. 按 'd' 删除
```

### 示例 4：查看系统信息

```bash
# 启动 TUI
ccr tui

# 在 TUI 中：
1. 按 '3' 进入系统页
2. 查看系统信息（主机名、用户、OS）
3. 查看 CCR 信息（版本、当前配置）
4. 查看文件路径
5. 按 'q' 退出
```

## 工作流建议

### 快速切换配置

```bash
ccr tui
# 1 → ↓↓ → Enter → q
# 仅需 4 步完成配置切换
```

### 检查配置和历史

```bash
ccr tui
# 1 → 查看配置列表
# 2 → 查看操作历史
# 3 → 查看系统信息
# q → 退出
```

### 批量管理配置

```bash
ccr tui --yolo
# Y → 启用 YOLO 模式
# 1 → 进入配置页
# ↓ → d → 删除配置 1
# ↓ → d → 删除配置 2
# Y → 禁用 YOLO 模式
# q → 退出
```

## 操作安全

### 当前配置保护

TUI 不允许删除当前正在使用的配置：

```
状态消息：❌ Cannot delete current config: anthropic
```

### 默认配置保护

TUI 不允许删除默认配置：

```
状态消息：❌ Cannot delete default config: anthropic
```

### YOLO 模式要求

删除操作必须启用 YOLO 模式：

```
状态消息：⚠️ YOLO mode required to delete configs in TUI (Press Y)
```

## 性能特点

- **响应速度**：250ms 事件轮询周期，确保流畅交互
- **资源占用**：轻量级终端界面，CPU 和内存占用极低
- **并发安全**：继承 CCR 核心的文件锁机制
- **数据实时性**：每次渲染读取最新配置和历史数据

## 故障排除

### 终端显示异常

**问题：** TUI 界面显示混乱或颜色错误

**解决：**
```bash
# 检查终端支持
echo $TERM

# 使用支持的终端（推荐）
# - Linux: GNOME Terminal, Konsole, Alacritty
# - macOS: iTerm2, Terminal.app
# - Windows: Windows Terminal, ConEmu

# 如果问题持续，尝试：
export TERM=xterm-256color
ccr tui
```

### 键盘快捷键不工作

**问题：** 按键没有响应或响应错误

**解决：**
```bash
# 检查终端模式
stty -a

# 重置终端状态
reset
ccr tui
```

### TUI 无法启动

**问题：** 启动失败或崩溃

**解决：**
```bash
# 检查配置文件
ccr validate

# 查看详细错误信息
CCR_LOG_LEVEL=debug ccr tui

# 如果配置文件损坏，重新初始化
mv ~/.ccs_config.toml ~/.ccs_config.toml.backup
ccr init
```

### 退出后终端混乱

**问题：** 退出 TUI 后终端显示异常

**解决：**
```bash
# 重置终端
reset

# 或清空屏幕
clear
```

## TUI vs CLI vs Web

选择合适的界面：

| 使用场景 | 推荐界面 | 原因 |
|----------|----------|------|
| 日常交互管理 | **TUI** | 可视化、键盘导航、实时反馈 |
| 脚本和自动化 | **CLI** | 命令行友好、易于集成 |
| 远程管理 | **Web** | HTTP API、跨平台访问 |
| 批量操作 | **CLI** | 配合 shell 脚本效率高 |
| 学习和探索 | **TUI** | 直观、交互式、有提示 |

## 技术细节

### 架构

TUI 基于以下技术栈：

- **ratatui 0.29**: 现代 Rust TUI 框架
- **crossterm 0.28**: 跨平台终端操作库
- **事件驱动**: 250ms tick 事件循环
- **状态机模式**: TabState 枚举管理导航
- **服务层集成**: 复用 ConfigService/HistoryService/SettingsService

### 事件循环

```rust
loop {
    // 渲染 UI
    terminal.draw(|f| ui::draw(f, &mut app))?;

    // 处理事件
    match event_handler.next()? {
        Event::Key(key) => {
            if app.handle_key(key)? {
                break; // 退出
            }
        }
        Event::Tick => {
            // 定时刷新
        }
    }
}
```

### 状态管理

```rust
pub struct App {
    pub current_tab: TabState,           // 当前标签页
    pub yolo_mode: bool,                 // YOLO 模式
    pub config_list_index: usize,        // 配置列表索引
    pub history_list_index: usize,       // 历史列表索引
    pub status_message: Option<String>,  // 状态消息
    // ... services
}
```

## 相关命令

- [`list`](./list.md) - 以表格形式列出配置
- [`current`](./current.md) - 显示当前配置
- [`switch`](./switch.md) - 切换配置
- [`history`](./history.md) - 查看操作历史
- [`web`](./web.md) - 启动 Web 界面

## 下一步

- 查看 [快速开始](/quick-start) 了解基本用法
- 查看 [配置管理](/configuration) 了解配置选项
- 查看 [架构文档](/architecture) 了解实现细节
