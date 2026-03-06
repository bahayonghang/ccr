# ui - 启动 CCR UI

`ccr ui` 是当前推荐的浏览器入口，用于启动完整 CCR UI 栈。

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

- 用浏览器查看完整模块地图
- 集中管理 skills、sessions、monitoring、statusline、provider health 等 UI 能力
- 本地存在 `ccr-ui/` 时，希望直接走开发版

## 与 `ccr web` 的区别

- `ccr ui`：推荐入口，指向完整 `ccr-ui` 产品面
- `ccr web`：legacy/programmatic path，保留为兼容和脚本场景

参见：[接口选择：ccr ui vs ccr web](/guide/web-guide)
