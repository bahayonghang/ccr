# Checkin Management

Transit station check-in feature supporting multi-account auto check-in, balance queries, and history tracking.

## Feature Overview

### Statistics Cards
Three statistics cards at the top:
- **Current Balance** 🟢 - Total available balance across all accounts
- **Total Quota** 🔵 - Cumulative quota earned by all accounts
- **Historical Consumption** 🟠 - Cumulative usage across all accounts

### Account Management
| Feature | Description |
|---------|-------------|
| Search Filter | Search by account or provider name |
| Provider Filter | Dropdown to select specific provider |
| Batch Check-in | Execute check-in for all enabled accounts |
| Refresh Balance | Batch refresh all account balances |

### Account Table Columns
- Account Name (with provider tag)
- Balance (current available)
- Total Quota (cumulative earned)
- Historical Consumption (cumulative used)
- Last Check-in Time
- Actions (check-in, refresh, edit, delete)

## Quick Start

### 1. Add Provider
In the "Providers" tab:
- Click built-in provider cards for quick add
- Or manually create custom providers

### 2. Add Account
In the "Account Management" tab:
1. Click "Add Account"
2. Select provider
3. Enter account name
4. Paste Cookies JSON (format: `{"session": "xxx", "token": "yyy"}`)
5. Optional: Fill in API User (New-Api-User header value)

### 3. Execute Check-in
- **Batch check-in**: Click "Batch Check-in" at top right
- **Single check-in**: Click "Check-in" button on account row

### 4. View Balance
- First time requires clicking "Refresh Balance" to fetch data
- Balance data comes from provider API `/api/user/self`

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/checkin/accounts` | Get account list |
| POST | `/api/checkin/accounts` | Create account |
| POST | `/api/checkin/execute` | Execute check-in |
| POST | `/api/checkin/accounts/:id/balance` | Refresh balance |
| GET | `/api/checkin/records` | Get check-in records |

## Data Storage

Check-in data is stored in `~/.ccr/checkin/`:
- `providers.json` - Provider configurations
- `accounts.json` - Account info (Cookies encrypted)
- `balance/` - Balance snapshots
- `records/` - Check-in records
