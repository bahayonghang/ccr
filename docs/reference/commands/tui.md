# TUI - 终端交互模式

本页说明当前默认构建下的终端交互模式，不对应独立的 `ccr tui` 子命令。

## 进入方式

```bash
# 默认方式：无子命令启动
ccr
```

补充行为：

```bash
# Codex 路径下，无 action 时也会进入 TUI
ccr codex

# Grok Auth 路径下，无嵌套动作时进入 Grok Auth 页签
ccr grok auth
```

## 键盘操作

| 按键 | 功能 |
|---|---|
| `Tab` | 在可用页签之间切换 |
| `←` / `→` / `h` / `l` | 翻页 |
| `↑` / `↓` / `j` / `k` | 选择配置 |
| `Enter` / `Space` | 应用选中配置并保持在 TUI 内（结果显示在 Focus 面板） |
| `q` / `Esc` | 退出 |

## 当前定位

- 适合纯终端环境下的 profile 浏览与切换
- 适合快速在 Claude / Codex / Grok 相关页签之间来回切换
- 不替代 `ccr <command>` 的精确命令面

## 技术事实

- 默认构建启用 `tui` feature
- 入口判断位于 `Cli::is_tui_mode()`
- 无子命令行为位于 `CommandDispatcher::handle_no_subcommand()`
- Grok Auth 页签只展示官方会话状态，并支持 `o` 登出官方运行时登录

## 示例

```bash
ccr
# Tab 切平台
# ↑↓ 选配置
# Enter/Space 应用并停留（按 q 或 Esc 退出）

ccr grok auth
# 在 Grok Auth 页签按 o，登出当前官方运行时登录
```

## 相关页面

- [`grok`](./grok.md)
- [`list`](./list.md)
- [`switch`](./switch.md)
- [`current`](./current.md)
- [`入口选择`](/guide/entrypoints)
