# 技术设计：中英文 README 的 TUI 与 CCR UI 截图介绍

## 1. 范围与产物

本任务形成一个共同验收的文档切片，不拆分父子任务：三张图片必须先完成脱敏，随后才能被两份 README 同时引用。

最终只提交以下产品文件：

- `README.md`
- `README_CN.md`
- `docs/assets/readme/ccr-tui-overview.png`
- `docs/assets/readme/ccr-ui-dashboard.png`
- `docs/assets/readme/ccr-ui-codex-profiles.png`

Trellis 任务文件随任务正常跟踪。不会修改 TUI、CCR UI 或后端源代码。

## 2. 隔离演示环境

### 2.1 构建与运行分离

CLI/TUI 与 CCR Desktop 先在正常开发环境中构建，避免临时 `HOME` 影响 Cargo、Bun 或 WebView2 依赖发现。构建完成后，仅运行截图用子进程时注入隔离环境变量。

### 2.2 子进程隔离边界

为每次采集创建一个工作区外的临时根目录，并把下列路径都指向该目录：

| 环境变量 | 隔离目标 |
| --- | --- |
| `HOME`, `USERPROFILE` | 临时 home |
| `APPDATA`, `LOCALAPPDATA` | 临时 Windows 应用数据目录 |
| `CCR_ROOT`, `CCR_DATA_DIR` | 临时 `.ccr` |
| `CCR_CODEX_DIR` | 临时 `.codex` |
| `CCR_OPENCODE_DIR` | 临时 OpenCode 数据目录 |
| `CLAUDE_CONFIG_DIR`, `CLAUDE_JSON_PATH`, `CCR_SETTINGS_PATH` | 临时 Claude 配置 |
| `CCR_BACKUP_DIR`, `CCR_LOCK_DIR` | 临时备份与锁目录 |
| `LLMUSAGE_HOME` | 临时 llmusage 目录 |
| `WEBVIEW2_USER_DATA_FOLDER` | 临时 WebView2 数据目录 |

截图进程不得继承任何指向真实配置目录的覆盖变量。`PATH` 保留，用于正常发现已安装 CLI；该信息不包含凭据。

### 2.3 演示数据

使用已构建的 `ccr` CLI 在隔离环境中创建少量 Codex Profiles，并切换一个当前 Profile。所有数据满足：

- 域名使用保留的 `.invalid` 后缀。
- token 使用明显的 `sk-demo-*` 虚构值。
- Profile 名称、说明、标签和账号不使用真实组织或个人信息。
- 不写入真实用量数据库，不采集 Usage 独立截图。

CLI 创建完成后，用 `ccr codex profile list --json` 和 `ccr current` 验证隔离数据可读，再启动 TUI/CCR Desktop。

## 3. 截图采集

### 3.1 TUI

- 使用默认 TUI 入口启动已构建的 `ccr`。
- 终端窗口设为 `1440 × 900`，选择 Codex Profile 页和一个有代表性的演示 Profile。
- 画面同时保留 tab、Profile 列表、Runtime/Auth 状态、详情面板和快捷键区。
- 不为了截图修改 Ratatui 渲染代码。

### 3.2 CCR UI

- 先执行前端构建，再以 `custom-protocol` 构建 `ccr-desktop` 调试二进制；最终截图不使用只有空态/限制提示的 Web 预览。
- 在隔离环境中启动桌面二进制，把界面语言切换为英文并将窗口设为 `1440 × 900`。
- Dashboard 截图展示运行状态、下一步和平台概览。
- Codex Profiles 截图展示配置统计、过滤、Profile 列表、上下文侧栏和健康审计。
- 不采集 Usage 页面，不为截图伪造 Tauri IPC 或修改前端代码。

## 4. 脱敏与图像处理

原始截图只存在于临时目录，不直接复制到仓库。最终图片通过 Pillow 完成以下确定性处理：

1. 裁切或补边到精确的 `1440 × 900`。
2. 用与界面背景协调的不透明实心矩形覆盖所有 URL、token/key、账号、私人路径和其他可识别值。
3. 重新编码为 RGB/RGBA PNG，不保留原始 PNG 文本块、EXIF 或其他元数据。
4. 输出到 `docs/assets/readme/` 的三个固定文件名。

即使演示值是虚构的，URL 与 token 仍必须遮挡，以确保最终画面体现真实产品的隐私处理方式。禁止使用模糊或马赛克。

## 5. README 集成

在两份根 README 的 Features/核心特性之后、Quick Start/快速开始之前加入对应章节：

- `README.md`: `## Interface Preview`
- `README_CN.md`: `## 界面预览`

两份 README 按 TUI、CCR UI Dashboard、CCR UI Codex Profiles 的相同顺序引用同一组图片。英文与中文说明信息等价，但不逐字硬译；说明只陈述截图中可验证的当前能力。其余章节只允许做必要的上下文衔接。

## 6. 验证策略

- Pillow 检查三个文件均为 PNG、尺寸均为 `1440 × 900`，并确认无 EXIF/文本元数据。
- RapidOCR 提取可见文本，扫描演示域名、演示 token、URL、真实用户名和本机绝对路径等敏感锚点。
- 使用图像查看工具逐张人工复核遮挡完整性、文字可读性、构图和无重叠问题。
- 使用 Markdown 解析器提取两份 README 的图片引用，确认顺序一致且相对路径全部存在。
- 运行 `just docs-check` 与 `git diff --check`。

## 7. 权衡与回滚

- 采用 PNG 而非 WebP，接受略大的仓库体积以换取终端小字和 UI 文本的无损清晰度。
- 采用桌面调试二进制而非 Web 预览，接受一次额外构建以获得真实 Tauri 数据路径和完整页面状态。
- 不构造 llmusage 数据，避免把 README 截图任务扩大为分析数据库 fixture 工作。
- 回滚时删除三张图片，并从两份 README 移除新增预览章节；临时演示目录与原始截图不进入 Git。

本设计没有引入新的领域术语，也没有难以逆转的架构选择，因此不新增 `CONTEXT.md` 或 ADR。
