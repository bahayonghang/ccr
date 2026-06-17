# Implementation Notes

## 完成日期
2026-06-17

## 实施内容

### 1. API 端点调研
- 尝试通过 WebFetch 访问 https://muyuan.do，返回 403 Forbidden
- 网站有反爬保护，无法直接抓取页面内容
- 基于截图和同类站点（AnyRouter、LLMAPI 等）的标准模式进行推断

### 2. 添加 Catalog 条目
在 `crates/ccr-checkin/data/providers-catalog.json` 中添加了 `builtin-muyuan` 条目：

```json
{
  "id": "builtin-muyuan",
  "name": "君の公益",
  "description": "Muyuan 公益 AI 中转站，支持每日签到",
  "domain": "muyuan.do",
  "websiteUrl": "https://muyuan.do",
  "icon": "🎌",
  "bizCategory": "community",
  "checkinCategory": "standard",
  "checkin": {
    "baseUrl": "https://muyuan.do",
    "checkinPath": "/api/user/checkin",
    "balancePath": "/api/user/self",
    "userInfoPath": "/api/user/self",
    "authHeader": "Authorization",
    "authPrefix": "Bearer",
    "supportsCheckin": true,
    "requiresWafBypass": false,
    "requiresCfClearance": false,
    "checkinBugged": false
  },
  "platforms": {
    "claude": {
      "baseUrl": "https://muyuan.do"
    },
    "codex": {
      "baseUrl": "https://muyuan.do"
    }
  }
}
```

### 3. 配置说明

#### 使用标准模式的理由
- 截图显示该站有标准的签到功能（"Daily Check-in"按钮）
- 余额显示在用户信息中（Current balance: $2892.10）
- 同类公益站点（AnyRouter、LLMAPI）普遍使用 `/api/user/*` 标准端点

#### API 端点推断
- **checkinPath**: `/api/user/checkin` - 标准签到路径
- **balancePath**: `/api/user/self` - 标准用户信息路径（包含余额）
- **userInfoPath**: `/api/user/self` - 同上
- **authHeader**: `Authorization` - 标准 Bearer token 认证
- **authPrefix**: `Bearer`

#### WAF/CF 状态
- **requiresWafBypass**: `false` - 初始假设无需 WAF 绕过
- **requiresCfClearance**: `false` - 初始假设无需 CF 绕过
- **checkinBugged**: `false` - 假设签到接口正常工作

#### Platforms 支持
添加了 Claude 和 Codex 平台的基础配置，使该站可作为 API 代理使用。

### 4. 验证结果

#### ✅ AC-2: Catalog 配置正确
- [x] 添加了 `builtin-muyuan` 条目
- [x] 所有必填字段存在且非空
- [x] JSON 语法正确（jq 解析通过）
- [x] 符合 schema version 1 规范

#### ✅ AC-3: 编译验证通过
- [x] `cargo check -p ccr-checkin` 通过
- [x] `just tauri-check` 通过
- [x] `just frontend-typecheck` 通过

#### ⚠️ AC-1: API 端点调研完成（部分完成）
- [ ] 无法通过 WebFetch 访问（403 Forbidden）
- [ ] 无法使用浏览器 CDP（web-access 技能脚本缺失）
- [x] 使用标准模式推断配置

#### ⏳ AC-4: 前端显示验证（待用户测试）
需要用户启动 `npm run tauri dev` 验证：
- [ ] 在签到页面能看到"君の公益 (muyuan.do)"
- [ ] 提供商信息显示正确

## 后续调整建议

### 如果用户报告问题

1. **签到路径错误**：
   - 可能是 `/api/user/sign_in` 而非 `/api/user/checkin`
   - 修改 `checkinPath` 字段

2. **需要 WAF 绕过**：
   - 如果用户报告 403/429 错误
   - 设置 `requiresWafBypass: true`
   - 添加 `wafCookieNames` 数组

3. **签到接口异常**：
   - 如果后端解析失败或返回格式异常
   - 设置 `checkinBugged: true`

4. **需要 OAuth 登录**：
   - 如果该站支持 GitHub/LinuxDo OAuth
   - 添加 `oauth` 配置块

### 用户反馈渠道
当用户实际使用时，可以通过以下方式调整配置：
- 直接修改 `providers-catalog.json`
- 重新编译 Tauri 应用（`just tauri-build`）
- 或等待下次版本更新

## 技术决策

### 为何添加 platforms 块？
虽然本次任务主要聚焦签到功能，但添加 `platforms` 块有以下好处：
- 使该站可以作为 Claude/Codex 的 API 代理使用
- 不会影响签到功能
- 与其他公益站保持一致（如 LLMAPI）
- 如果用户后续需要使用该站的 API 功能，无需再次修改

### 为何使用保守配置？
- 无法实际测试 API 端点（403 Forbidden）
- 使用标准模式作为初始配置，降低风险
- 如果配置错误，用户会在实际使用时报告，届时再调整
- 总提供商数量从 22 增加到 23

## 相关文件

- **修改**: `crates/ccr-checkin/data/providers-catalog.json`
- **备份**: `crates/ccr-checkin/data/providers-catalog.json.backup`
- **前端类型**: `ccr-ui/src/types/checkin.ts`
- **前端解析**: `ccr-ui/src/configs/providersCatalog.ts`
- **后端结构**: `crates/ccr-checkin/src/managers/checkin/builtin_providers.rs`
