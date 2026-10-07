原生混合流复测 PASS。12 组真实 Windows 子进程运行，每组退出 0。stdout 单独捕获、stderr 单独捕获和双流捕获分别覆盖 normal、CLICOLOR_FORCE=1 + NO_COLOR=1、TERM=dumb + FORCE、NO_COLOR。捕获流均使用纯文本状态词；只有非 dumb 的显式 FORCE 保留 ANSI。未捕获流由 Windows ConPTY 实际显示，回执保留目标流符号及状态词。

首次回执 native-stream-pty.json / native-stream-files.json 保留。首次脚本以 .NET SetEnvironmentVariable(null) 设置了空变量；colored 把空 CLICOLOR_FORCE 视为强制颜色。首次 PowerShell 2> 还使用了文本处理。复测以 ProcessStartInfo.Environment.Remove 清除变量，并通过独立管道直接读取原始流。产品代码没有因脚本修正发生变化。首次文本断言遗漏 ANSI reset，失败信息保存在 native-stream-assert-first.txt；去除样式后比较文字的复测通过。
