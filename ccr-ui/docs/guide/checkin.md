# 签到管理

中转站签到功能，支持 **30+ 内置提供商**、多账号自动签到、WAF/CF 绕过、CDK 充值、OAuth 引导登录、余额查询和历史记录追踪。

## 功能概览

### 统计卡片
页面顶部显示三个统计卡片：
- **当前余额** 🟢 - 所有账号的总可用余额
- **总额度** 🔵 - 所有账号累计获得的总额度
- **历史消耗** 🟠 - 所有账号累计使用的总消耗

### 账号管理
| 功能 | 说明 |
|------|------|
| 搜索过滤 | 按账号名称或提供商名称搜索 |
| 提供商过滤 | 下拉选择特定提供商 |
| 一键签到 | 对所有启用账号执行签到 |
| 刷新余额 | 批量刷新所有账号余额 |
| CDK 充值 | 自动获取充值码并充值（部分站点） |
| 连接测试 | 测试账号连接是否正常 |

### 账号表格列
- 账号名（含提供商标签）
- 余额（当前可用）
- 总额度（累计获得）
- 历史消耗（累计使用）
- 最后签到时间
- 操作（签到、刷新、充值、编辑、删除）

### 账号 Dashboard
点击账号可查看详细 Dashboard，包括：
- 连续签到统计（当前连续天数、最长连续天数）
- 月度签到日历（可视化签到记录）
- 额度趋势图表（余额变化走势）
- 月度统计（签到率、额度增量）

## 内置提供商

CCR UI 内置 **30+ 个** NewAPI 公益站提供商，按功能分类：

### 标准站点（24 个）
直接支持签到，无需特殊绕过：

| 提供商 | 域名 | 签到路径 |
|--------|------|----------|
| Wong | wzw.pp.ua | `/api/user/checkin` |
| Huan666 | ai.huan666.de | `/api/user/checkin` |
| KFC | kfc-api.sxxe.net | `/api/user/checkin` |
| Neb | ai.zzhdsgsss.xyz | `/api/user/checkin` |
| LightLLM | lightllm.online | `/api/user/checkin` |
| TakeAPI | codex.661118.xyz | `/api/user/checkin` |
| ThatAPI | gyapi.zxiaoruan.cn | `/api/user/checkin` |
| DuckCoding | duckcoding.com | `/api/user/checkin` |
| Free DuckCoding | free.duckcoding.com | `/api/user/checkin` |
| Taizi | api.codeme.me | `/api/user/checkin` |
| OpenAI Test | openai.api-test.us.ci | `/api/user/checkin` |
| ChengTX | api.chengtx.vip | `/api/user/checkin` |
| Codex.cab | codex.cab | `/api/user/checkin` |
| Clove | clove.cc.cd | `/api/user/checkin` |
| NPCodex | npcodex.kiroxubei.tech | `/api/user/checkin` |
| MuAPI | ai.muapi.cn | `/api/user/checkin` |
| Feisakura | api.feisakura.fun | `/api/user/checkin` |
| Xionger | api.xionger.ccwu.cc | `/api/user/checkin` |
| Einzieg | api.einzieg.site | `/api/user/checkin` |
| 2020111 | api.2020111.xyz | `/api/user/checkin` |
| 361888 | api.361888.xyz | `/api/user/checkin` |
| YYDS | yyds.215.im | `/api/user/checkin` |
| Anthorpic | anthorpic.us.ci | `/api/user/checkin` |
| Nanohajimi | free.nanohajimi.mom | `/api/user/checkin` |

### 需 WAF 绕过（1 个）
需要阿里云 WAF Cookie 绕过：
- **AnyRouter** - `anyrouter.top`（自动浏览器绕过）

### 需 CF Clearance 绕过（4 个）
需要 Cloudflare 挑战绕过：
- **RunAnytime** - `runanytime.hxi.me`（还支持 CDK 充值）
- **Elysiver** - `elysiver.h-e.top`
- **Hotaru** - `hotaruapi.com`
- **B4U** - `b4u.qzz.io`（还支持 CDK 充值）

### 特殊站点（2 个）
- **AgentRouter** - `agentrouter.org`（查询用户信息时自动签到）
- **CodeRouter** - `api.codemirror.codes`（仅查询余额，无签到）

## 快速开始

### 1. 添加提供商
在「提供商」Tab 中：
- 点击内置提供商卡片快速添加（推荐，按分类浏览）
- 或手动创建自定义提供商

### 2. 添加账号
在「账号管理」Tab 中：
1. 点击「添加账号」
2. 选择提供商
3. 输入账号名称
4. 粘贴 Cookies JSON（格式：`{"session": "xxx", "token": "yyy"}`）
5. 可选：填写 API User（New-Api-User 请求头值）

**OAuth 引导登录**（推荐）：
1. 点击「OAuth 登录」按钮
2. 选择提供商和 OAuth 方式（GitHub / LinuxDo）
3. 在浏览器中完成授权
4. 粘贴获取的 Cookies
5. 系统自动创建账号

### 3. 执行签到
- **批量签到**：点击右上角「一键签到」
- **单个签到**：点击账号行的「签到」按钮

### 4. 查看余额
- 首次需点击「刷新余额」获取数据
- 余额数据来自提供商 API `/api/user/self`

### 5. CDK 充值（部分站点）
支持 CDK 充值的站点（RunAnytime、B4U、x666）：
- 签到后自动获取充值码并充值
- 也可手动点击「充值」按钮触发

## API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/checkin/providers` | 获取提供商列表 |
| GET | `/api/checkin/providers/builtin` | 获取内置提供商列表 |
| POST | `/api/checkin/providers/builtin/add` | 添加内置提供商 |
| POST | `/api/checkin/providers` | 创建自定义提供商 |
| GET | `/api/checkin/accounts` | 获取账号列表 |
| POST | `/api/checkin/accounts` | 创建账号 |
| GET | `/api/checkin/accounts/:id/dashboard` | 获取账号 Dashboard |
| POST | `/api/checkin/execute` | 批量签到 |
| POST | `/api/checkin/accounts/:id/checkin` | 单个签到 |
| POST | `/api/checkin/accounts/:id/balance` | 刷新余额 |
| GET | `/api/checkin/accounts/:id/balance/history` | 余额历史 |
| POST | `/api/checkin/accounts/:id/topup` | CDK 充值 |
| POST | `/api/checkin/accounts/:id/test` | 连接测试 |
| GET | `/api/checkin/records` | 获取签到记录 |
| GET | `/api/checkin/records/export` | 导出签到记录 |
| GET | `/api/checkin/stats/today` | 今日统计 |
| POST | `/api/checkin/export` | 导出配置 |
| POST | `/api/checkin/import/preview` | 预览导入 |
| POST | `/api/checkin/import` | 执行导入 |
| POST | `/api/checkin/oauth/authorize-url` | 获取 OAuth 授权链接 |

## 数据安全

- Cookies 使用 **AES-256-GCM** 加密存储
- CDK 凭证通过 `extra_config` 字段加密存储
- 数据存储在 SQLite 数据库 `~/.ccr-ui/checkin.db`

## 相关文档

- [签到功能详细指南](./checkin-detailed.md) - 完整使用说明
- [后端 API 文档](../reference/backend/api.md) - API 端点详情
