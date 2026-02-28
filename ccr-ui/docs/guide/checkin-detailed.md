# 签到功能详细指南

> **版本**: v4.0+
> **最后更新**: 2026-02-13
> **功能状态**: ✅ 稳定

AI 中转站自动签到功能，支持 30+ 内置提供商、多账号管理、WAF/CF 绕过、CDK 充值、OAuth 引导登录，自动签到获取积分/额度。

## 📋 目录

- [功能概述](#功能概述)
- [核心概念](#核心概念)
- [快速开始](#快速开始)
- [提供商管理](#提供商管理)
- [账号管理](#账号管理)
- [签到操作](#签到操作)
- [余额查询](#余额查询)
- [WAF 绕过](#waf-绕过阿里云-waf)
- [CF Clearance 绕过](#cf-clearance-绕过cloudflare)
- [CDK 充值](#cdk-充值)
- [账号 Dashboard](#账号-dashboard)
- [数据管理](#数据管理)
- [常见问题](#常见问题)

---

## 🌟 功能概述

### 核心能力

| 功能 | 说明 |
|------|------|
| 📋 多提供商支持 | 30+ 内置提供商，按分类管理（标准/WAF/CF/特殊） |
| 👥 多账号管理 | 每个提供商可配置多个账号，独立管理 |
| 🔄 自动签到 | 一键批量签到所有账号，支持单账号签到 |
| 💰 余额查询 | 实时查询账号余额、配额、消耗量 |
| 📊 账号 Dashboard | 连续签到统计、月历日历、余额趋势图 |
| 🛡️ WAF 绕过 | 自动处理阿里云 WAF 防护（AnyRouter） |
| ☁️ CF Clearance 绕过 | 自动处理 Cloudflare 防护（4 个站点） |
| 🎁 CDK 充值 | 自动获取并兑换充值码（3 个站点） |
| 🔑 OAuth 引导登录 | GitHub/LinuxDo OAuth 向导简化账号配置 |
| 📦 导入/导出 | 完整配置备份与恢复，冲突策略可选 |
| 🔒 加密存储 | AES-256-GCM 加密 Cookie，SQLite 本地存储 |
| 🧪 连接测试 | 验证账号连通性，确保配置正确 |

### 统计卡片

签到页面顶部显示三个核心统计卡片：

- **当前余额** 🟢 — 所有账号的可用余额总和
- **总配额** 🔵 — 所有账号累计获得的配额
- **历史消耗** 🟠 — 所有账号累计使用量

---

## 📖 核心概念

### 提供商 (Provider)

提供商是 AI 中转站的服务配置，包含域名、API 路径、认证方式等信息。

**提供商分类：**

| 分类 | 说明 | 数量 |
|------|------|------|
| `standard` | 标准 NewAPI 站点，无特殊绕过 | 24 |
| `waf_required` | 需要阿里云 WAF 绕过 | 1 |
| `cf_required` | 需要 Cloudflare Clearance 绕过 | 4 |
| `special` | 特殊签到机制 | 2 |

### 账号 (Account)

账号是在某个提供商下的用户凭证，包含 Cookie、API User 等认证信息。

- 每个提供商可配置多个账号
- Cookie 使用 **AES-256-GCM** 加密存储
- 支持 `extra_config` 扩展字段（如 CDK 凭证）

### 签到记录 (Record)

每次签到操作会生成一条记录，包含签到状态、获得的积分/额度、签到时间等。

---

## 🚀 快速开始

### 1. 添加提供商

从 30+ 内置提供商中选择：

1. 进入「签到管理」页面
2. 点击「添加提供商」按钮
3. 在「内置提供商」标签页中选择提供商
4. 提供商按分类展示（标准 / WAF / CF / 特殊）
5. 点击「添加」完成配置

也可以手动添加自定义提供商，填写域名、签到路径等信息。

### 2. 添加账号

#### 方式一：手动配置

1. 选择一个提供商
2. 点击「添加账号」
3. 填写账号名称
4. 在浏览器中登录对应站点，打开 DevTools → Application → Cookies
5. 复制所有 Cookie，粘贴到 Cookie 输入框（JSON 格式）
6. 可选填 `api_user` 字段
7. 保存

#### 方式二：OAuth 引导登录

支持 OAuth 的提供商会显示「OAuth 登录」按钮：

1. 点击「OAuth 登录」
2. 选择 OAuth 方式（GitHub / LinuxDo）
3. 系统生成授权链接，点击在浏览器中打开
4. 在浏览器完成登录和授权
5. 授权完成后，按引导从浏览器复制 Cookie
6. 粘贴到向导中，自动解析并创建账号

### 3. 执行签到

- **批量签到**：点击页面顶部的「一键签到」按钮
- **单账号签到**：在账号列表中点击对应账号的「签到」按钮

### 4. 查看结果

- 签到结果会实时显示在页面上
- 可在「签到记录」标签页查看历史记录
- 余额变化会自动更新到统计卡片

---

## 📡 提供商管理

### 内置提供商列表

#### 📦 标准站点（24 个）

无需特殊绕过，使用标准 Cookie + API User 认证，签到路径统一为 `/api/user/checkin`。

| 名称 | 域名 | 签到路径 |
|------|------|----------|
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

#### 🛡️ 需 WAF 绕过（1 个）

| 名称 | 域名 | 说明 |
|------|------|------|
| AnyRouter | anyrouter.top | 需要阿里云 WAF Cookie 绕过 |

#### ☁️ 需 CF Clearance 绕过（4 个）

| 名称 | 域名 | 附加功能 |
|------|------|----------|
| RunAnytime | runanytime.hxi.me | 支持 CDK 充值（fuli.hxi.me） |
| Elysiver | elysiver.h-e.top | — |
| Hotaru | hotaruapi.com | — |
| B4U | b4u.qzz.io | 支持 CDK 充值（tw.b4u.qzz.io） |

#### ⚡ 特殊站点（2 个）

| 名称 | 域名 | 机制 |
|------|------|------|
| AgentRouter | agentrouter.cc | 查询用户信息时自动触发签到 |
| CodeRouter | coderouter.cc | 仅余额查询，无签到接口 |

### 自定义提供商

如果目标站点不在内置列表中，可以手动添加：

**必填字段：**

| 字段 | 说明 | 示例 |
|------|------|------|
| `name` | 提供商名称 | `MyProvider` |
| `base_url` | 站点基础 URL | `https://example.com` |
| `checkin_path` | 签到 API 路径 | `/api/user/checkin` |
| `balance_path` | 余额查询路径 | `/api/user/self` |
| `user_info_path` | 用户信息路径 | `/api/user/self` |
| `auth_header` | 认证请求头名 | `Authorization` |
| `auth_prefix` | 认证前缀 | `Bearer` |

---

## 👤 账号管理

### 账号信息

每个账号包含以下信息：

| 字段 | 说明 |
|------|------|
| `name` | 账号显示名称 |
| `provider_id` | 所属提供商 |
| `cookies_json` | Cookie（JSON 格式，加密存储） |
| `api_user` | API User 标识（可选） |
| `enabled` | 是否启用 |
| `extra_config` | 扩展配置（CDK 凭证等） |
| `last_checkin_at` | 上次签到时间 |
| `latest_balance` | 最新余额 |
| `total_quota` | 总配额 |
| `total_consumed` | 总消耗 |

### Cookie 格式

Cookie 支持以下格式输入：

```json
{
  "session": "abc123",
  "new-api-user": "user_token_here"
}
```

或者直接使用浏览器的 Cookie 字符串：

```
session=abc123; new-api-user=user_token_here
```

### 连接测试

添加账号后，建议先执行连接测试：

1. 点击账号旁的「测试」按钮
2. 系统会尝试调用 `/api/user/self` 获取用户信息
3. 成功则显示用户名和余额信息
4. 失败会显示具体错误原因

### OAuth 引导登录

支持 OAuth 的提供商可使用引导式登录：

**支持的 OAuth 方式：**
- **GitHub OAuth** — 部分提供商支持（如 AnyRouter, AgentRouter, DuckCoding）
- **LinuxDo OAuth** — 大多数提供商支持

**使用流程：**

1. 在添加账号对话框中点击「OAuth 引导登录」
2. 选择 OAuth 方式（GitHub / LinuxDo）
3. 系统调用 `POST /checkin/oauth/authorize-url` 获取授权链接
4. 点击链接，在系统浏览器中打开
5. 在浏览器中完成登录和授权
6. 授权完成后，按引导从浏览器 DevTools 中复制 Cookie
7. 将 Cookie 粘贴到向导输入框
8. 系统自动解析并创建账号

::: tip 提示
OAuth 引导式登录不会在后端存储您的 OAuth 密码，所有登录操作在您自己的浏览器中完成。
:::

---

## ✅ 签到操作

### 批量签到

```
POST /checkin/execute
```

请求体（可选）：

```json
{
  "account_ids": ["account_id_1", "account_id_2"]
}
```

- 不传 `account_ids` 时签到所有启用的账号
- 传入特定 ID 列表时只签到指定账号

### 单账号签到

```
POST /checkin/accounts/{id}/checkin
```

### 签到响应

签到成功后返回签到记录，包含：

```json
{
  "id": "record_id",
  "account_id": "account_id",
  "provider_id": "provider_id",
  "status": "success",
  "message": "签到成功！获得 1000 积分",
  "reward_amount": 1000.0,
  "balance_after": 5000.0,
  "checked_in_at": "2026-02-13T10:00:00Z"
}
```

### 签到状态

| 状态 | 说明 |
|------|------|
| `success` | 签到成功 |
| `already_checked_in` | 今日已签到 |
| `failed` | 签到失败 |
| `waf_challenge` | 遇到 WAF 挑战（自动重试） |
| `cf_challenge` | 遇到 CF 挑战（自动重试） |

---

## 💰 余额查询

### 查询余额

```
POST /checkin/accounts/{id}/balance
```

返回余额快照：

```json
{
  "balance": 5000.0,
  "currency": "积分",
  "total_quota": 10000.0,
  "total_consumed": 5000.0,
  "checked_at": "2026-02-13T10:00:00Z"
}
```

### 余额历史

```
GET /checkin/accounts/{id}/balance/history?limit=30
```

返回历史余额快照列表，支持按时间范围查询，用于绘制趋势图。

---

## 🛡️ WAF 绕过（阿里云 WAF）

### 概述

AnyRouter（anyrouter.top）使用阿里云 WAF 防护，直接请求 API 会被拦截。CCR UI 通过 headless Chromium 浏览器自动获取 WAF Cookie，实现绕过。

### 工作原理

```
1. 启动 headless Chromium 浏览器
2. 注入反检测 JS（Stealth JS）
3. 访问目标站点登录页
4. 等待 WAF 验证完成（~12 秒）
5. 提取 WAF Cookie（acw_tc, cdn_sec_tc, acw_sc__v2）
6. 将 WAF Cookie 合并到后续 API 请求中
```

### 需要的 Cookie

| Cookie 名 | 说明 |
|-----------|------|
| `acw_tc` | 阿里云 WAF 跟踪 Cookie |
| `cdn_sec_tc` | CDN 安全验证 Cookie |
| `acw_sc__v2` | 阿里云 WAF 安全挑战 Cookie |

### Stealth JS 注入

为了避免 headless 浏览器被检测，CCR UI 在页面加载前注入反检测脚本：

```javascript
// 移除 navigator.webdriver 标记
Object.defineProperty(navigator, 'webdriver', { get: () => undefined });

// 伪装 Chrome runtime
window.chrome = { runtime: {}, loadTimes: function(){}, csi: function(){} };

// 伪装 permissions API
const originalQuery = window.navigator.permissions.query;
window.navigator.permissions.query = (parameters) => (
    parameters.name === 'notifications' ?
        Promise.resolve({ state: Notification.permission }) :
        originalQuery(parameters)
);

// 伪装 plugins（headless 通常为空）
Object.defineProperty(navigator, 'plugins', {
    get: () => [1, 2, 3, 4, 5]
});

// 伪装 languages
Object.defineProperty(navigator, 'languages', {
    get: () => ['zh-CN', 'zh', 'en-US', 'en']
});
```

### 反检测浏览器参数

```
--disable-blink-features=AutomationControlled
--disable-features=IsolateOrigins,site-per-process
--disable-infobars
--disable-dev-shm-usage
```

### 前置条件

- 系统需安装 Chromium 内核浏览器（Chrome/Chromium/Brave/Edge）
- 支持 Windows、macOS、Linux 系统
- 自动检测浏览器安装路径

---

## ☁️ CF Clearance 绕过（Cloudflare）

### 概述

4 个提供商使用 Cloudflare 防护：**RunAnytime**、**Elysiver**、**Hotaru**、**B4U**。访问时会出现 "Just a moment..." 挑战页面，CCR UI 通过 headless Chromium 自动解决。

### 工作原理

```
1. 启动 headless Chromium 浏览器
2. 注入反检测 JS（同 WAF 绕过）
3. 访问目标站点
4. 检测页面标题（轮询判断是否仍在 "Just a moment..." 挑战页）
5. 等待 Cloudflare 验证通过（标题变化表示挑战通过）
6. 提取 cf_clearance Cookie
7. 将 cf_clearance 合并到后续 API 请求中
```

### 需要的 Cookie

| Cookie 名 | 说明 |
|-----------|------|
| `cf_clearance` | Cloudflare 挑战通过凭证 |

### 需要 CF 绕过的站点

| 站点 | 域名 | 附加说明 |
|------|------|----------|
| RunAnytime | runanytime.hxi.me | 还支持 CDK 充值 |
| Elysiver | elysiver.h-e.top | — |
| Hotaru | hotaruapi.com | — |
| B4U | b4u.qzz.io | 还支持 CDK 充值 |

### 自动重试机制

当签到或余额查询时检测到 CF 挑战响应（HTTP 403 + CF 标记），系统会：

1. 自动触发 CF Clearance 获取
2. 重新发送原请求（附带 `cf_clearance` Cookie）
3. 如果再次失败，记录为 `cf_challenge` 状态

::: warning 注意
CF Clearance 有效期有限（通常几小时），系统会在过期后自动重新获取。首次获取可能需要 10-30 秒。
:::

---

## 🎁 CDK 充值

### 概述

3 个站点支持通过外部福利站获取充值码（CDK）并自动充值到账户。

### 支持站点

| 站点 | CDK 来源 | 获取方式 | 充值方式 |
|------|----------|----------|----------|
| RunAnytime | fuli.hxi.me | 签到 + 大转盘 | `POST /api/user/topup` |
| B4U | tw.b4u.qzz.io | 幸运抽奖 | `POST /api/user/topup` |
| x666 | up.x666.me | 每日抽奖 | 直接到账（无需充值） |

### 使用方式

1. 为支持 CDK 的账号配置 CDK 凭证（`extra_config` 中）
2. 签到完成后，系统会自动检测 CDK 配置
3. 自动获取充值码并执行充值
4. 也可以手动点击「CDK 充值」按钮

### CDK 充值 API

```
POST /checkin/accounts/{id}/topup
```

返回 `CdkTopupResult`：

```json
{
  "cdk_type": "runawaytime",
  "success": true,
  "message": "充值成功",
  "codes_found": 3,
  "codes_redeemed": 2,
  "failed_codes": ["INVALID_CODE"],
  "direct_reward": null
}
```

### RunAnytime CDK 详解

1. **签到**: POST `fuli.hxi.me/api/checkin` → 获取 CDK 码
2. **大转盘**: POST `fuli.hxi.me/api/wheel` → 循环抽奖直到次数用完
3. **充值**: 将所有 CDK 码逐个发送到 `/api/user/topup`

**需要的凭证**: `fuli.hxi.me` 的登录 Cookie（存储在 `extra_config.cdk_cookies` 中）

### B4U CDK 详解

1. 需要先获取 `cf_clearance`（复用 CF 绕过服务）
2. 访问 `tw.b4u.qzz.io/luckydraw` 执行抽奖
3. 解析 Next.js Server Actions 响应格式
4. 将获得的 CDK 码充值到账户

**需要的凭证**: `tw.b4u.qzz.io` 的登录 Cookie

### x666 CDK 详解

1. 使用 JWT `access_token` 认证
2. POST `up.x666.me/api/checkin/spin` 执行抽奖
3. 奖励直接充值到账户（无需额外 topup 步骤）

**需要的凭证**: `access_token`（JWT，存储在 `extra_config.access_token` 中）

---

## 📊 账号 Dashboard

### 概述

每个账号都有独立的 Dashboard 面板，展示详细的签到统计、日历视图和余额趋势。

### 访问方式

```
GET /checkin/accounts/{id}/dashboard?year=2026&month=2&days=30
```

### Dashboard 数据

#### 连续签到统计 (Streak)

```json
{
  "current_streak": 15,
  "longest_streak": 30,
  "total_check_in_days": 120
}
```

#### 月历日历 (Calendar)

展示当月每天的签到状态：

```json
{
  "days": [
    {
      "date": "2026-02-01",
      "is_checked_in": true,
      "income_increment": 1000.0,
      "current_balance": 5000.0
    }
  ]
}
```

#### 余额趋势 (Trend)

展示指定天数内的余额变化趋势：

```json
{
  "data_points": [
    {
      "date": "2026-02-01",
      "total_quota": 10000.0,
      "income_increment": 1000.0,
      "current_balance": 5000.0,
      "is_checked_in": true
    }
  ]
}
```

---

## 📦 数据管理

### 导出配置

```
POST /checkin/export
```

请求参数：

```json
{
  "include_plaintext_keys": false,
  "providers_only": false
}
```

- `include_plaintext_keys`: 是否包含明文 Cookie（默认加密）
- `providers_only`: 是否只导出提供商（不含账号）

### 预览导入

```
POST /checkin/import/preview
```

上传导出文件，预览将要导入的内容和冲突项。

### 执行导入

```
POST /checkin/import
```

请求参数：

```json
{
  "data": { "...导出数据..." },
  "options": {
    "conflict_strategy": "skip"
  }
}
```

**冲突策略：**

| 策略 | 说明 |
|------|------|
| `skip` | 跳过冲突项，保留现有配置 |
| `overwrite` | 覆盖冲突项，使用导入的配置 |

### 导出签到记录

```
GET /checkin/records/export
```

以文件下载方式导出签到记录。

---

## ❓ 常见问题

### Q1: 支持哪些提供商？

CCR UI 内置 30+ 提供商，覆盖主流 NewAPI 公益 AI 中转站。分为四类：
- **标准站点** (24 个)：直接 Cookie 认证
- **WAF 站点** (1 个)：AnyRouter，需自动绕过阿里云 WAF
- **CF 站点** (4 个)：需自动绕过 Cloudflare 防护
- **特殊站点** (2 个)：AgentRouter（自动签到）、CodeRouter（仅余额）

### Q2: WAF/CF 绕过需要什么前置条件？

需要系统安装 Chromium 内核浏览器（Chrome、Chromium、Brave 或 Edge）。系统会自动检测安装路径，支持 Windows/macOS/Linux。

### Q3: Cookie 如何获取？

三种方式：
1. **手动获取**：在浏览器中登录站点 → F12 打开 DevTools → Application → Cookies → 复制
2. **OAuth 引导**：使用 GitHub/LinuxDo OAuth 向导（支持的提供商会显示按钮）
3. **字符串格式**：直接粘贴浏览器的 Cookie 字符串（`key1=value1; key2=value2`）

### Q4: CDK 充值如何配置？

在编辑账号时，为支持 CDK 的提供商配置 `extra_config`：
- **RunAnytime**: 填入 `fuli.hxi.me` 的登录 Cookie
- **B4U**: 填入 `tw.b4u.qzz.io` 的登录 Cookie
- **x666**: 填入 `access_token`（JWT）

### Q5: 签到失败怎么办？

1. 先执行「连接测试」确认账号可用
2. 检查 Cookie 是否过期（重新登录获取新 Cookie）
3. WAF/CF 站点：确认系统已安装 Chrome/Chromium 浏览器
4. 查看签到记录中的错误信息排查具体原因

### Q6: 数据存储在哪里？

- 数据库路径：`~/.ccr-ui/checkin.db`（SQLite）
- Cookie 加密方式：AES-256-GCM
- 所有数据完全在本地存储，不会上传到任何服务器

### Q7: 如何迁移到新设备？

使用导入/导出功能：
1. 在旧设备上执行「导出配置」（勾选"包含明文密钥"）
2. 将导出文件传输到新设备
3. 在新设备上执行「导入配置」
4. 选择冲突策略（skip 或 overwrite）

### Q8: OAuth 登录安全吗？

完全安全。OAuth 引导式登录在您自己的浏览器中完成：
- CCR UI 仅获取授权链接，不接触您的密码
- 登录和授权在系统浏览器中进行
- 您手动复制 Cookie 粘贴到 CCR UI
- Cookie 使用 AES-256-GCM 加密后存入本地数据库

---

**浮浮酱温馨提示**：签到功能完全在本地运行，您的 API Key 使用 AES-256-GCM 加密存储，安全可靠！(´｡• ᵕ •｡`) ♡
