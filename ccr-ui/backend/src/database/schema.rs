// Database schema definitions for unified SQLite storage
// Schema version: 2
// See: openspec/changes/add-unified-sqlite-storage/proposal.md

/// Current schema version for migration tracking
#[allow(dead_code)]
pub const SCHEMA_VERSION: i32 = 4;

/// Database file path relative to user home directory
pub const DB_RELATIVE_PATH: &str = ".ccr-ui/ccr-ui.db";

/// SQL statements for creating all tables
pub const CREATE_TABLES_SQL: &str = r#"
-- ═══════════════════════════════════════════════════════════
-- UI State Tables
-- ═══════════════════════════════════════════════════════════

-- UI Favorites: stored favorite commands
CREATE TABLE IF NOT EXISTS ui_favorites (
    id TEXT PRIMARY KEY,
    command TEXT NOT NULL,
    args_json TEXT NOT NULL,  -- JSON array
    display_name TEXT,
    module TEXT NOT NULL,
    created_at TEXT NOT NULL  -- ISO8601 UTC
);

CREATE INDEX IF NOT EXISTS idx_ui_favorites_created_at
    ON ui_favorites (created_at DESC);

-- UI History: command execution history (starts fresh after migration)
CREATE TABLE IF NOT EXISTS ui_history (
    id TEXT PRIMARY KEY,
    full_command TEXT NOT NULL,
    command TEXT NOT NULL,
    args_json TEXT NOT NULL,
    success INTEGER NOT NULL,  -- 0/1
    executed_at TEXT NOT NULL,
    duration_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ui_history_executed_at
    ON ui_history (executed_at DESC);

-- ═══════════════════════════════════════════════════════════
-- Checkin Tables
-- ═══════════════════════════════════════════════════════════

-- Checkin Providers: service provider configurations
CREATE TABLE IF NOT EXISTS checkin_providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    checkin_path TEXT NOT NULL,
    balance_path TEXT NOT NULL,
    user_info_path TEXT NOT NULL,
    auth_header TEXT NOT NULL,
    auth_prefix TEXT NOT NULL,
    enabled INTEGER NOT NULL,  -- 0/1
    created_at TEXT NOT NULL,
    updated_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_checkin_providers_name
    ON checkin_providers (name);

-- Checkin Accounts: user accounts with encrypted cookies
CREATE TABLE IF NOT EXISTS checkin_accounts (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    name TEXT NOT NULL,
    cookies_json_encrypted TEXT NOT NULL,
    api_user TEXT NOT NULL,
    enabled INTEGER NOT NULL,  -- 0/1
    created_at TEXT NOT NULL,
    updated_at TEXT,
    last_checkin_at TEXT,
    last_balance_check_at TEXT,
    extra_config TEXT NOT NULL DEFAULT '{}'  -- JSON: CDK credentials, OAuth tokens, etc.
);

CREATE INDEX IF NOT EXISTS idx_checkin_accounts_provider_id
    ON checkin_accounts (provider_id);

CREATE INDEX IF NOT EXISTS idx_checkin_accounts_enabled
    ON checkin_accounts (enabled);

-- Checkin Records: daily checkin results
CREATE TABLE IF NOT EXISTS checkin_records (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    status TEXT NOT NULL,  -- success|failed|already_checked_in
    message TEXT,
    reward TEXT,
    balance_before REAL,
    balance_after REAL,
    checked_in_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_checkin_records_account_id
    ON checkin_records (account_id);

CREATE INDEX IF NOT EXISTS idx_checkin_records_checked_in_at
    ON checkin_records (checked_in_at DESC);

CREATE INDEX IF NOT EXISTS idx_checkin_records_status
    ON checkin_records (status);

-- Checkin Balances: balance snapshots (quota tracking)
CREATE TABLE IF NOT EXISTS checkin_balances (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    total_quota REAL NOT NULL,
    used_quota REAL NOT NULL,
    remaining_quota REAL NOT NULL,
    currency TEXT NOT NULL,
    raw_response TEXT,
    recorded_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_checkin_balances_account_id
    ON checkin_balances (account_id);

CREATE INDEX IF NOT EXISTS idx_checkin_balances_recorded_at
    ON checkin_balances (recorded_at DESC);

-- Checkin WAF Cookies: cached WAF bypass cookies
CREATE TABLE IF NOT EXISTS checkin_waf_cookies (
    provider_id TEXT PRIMARY KEY,
    cookies_json TEXT NOT NULL,
    fetched_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_checkin_waf_cookies_expires_at
    ON checkin_waf_cookies (expires_at);

-- ═══════════════════════════════════════════════════════════
-- Log Tables
-- ═══════════════════════════════════════════════════════════

-- Log Entries: application log storage
CREATE TABLE IF NOT EXISTS log_entries (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    level TEXT NOT NULL,
    source TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_log_entries_timestamp
    ON log_entries (timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_log_entries_level
    ON log_entries (level);

-- ═══════════════════════════════════════════════════════════
-- Usage Tracking Tables
-- ═══════════════════════════════════════════════════════════

-- Usage Sources: tracks imported files with offsets
CREATE TABLE IF NOT EXISTS usage_sources (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,  -- claude|codex|gemini
    file_path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    last_offset INTEGER NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_usage_sources_platform
    ON usage_sources (platform);

CREATE INDEX IF NOT EXISTS idx_usage_sources_file_path
    ON usage_sources (file_path);

-- Usage Records: individual usage entries
CREATE TABLE IF NOT EXISTS usage_records (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    project_path TEXT NOT NULL,
    record_json TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    source_id TEXT NOT NULL,
    model TEXT,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    cache_read_tokens INTEGER DEFAULT 0,
    cost_usd REAL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_usage_records_platform_recorded_at
    ON usage_records (platform, recorded_at DESC);

CREATE INDEX IF NOT EXISTS idx_usage_records_source_id
    ON usage_records (source_id);

CREATE INDEX IF NOT EXISTS idx_usage_records_platform_model_recorded_at_id
    ON usage_records (platform, model, recorded_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_usage_records_platform_recorded_at_id
    ON usage_records (platform, recorded_at DESC, id DESC);

-- ═══════════════════════════════════════════════════════════
-- Usage Daily Aggregation Table (pre-computed)
-- ═══════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS usage_daily_agg (
    date TEXT NOT NULL,
    platform TEXT NOT NULL,
    request_count INTEGER DEFAULT 0,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    cache_read_tokens INTEGER DEFAULT 0,
    cost_usd REAL DEFAULT 0,
    PRIMARY KEY (date, platform)
);

-- ═══════════════════════════════════════════════════════════
-- Migration Tracking Table
-- ═══════════════════════════════════════════════════════════

-- Migrations: tracks applied schema migrations
CREATE TABLE IF NOT EXISTS migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL
);
"#;

/// SQL to check if initial migration was performed
#[allow(dead_code)]
pub const CHECK_MIGRATION_SQL: &str = "SELECT COUNT(*) FROM migrations WHERE version = 1";

/// SQL to insert migration record
pub const INSERT_MIGRATION_SQL: &str =
    "INSERT INTO migrations (version, name, applied_at) VALUES (?, ?, ?)";
