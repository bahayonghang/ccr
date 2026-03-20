# ui - 启动 CCR UI

`ccr ui` 是当前推荐的图形入口，用于启动完整 CCR UI 栈。

## 用法

```bash
ccr ui [-p <frontend-port>] [--backend-port <port>]
ccr ui update
ccr ui help
```

## 默认值

- 前端端口：`15173`
- 后端端口：`38081`

## 启动顺序

1. 当前目录或父目录中的 `ccr-ui/`
2. `~/.ccr/ccr-ui/`
3. 首次使用时提示从 GitHub 下载

## 常见示例

```bash
ccr ui
ccr ui -p 15173 --backend-port 38081
ccr ui update
```

## 适用场景

- 用图形界面查看完整模块地图
- 集中管理 skills、monitoring、statusline 等 UI 能力
- 本地存在 `ccr-ui/` 时，希望直接走开发版

