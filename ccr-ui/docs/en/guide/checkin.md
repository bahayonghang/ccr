# Checkin Management

> **Version**: v4.0+
> **Last Updated**: 2026-02-13
> **Status**: ✅ Stable

Automated check-in management for AI transit station providers. Supports 30+ builtin providers, multi-account management, WAF/CF bypass, CDK topup, OAuth guided login, and encrypted credential storage.

## Feature Overview

### Statistics Cards

Three statistics cards at the top of the checkin page:

- **Current Balance** 🟢 — Total available balance across all accounts
- **Total Quota** 🔵 — Cumulative quota earned by all accounts
- **Historical Consumption** 🟠 — Cumulative usage across all accounts

### Core Capabilities

| Feature | Description |
|---------|-------------|
| 📋 Multi-Provider | 30+ builtin providers categorized by type |
| 👥 Multi-Account | Multiple accounts per provider, independently managed |
| 🔄 Auto Check-in | One-click batch check-in for all accounts |
| 💰 Balance Query | Real-time balance, quota, and consumption tracking |
| 📊 Dashboard | Streak stats, monthly calendar, balance trend chart |
| 🛡️ WAF Bypass | Automatic Alibaba Cloud WAF bypass (AnyRouter) |
| ☁️ CF Bypass | Automatic Cloudflare challenge bypass (4 sites) |
| 🎁 CDK Topup | Auto-fetch and redeem CDK codes (3 sites) |
| 🔑 OAuth Login | GitHub/LinuxDo OAuth wizard for easy setup |
| 📦 Import/Export | Full config backup and restore with conflict resolution |
| 🔒 Encryption | AES-256-GCM encrypted cookies in local SQLite |
| 🧪 Connection Test | Verify account connectivity before check-in |

---

## Builtin Providers

### 📦 Standard Sites (24)

Standard NewAPI sites with no special bypass required. All use Cookie + API User authentication with `/api/user/checkin` path.

| Name | Domain | Checkin Path |
|------|--------|-------------|
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

### 🛡️ WAF Required (1)

| Name | Domain | Notes |
|------|--------|-------|
| AnyRouter | anyrouter.top | Requires Alibaba Cloud WAF cookie bypass |

### ☁️ CF Clearance Required (4)

| Name | Domain | Extra Features |
|------|--------|----------------|
| RunAnytime | runanytime.hxi.me | CDK topup via fuli.hxi.me |
| Elysiver | elysiver.h-e.top | — |
| Hotaru | hotaruapi.com | — |
| B4U | b4u.qzz.io | CDK topup via tw.b4u.qzz.io |

### ⚡ Special Sites (2)

| Name | Domain | Mechanism |
|------|--------|-----------|
| AgentRouter | agentrouter.cc | Auto check-in when querying user info |
| CodeRouter | coderouter.cc | Balance-only, no check-in endpoint |

---

## Quick Start

### 1. Add a Provider

1. Navigate to the **Checkin Management** page
2. Click **Add Provider**
3. Select from the **Builtin Providers** tab (grouped by category)
4. Click **Add** to confirm

You can also manually add custom providers by filling in the domain, check-in path, and other configuration.

### 2. Add an Account

#### Option A: Manual Configuration

1. Select a provider
2. Click **Add Account**
3. Enter account name
4. Log in to the provider site in your browser, open DevTools → Application → Cookies
5. Copy all cookies and paste into the Cookie input (JSON format)
6. Optionally fill in the `api_user` field
7. Save

#### Option B: OAuth Guided Login

Providers with OAuth support show an **OAuth Login** button:

1. Click **OAuth Login**
2. Choose OAuth method (GitHub / LinuxDo)
3. System generates an authorization URL — click to open in your browser
4. Complete login and authorization in the browser
5. Follow the guide to copy cookies from the browser
6. Paste into the wizard — account is created automatically

### 3. Execute Check-in

- **Batch check-in**: Click the **Check-in All** button at the top
- **Single account**: Click the **Check-in** button next to an account

### 4. View Results

- Check-in results display in real-time
- View history in the **Records** tab
- Balance changes auto-update the statistics cards

---

## WAF Bypass

### Overview

AnyRouter (anyrouter.top) uses Alibaba Cloud WAF protection. CCR UI automatically obtains WAF cookies via headless Chromium browser.

### How It Works

1. Launch headless Chromium browser
2. Inject stealth JavaScript (anti-detection)
3. Navigate to the target site's login page
4. Wait for WAF verification (~12 seconds)
5. Extract WAF cookies (`acw_tc`, `cdn_sec_tc`, `acw_sc__v2`)
6. Merge WAF cookies into subsequent API requests

### Anti-Detection Measures

