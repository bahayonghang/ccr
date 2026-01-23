# `ccr web` 轻量 Web API 指南

> 提示：浏览器场景推荐使用 `ccr ui`（Vue 3 + Axum + Tauri，全功能界面）。`ccr web` 保留为兼容/自动化用途的轻量 API 服务器，不包含现代 UI。

## 启动
```bash
# 默认地址 0.0.0.0 + 默认端口 19527，自动打开浏览器
ccr web
# 仅本机访问
ccr web --host 127.0.0.1
# 自定义端口
ccr web --port 3000
# 不打开浏览器（CI/远程）
ccr web --no-browser
```
若端口被占用会自动向上递增，日志会提示实际端口。

## 提供的 API（简要）
- 配置：`GET /api/configs`，`POST /api/switch`，`POST /api/config`，`DELETE /api/config/{name}`，`POST /api/validate`
- 历史与备份：`GET /api/history`，`POST /api/clean`，`GET /api/settings/backups`，`POST /api/settings/restore`
- 导入导出：`POST /api/export`，`POST /api/import`
- 系统：`GET /api/system`

## 适用场景
- 脚本/CI：调用 REST 接口完成切换、校验、导出导入。
- 兼容旧流程：需要与早期 Web 接口对接的脚本。
- 避免 UI 依赖：在无桌面或无前端环境下用浏览器/HTTP 调试。

## 安全与限制
- 服务默认绑定 0.0.0.0（内网可访问）；如需仅本机访问请使用 `--host 127.0.0.1`。
- API 参数严格传递，不做 shell 拼接；仍建议在可信环境运行。
- 如需实时输出、可视化同步与平台信息，请使用 `ccr ui`。
