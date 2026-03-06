# web - Legacy Web API

`ccr web` 启动内置的轻量 HTTP 服务。它的定位是兼容旧流程和编程式访问，而不是主图形界面。

## 用法

```bash
ccr web [--host <host>] [--port <port>] [--no-browser]
```

## 默认值

- Host：`127.0.0.1`
- Port：`19527`

## 常见示例

```bash
# 默认仅本机监听
ccr web

# 显式指定 host / port
ccr web --host 127.0.0.1 --port 19527 --no-browser

# 在可信内网暴露
ccr web --host 0.0.0.0 --port 19527 --no-browser
```

## 适用场景

- `curl`、CI、脚本调用
- 兼容旧版 Web/HTTP 使用方式
- 需要轻量 API，但不需要完整 `ccr-ui`

## 不再承担的职责

以下内容不应再由 `ccr web` 文档承担：

- “现代完整 Web 界面”的产品叙事
- `ccr-ui` 页面地图
- 桌面壳或前端模块说明

这些内容统一放在 [UI 概览](/guide/ui-overview) 和 [UI 模块地图](/guide/ui-modules)。

## 相关文档

- [Web API 参考](/reference/api)
- [接口选择：ccr ui vs ccr web](/guide/web-guide)