- **navigator.webdriver removal** — prevents headless detection
- **Chrome runtime fake** — simulates real Chrome runtime object
- **Permissions API fake** — returns proper notification permission
- **Plugins/Languages fake** — non-empty plugin list, realistic language array
- **Browser flags** — `--disable-blink-features=AutomationControlled`, `--disable-infobars`, etc.

### Prerequisites

A Chromium-based browser must be installed (Chrome, Chromium, Brave, or Edge). The system auto-detects installation paths on Windows, macOS, and Linux.

---

## CF Clearance Bypass

### Overview

Four providers use Cloudflare protection: **RunAnytime**, **Elysiver**, **Hotaru**, and **B4U**. When accessed, these sites display a "Just a moment..." challenge page. CCR UI solves this automatically via headless Chromium.

### How It Works

1. Launch headless Chromium with stealth JS injection
2. Navigate to the target site
3. Poll page title to detect challenge completion (title changes from "Just a moment...")
4. Extract `cf_clearance` cookie after challenge is solved
5. Merge cookie into subsequent API requests

### Auto-Retry

When a check-in or balance query detects a CF challenge response (HTTP 403 + CF markers), the system automatically:

1. Triggers CF Clearance acquisition
2. Retries the original request with `cf_clearance` cookie
3. Records as `cf_challenge` status if retry also fails

::: warning Note
CF Clearance cookies expire after a few hours. The system automatically re-acquires them when needed. Initial acquisition may take 10-30 seconds.
:::

---

## CDK Topup

### Overview

Three sites support automatic CDK (code) acquisition and redemption:

| Site | CDK Source | Method | Redemption |
|------|-----------|--------|------------|
| RunAnytime | fuli.hxi.me | Check-in + Wheel spin | `POST /api/user/topup` |
| B4U | tw.b4u.qzz.io | Lucky draw | `POST /api/user/topup` |
| x666 | up.x666.me | Daily spin | Direct reward (no topup needed) |

### Usage

1. Configure CDK credentials in the account's `extra_config`
2. After check-in, the system auto-detects CDK configuration
3. Automatically fetches codes and redeems them
4. Manual topup is also available via the **CDK Topup** button

### CDK Topup API

```
POST /checkin/accounts/{id}/topup
```

Response:

```json
{
  "cdk_type": "runawaytime",
  "success": true,
  "message": "Topup successful",
  "codes_found": 3,
  "codes_redeemed": 2,
  "failed_codes": ["INVALID_CODE"],
  "direct_reward": null
}
```

### Credential Requirements

| Site | Required Credential | Storage |
|------|-------------------|---------|
| RunAnytime | fuli.hxi.me login cookies | `extra_config.cdk_cookies` |
| B4U | tw.b4u.qzz.io login cookies | `extra_config.cdk_cookies` |
| x666 | JWT access_token | `extra_config.access_token` |

---

## OAuth Guided Login

### Supported Methods

- **GitHub OAuth** — available for some providers (AnyRouter, AgentRouter, DuckCoding, etc.)
- **LinuxDo OAuth** — available for most providers

### Workflow

1. Click **OAuth Login** in the Add Account dialog
2. Select OAuth method (GitHub / LinuxDo)
3. System calls `POST /checkin/oauth/authorize-url` to get the authorization URL
4. Click the link to open in your system browser
5. Complete login and authorization in the browser
6. After authorization, follow the guide to copy cookies from browser DevTools
7. Paste into the wizard input — system auto-parses and creates the account

::: tip Security
OAuth guided login does NOT store your OAuth password on the backend. All login operations happen in your own browser. You manually copy-paste cookies to CCR UI, where they are encrypted with AES-256-GCM.
:::

---

## Account Dashboard

Each account has a dedicated dashboard with detailed statistics:

### Streak Statistics

```json
{
  "current_streak": 15,
  "longest_streak": 30,
  "total_check_in_days": 120
}
```

### Monthly Calendar

Visual calendar showing daily check-in status with income and balance data for each day.

### Balance Trend

Line chart showing balance changes over a configurable time period (default: 30 days).

### API

```
GET /checkin/accounts/{id}/dashboard?year=2026&month=2&days=30
```

---

## Data Management

### Export

```
POST /checkin/export
```

Options:
- `include_plaintext_keys` — Include decrypted cookies (default: encrypted)
- `providers_only` — Export only providers (no accounts)

### Import Preview

```
POST /checkin/import/preview
```

Upload export data to preview contents and conflicts before importing.

### Import

```
POST /checkin/import
```

Conflict strategies:

| Strategy | Description |
|----------|-------------|
| `skip` | Skip conflicting items, keep existing config |
| `overwrite` | Overwrite conflicting items with imported config |

