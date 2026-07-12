# 执行计划：中英文 README 的 TUI 与 CCR UI 截图介绍

## Checklist

1. [x] 读取执行期规范并确认边界。
   - 加载 `trellis-before-dev`。
   - 复核 `.trellis/spec/ccr-tui/backend/index.md`、`.trellis/spec/ccr-ui/frontend/index.md` 与 `.codex/skills/ccr-ui-visual-workflow/SKILL.md`。
   - 确认 Git 变更仅包含当前 Trellis 任务，不覆盖用户的并行改动。

2. [x] 在正常开发环境中构建截图所需二进制。
   - `just build`
   - `cd ccr-ui && bun run build`
   - `cargo build --manifest-path ccr-ui/src-tauri/Cargo.toml --features custom-protocol`
   - 验证 `target/debug/ccr.exe` 与 CCR Desktop 调试二进制实际存在。

3. [x] 创建隔离演示环境并生成 Codex Profiles。
   - 在工作区外创建唯一临时目录，按 `design.md` 2.2 设置截图子进程环境变量。
   - 使用 `ccr codex profile create` 创建 3 个左右的虚构 Profile；域名只用 `.invalid`，token 只用 `sk-demo-*`。
   - 使用 `ccr codex profile switch <demo-name>` 设置当前 Profile。
   - 验证：`ccr codex profile list --json` 与 `ccr current` 只返回演示数据，且真实配置文件时间戳未变化。

4. [x] 采集 TUI 原图。
   - 以隔离环境启动默认 TUI，窗口设为 `1440 × 900`。
   - 切换到 Codex Profile 页并选中代表性 Profile。
   - 采集包含 tabs、Profile 列表、Runtime/Auth、详情与快捷键区的原始 PNG。
   - 原图只保存在临时目录。

5. [x] 采集两张 CCR UI 原图。
   - 以隔离环境启动 `ccr-desktop` 调试二进制，使用独立 `WEBVIEW2_USER_DATA_FOLDER`。
   - 设置英文界面和 `1440 × 900` 窗口。
   - 分别采集 Dashboard 与 `/codex/profiles` 的完整窗口原图。
   - 确认页面没有 Web 预览限制提示，Profiles 页面加载的是隔离演示数据。

6. [x] 生成最终脱敏图片。
   - 用 Pillow 对 URL、token/key、账号和绝对路径绘制不透明实心遮挡。
   - 裁切/补边到 `1440 × 900`，无损重编码并清除元数据。
   - 输出三个固定文件到 `docs/assets/readme/`。
   - 不提交处理脚本、原图或临时演示数据。
   - TUI 当前实现含有两处硬编码中文标签；成品保留真实界面，不在图片中伪造英文翻译。

7. [x] 更新两份根 README。
   - 在 Features/核心特性之后加入 Interface Preview/界面预览章节。
   - 按 TUI、Dashboard、Codex Profiles 顺序引用同一组图片。
   - 中英文说明保持信息等价，并只做必要的上下文衔接。
   - 不修改 `ccr-ui/README*.md`，不扩写安装、迁移、命令或开发章节。

8. [x] 执行资产与隐私验收。
   - Pillow：断言三张图片格式、`1440 × 900` 尺寸和空元数据。
   - RapidOCR：扫描 `.invalid` 域名、`sk-demo-*`、`http://`/`https://`、真实用户名和本机绝对路径等敏感锚点。
   - 图像人工复核：遮挡不可逆、文字清晰、没有重叠或错误裁切。
   - Markdown 解析：两份 README 各有三张目标图片，顺序一致，所有相对路径存在。

9. [x] 运行文档交付门禁。
   - `just docs-check`
   - `git diff --check`
   - `python ./.trellis/scripts/task.py validate .trellis/tasks/07-10-bilingual-readme-tui-ui-screenshots`
    - `git status --short` 只出现任务文件、两份根 README 与三张最终图片。
    - `crates/ccr-tui/src/tui/ui.rs` 可因工作树行尾统计噪声显示为修改；规范化 blob 与 `HEAD` 一致，`git diff` 无内容。

10. [x] 清理并提交规划复核。
    - 关闭 TUI、CCR Desktop 和临时预览进程。
    - 核对临时演示目录的绝对路径位于预期临时根后再递归删除。
    - 最终向用户展示三张成品图、README 差异摘要、OCR/尺寸/门禁结果。

## Review Gates

- Gate A：隔离数据验证通过后才允许启动截图进程。
- Gate B：三张图片通过 OCR 与人工隐私复核后才允许写入 README 引用。
- Gate C：用户审阅本规划并批准后，才运行 `task.py start` 进入实现。

## Rollback

- 图片阶段失败：删除临时原图和未验收的最终图片，不修改 README。
- README 阶段失败：移除新增预览章节并删除三张图片，保留 Trellis 任务记录用于复盘。
- 任何时候都不回滚或清理本任务之外的工作区变更。
