# ui - 启动 CCR UI 全量界面

启动完整的 CCR UI（前端 Vue3+Vite+Pinia+Tauri，后端 Axum）。命令会自动检测或下载 UI 资源。

**支持版本**：v1.4.0+

## 用法

```bash
ccr ui [-p <frontend-port>] [--backend-port <port>]
```

- `-p, --port`：前端端口，默认 `3000`
- `--backend-port`：后端端口，默认 `38081`

## 启动顺序

1) `./ccr-ui/` 或 `../ccr-ui/`（开发者本地源码）  
2) `~/.ccr/ccr-ui/`（用户目录缓存）  
3) 询问是否从 GitHub 下载并解压（首次使用）

## 典型流程

```bash
ccr ui                     # 自动检测或下载，端口默认 3000/38081
ccr ui -p 3100 --backend-port 39000
```

> 如果需要浏览器内使用轻量 API，请使用 `ccr web`；`ccr ui` 提供完整图形界面与命令执行、同步、成本等能力。
