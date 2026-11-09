# 原生 Linux 验收

2026-09-29 UTC。attempt6 的六项检查通过，构建、资源、源码与脚本 SHA 归属已核对。

- 原生 tauri://localhost 启动及明文确认通过。
- 确认弹窗滚动锁样式具有页面 nonce；CodeMirror 为 13px、448px，style CSP 无违规。
- 无 nonce 的内联脚本被生产 CSP 拒绝。
- 合成 Claude settings 仅将 model 从 sonnet 改为 haiku；env、未知对象及数值均精确保留。
- 保存后 runtime styles 卸载；WebDriver 会话和进程组清理完成。
- root 已查看截图。截图记录保存前编辑器，磁盘往返由结构化运行记录证明。

适用范围：WSLg Linux 原生 WebKitGTK，debug 二进制嵌入生产 custom-protocol 资源/CSP。环境重定向不构成 OS 文件沙箱。未覆盖 Windows WebView2、macOS、物理输入、发布安装包、真实 SSH 或 OAuth。attempt1–5 失败记录均保留。

证据及 SHA 见 remaining-native-acceptance.json。
