# 公开资料研究与采用决定

## 检索范围

2026-10-06 按 web-access 指南检索并读取一手指南、标准和官方源码。网络检索仅使用通用 CLI 关键词，没有发送账号、邮箱、凭据、本地路径或私有源码。

## 来源与采用机制

| 来源 | 核实内容 | CCR 决定 |
| --- | --- | --- |
| [CLI Guidelines：Output](https://clig.dev/#output)、[Basics](https://clig.dev/#the-basics) | 克制用色，给状态相关建议，考虑 TTY，区分数据与消息。 | 结果、字段和建议分层；保留现有通道，不批量迁移 stderr。 |
| [NO_COLOR 标准](https://no-color.org/) | 非空变量请求关闭自动颜色；颜色控制与终端能力分开。 | 关闭样式但保留状态字符；去掉全部 ANSI 是 CCR 产品选择。 |
| [GitHub CLI auth/status](https://github.com/cli/cli/blob/trunk/pkg/cmd/auth/status/status.go) | 账号主行与下方字段分开，恢复命令取决于状态，JSON 单独导出。 | 采用简洁主行、缩进字段与相关建议，不复制退出码或认证架构。 |
| [GitHub CLI ColorScheme](https://github.com/cli/cli/blob/trunk/pkg/iostreams/color.go) | 成功、警告、失败标记集中定义，颜色为附加表现。 | 统一小型标记，移除重复图标，不增加主题设置。 |
| [Vite logger](https://github.com/vitejs/vite/blob/main/packages/vite/src/node/logger.ts) | 默认 info 可直接输出正文，清屏受终端条件约束，服务地址按字段分层。 | 普通说明不加等级；不引入清屏、重复计数或 dev-server 功能。 |
| [colored v3.1.1 control](https://github.com/colored-rs/colored/blob/v3.1.1/src/control.rs) | 环境变量集中处理，显式强制优先，默认检测 stdout，提供全局 override。 | 复用颜色策略，字符检测使用标准库，不逐条修改全局状态。 |

采用决定为设计推论，来源不规定 CCR 的具体版式。仅保存简短分析和链接，不复制来源全文。

## 设计推论

结果使用动作与对象，字段使用键值行，统计和说明采用中性表达。状态字符携带语义，颜色作为补充。建议采用现有 handler 条件，成功查看结果，失败恢复操作；命令与解释分行。机器格式与真实终端分别验收。

## 未采用方案

- 只将标签换成 emoji：字段和建议仍缺乏层次。
- 每种信息设置颜色：增加颜色噪声与明暗背景风险。
- 全部消息迁移 stderr：改变现有接口，需要独立授权。
- 引入事件总线、通用消息系统或主题：现有入口足以完成目标。
- 每次列出全部命令：形成重复帮助目录，增加长度。

## 版本与证据

colored 固定 v3.1.1；GitHub CLI 与 Vite 使用研究时点 trunk/main。任务只借鉴机制，不调用这些项目 API。公开来源没有验证 CCR 终端、真实账号、其他 OS 或 hosted CI。
