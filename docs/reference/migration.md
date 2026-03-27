# 迁移指南

本文只说明当前仓库布局和入口选择如何对应旧文档中的历史说法，不把旧命令当成当前支持面。

## 入口映射

| 历史说法 | 当前做法 | 说明 |
|---|---|---|
| `ccr web` | `ccr ui` | 图形入口已独立为 `ccr ui` + `ccr-ui` 工程 |
| 内置 Web API | 无直接替代 | 当前 UI 通过 `ccr-ui/src-tauri` 复用 crate，不再暴露内置 HTTP 路由 |
| `ccr tui` | 直接运行 `ccr` | 默认构建下，无子命令时进入 TUI |
| `ccr migrate` | 手动初始化当前布局，再导入或重建 profile | 当前命令面不再文档化单独的迁移子命令 |

## 路径映射

| 旧路径 | 当前路径 | 说明 |
|---|---|---|
| `src/` | `crates/ccr/src/` | 主 CLI crate |
| `tests/` | `crates/ccr/tests/` | CLI 集成测试 |
| `ccr-db/` | `crates/ccr-db/` | 数据库与桌面侧服务 |
| `ccr-types/` | `crates/ccr-types/` | 共享类型 |
| `ccr-ui/backend/` | `ccr-ui/src-tauri/` | Tauri 桌面壳 |
| `ccr-ui/frontend/` | `ccr-ui/src/` | Vue 前端源码 |

## 迁移到当前工作区的建议顺序

1. 用当前入口重新建立基本目录：

```bash
ccr init
ccr platform list
```

2. 如需图形入口，使用：

```bash
ccr ui
```

3. 如果你手上已经有可导入的 profile 文件，用：

```bash
ccr import <file> --merge --backup
```

4. 如果你只有旧配置文件而没有现成导入包，把旧文件当参考源，逐个平台重建 profile。

## 保留与移除

- 保留：Legacy 配置文件、旧路径、旧入口的历史背景说明
- 移除：把旧 Web API、`ccr web`、`ccr migrate` 当成当前可执行文档入口

## 相关页面

- [架构设计](/reference/architecture)
- [Crate 地图](/reference/internals/crate-map)
- [命令参考](/reference/commands/)
- [入口选择](/guide/entrypoints)