### Export Records

```
GET /checkin/records/export
```

Download check-in records as a file.

---

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/checkin/providers` | List all configured providers |
| `POST` | `/checkin/providers` | Create custom provider |
| `GET` | `/checkin/providers/builtin` | List all 30+ builtin providers |
| `POST` | `/checkin/providers/builtin/add` | Add builtin provider to config |
| `GET` | `/checkin/providers/{id}` | Get single provider |
| `PUT` | `/checkin/providers/{id}` | Update provider |
| `DELETE` | `/checkin/providers/{id}` | Delete provider (cascades to accounts) |
| `GET` | `/checkin/accounts` | List all accounts |
| `POST` | `/checkin/accounts` | Create account |
| `GET` | `/checkin/accounts/{id}` | Get single account |
| `GET` | `/checkin/accounts/{id}/dashboard` | Get account dashboard |
| `PUT` | `/checkin/accounts/{id}` | Update account |
| `DELETE` | `/checkin/accounts/{id}` | Delete account |
| `GET` | `/checkin/accounts/{id}/cookies` | Get decrypted cookies |
| `POST` | `/checkin/execute` | Batch check-in |
| `POST` | `/checkin/accounts/{id}/checkin` | Single account check-in |
| `POST` | `/checkin/accounts/{id}/topup` | Execute CDK topup |
| `POST` | `/checkin/accounts/{id}/balance` | Query balance |
| `GET` | `/checkin/accounts/{id}/balance/history` | Get balance history |
| `GET` | `/checkin/records` | List all records |
| `GET` | `/checkin/records/export` | Export records |
| `GET` | `/checkin/accounts/{id}/records` | Get account records |
| `GET` | `/checkin/stats/today` | Today's checkin stats |
| `POST` | `/checkin/export` | Export config |
| `POST` | `/checkin/import/preview` | Preview import |
| `POST` | `/checkin/import` | Execute import |
| `POST` | `/checkin/accounts/{id}/test` | Test account connectivity |
| `POST` | `/checkin/oauth/authorize-url` | Get OAuth authorization URL |

---

## Data Security

- **Encryption**: All cookies are encrypted with **AES-256-GCM** before storage
- **Local Storage**: Data stored in SQLite database at `~/.ccr-ui/checkin.db`
- **No Cloud Sync**: All data remains on your local machine — nothing is uploaded to any server
- **OAuth Safety**: OAuth login happens in your own browser; CCR UI never handles your passwords
- **Secure Export**: Export files contain encrypted credentials by default; plaintext is opt-in

---

## FAQ

### Q1: What providers are supported?

CCR UI includes 30+ builtin providers covering major NewAPI transit stations:
- **Standard** (24): Direct cookie authentication
- **WAF Required** (1): AnyRouter with automatic Alibaba Cloud WAF bypass
- **CF Required** (4): Automatic Cloudflare challenge bypass
- **Special** (2): AgentRouter (auto check-in), CodeRouter (balance only)

### Q2: What are the prerequisites for WAF/CF bypass?

A Chromium-based browser must be installed (Chrome, Chromium, Brave, or Edge). The system auto-detects the browser path on Windows, macOS, and Linux.

### Q3: How do I get cookies?

Three methods:
1. **Manual**: Log in to the site → F12 → Application → Cookies → Copy
2. **OAuth Wizard**: Use the GitHub/LinuxDo OAuth guided login
3. **String format**: Paste browser cookie string (`key1=value1; key2=value2`)

### Q4: How do I configure CDK topup?

Edit the account and set `extra_config`:
- **RunAnytime**: Enter `fuli.hxi.me` login cookies
- **B4U**: Enter `tw.b4u.qzz.io` login cookies
- **x666**: Enter `access_token` (JWT)

### Q5: What if check-in fails?

1. Run a **Connection Test** to verify the account is reachable
2. Check if cookies have expired (re-login to get fresh cookies)
3. For WAF/CF sites: ensure Chrome/Chromium is installed
4. Check the error message in the check-in records

### Q6: Where is data stored?

- Database: `~/.ccr-ui/checkin.db` (SQLite)
- Encryption: AES-256-GCM
- All data is stored locally — never uploaded to any server

### Q7: How do I migrate to a new device?

Use Import/Export:
1. Export config on the old device (enable "include plaintext keys")
2. Transfer the export file to the new device
3. Import config on the new device
4. Choose conflict strategy (skip or overwrite)

### Q8: Is OAuth login secure?

Yes. OAuth guided login runs entirely in your own browser:
- CCR UI only provides the authorization URL
- You log in and authorize in your own browser
- You manually copy cookies to CCR UI
- Cookies are encrypted with AES-256-GCM in the local database
