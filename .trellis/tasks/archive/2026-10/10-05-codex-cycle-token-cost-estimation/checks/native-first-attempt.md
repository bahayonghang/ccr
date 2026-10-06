# 原生首次启动记录

2026-10-06，最终 release 二进制，Windows 原生 PTY，合成隔离目录。

实际窗口：140×40。工具会话继承 `TERM=dumb`，产品终端能力检查拒绝启动；未进入 TUI，未计为界面 PASS。

```text
[ERR] The current terminal lacks the capabilities required by the TUI (TERM=dumb).
Use a terminal with ANSI and cursor control support, or use `ccr list` / `ccr current` instead.
Native process exit: 74
```

该 PTY 支持 ANSI cursor control；窗口 resize 为140×40已成功。复测仅在帮助脚本设置并恢复进程级 `TERM=xterm-256color`，另记录 stdin/stdout 是否重定向；未修改产品终端检查或正式门槛。
