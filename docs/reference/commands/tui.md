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
| `Enter` / `Space` | 打开应用确认。按 `y` 才写入并留在 TUI。再按 `Enter`、`Space`、`n` 或 `Esc` 取消 |
| `o` | 打开解绑确认。按 `y` 才关闭配置模式。再按 `o`、`n` 或 `Esc` 取消 |
| `y` | 确认当前对话框 |
| `q` / `Esc` | 退出。确认框打开时，`Esc` 只取消确认，下一次 `Esc` 才退出 |

Profile 的 `Enter` / `Space` 和 `o` 都会先确认，包括重新应用当前配置。确认框打开时，方向键、`Tab`、`q` 和鼠标不会改选择、换页或退出。

Claude Auth 与 Codex Auth 的 `Enter` 只在这次按键确实会切换账号时打开确认；`o` 打开退出登录确认。没有可切换账号、已经是当前账号、Codex 虚拟登录和只读注册表仍只显示原来的提示。按 `y` 之后，Claude 切换成功仍退出 TUI；Codex 切换成功仍退出，或接着打开原来的守护进程重启确认。

Grok Auth 页使用账号操作：`s` 保存、`Enter` 确认切换、`d` 删除保存项、`o` 登出全部运行时凭据、`r` 刷新。确认弹窗默认取消，按 `y` 才确认。保存可在 Grok 运行时执行；切换前请自行结束当前 Grok，供新会话使用。后台操作完成前暂时禁止换页和退出，语言切换与 resize 仍可用。详情见 [`grok`](./grok.md)。

## 当前定位

- 适合纯终端环境下的 profile 浏览与切换
- 适合快速在 Claude / Codex / Grok 相关页签之间来回切换
- 不替代 `ccr <command>` 的精确命令面

## 技术事实

- 默认构建启用 `tui` feature
- 入口判断位于 `Cli::is_tui_mode()`
- 无子命令行为位于 `CommandDispatcher::handle_no_subcommand()`
- Grok Auth 页签展示保存账号、选中项和本地会话匹配；实际认证有效性未验证，第三方 profile 保持不变

## 示例

```bash
ccr
# Tab 切平台
# ↑↓ 选配置
# Enter/Space 打开确认，y 才应用并停留（按 q 或 Esc 退出）

ccr grok auth
# s 保存当前账号副本；Enter 打开切换确认
# d 仅删除保存项；o 打开整个运行时的登出确认
```

## 相关页面

- [`grok`](./grok.md)
- [`list`](./list.md)
- [`switch`](./switch.md)
- [`current`](./current.md)
- [`入口选择`](/guide/entrypoints)
