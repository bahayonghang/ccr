# GPUI 桌面端

CCR 的原生桌面应用，在一个窗口里管理各编码工具的接入、定制和用量。本文件只收录这个界面里的用语。

## 页面结构

**平台目的地**（Platform destination）：
侧栏「平台」组里的一项，如 Claude Code、Codex。它只呈现这一个平台的内容。
_Avoid_: 平台页

**Claude Code 页面**：
Claude Code 这个平台目的地，包括它的全部平台 tab。
_Avoid_: Claude 首页（指工作台时）

**平台 tab**（Platform tab）：
平台目的地里的一个工具页。一个 tab 对应一个页面，选中哪个 tab，页面就只显示那一项。
_Avoid_: lens（代码里的名字）、子页

**工作台**（Workbench）：
平台目的地的第一个 tab，概括这个平台的当前状态和用量。
_Avoid_: 首页、主页、Home

**运行概览**（Overview）：
应用的默认首页，汇总所有平台。
_Avoid_: 仪表盘、Dashboard、首页

## 连接与身份

**配置方案**（Profile）：
一组有名字、可以切换的接入参数，如端点、凭据和模型。平台一次只用其中一个。
_Avoid_: 配置（单独使用时）、配置集、界面中文里的「Profiles」

**账户**（Account）：
保存下来的平台登录凭据，可以在不同账户之间切换。
_Avoid_: 认证（指这个对象时）、Authentication

**设置**（Settings）：
平台自己的设置文件，如 Claude Code 的 settings.json。
_Avoid_: 设置与来源

**配置来源**（Configuration sources）：
一项设置的生效值来自哪几层文件。
_Avoid_: 来源（单独使用时）

## 定制

**指令**（Instructions）：
告诉平台该怎么做的文本，包括指令文件、规则和输出样式。
_Avoid_: 资源、系统提示词

**指令文件**（Instruction files）：
平台启动时读取的说明文件，如 CLAUDE.md、AGENTS.md。
_Avoid_: System Prompts、系统提示词

**规则**（Rules）：
拆成多个文件、按主题组织的补充指令。

**输出样式**（Output styles）：
改变回复风格的预设指令。

**扩展**（Extensions）：
给平台增加能力的条目。Claude Code 的扩展是子代理、斜杠命令和插件，其他平台各有不同。
_Avoid_: 资源、代理与来源、代理与插件

**子代理**（Subagent）：
Claude Code 可以把任务委派出去的专用代理。
_Avoid_: Agents、代理配置（容易和 Agent 会话混淆）

**斜杠命令**（Slash command）：
在 Claude Code 会话里用 `/名称` 调用的命令。
_Avoid_: Commands、命令（侧栏的「命令」指运行 CCR 命令）

**插件**（Plugin）：
打包分发的一组扩展。
_Avoid_: 插件配置

## 活动

**Agent 会话**（Agent session）：
Claude Code、Codex 等编码工具留下的会话记录。这里的 agent 指编码工具本身，不是子代理。
_Avoid_: 会话（跨平台说的时候）

**记录费用**（Recorded cost）：
llmusage 按 API 价目估算的金额，不是账单，也不是实际支付的金额。
_Avoid_: 花费、账单、实付

**订阅设置**（Subscription settings）：
用户填写的订阅模式、套餐和月费，用来和记录费用对比。
_Avoid_: 订阅（只指这些设置时）
