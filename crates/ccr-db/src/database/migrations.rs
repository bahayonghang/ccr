// Database migrations for unified SQLite storage
// Handles schema creation and data migration from legacy JSON files

use ccr_types::ModelRateCatalog;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

use crate::database::repositories::{checkin_repo, ui_state_repo};
use crate::database::schema::{CREATE_TABLES_SQL, INSERT_MIGRATION_SQL};
use crate::models::checkin::balance::BalanceSnapshot;
use crate::models::checkin::{CheckinAccount, CheckinProvider, CheckinRecord};
use crate::models::ui_state::FavoriteCommand;

use crate::core::error::MigrationError;

/// Result type for migration operations
pub type MigrationResult<T> = Result<T, MigrationError>;

/// Check if schema migration has been applied
pub fn is_migration_applied(conn: &Connection, version: i32) -> MigrationResult<bool> {
    let result: Result<i32, _> = conn.query_row(
        "SELECT COUNT(*) FROM migrations WHERE version = ?",
        [version],
        |row| row.get(0),
    );

    match result {
        Ok(count) => Ok(count > 0),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        // Table doesn't exist yet - migration not applied
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.extended_code == rusqlite::ffi::SQLITE_ERROR =>
        {
            Ok(false)
        }
        Err(e) => Err(MigrationError::Database(e.to_string())),
    }
}

/// Run initial schema migration (version 1)
/// Creates all tables and indexes as defined in schema.rs
pub fn run_initial_migration(conn: &Connection) -> MigrationResult<()> {
    // Check if already applied (always version 1 for initial schema)
    if is_migration_applied(conn, 1)? {
        info!("Migration version 1 (initial_schema) already applied, skipping");
        return Ok(());
    }

    info!("Running initial schema migration (version 1)");

    // Execute schema creation in a transaction
    conn.execute_batch(CREATE_TABLES_SQL)
        .map_err(|e| MigrationError::Database(e.to_string()))?;

    // Record migration
    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![1, "initial_schema", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Initial schema migration completed successfully");
    Ok(())
}

/// Marker struct for legacy data migration status
#[derive(Default)]
pub struct LegacyMigrationStatus {
    pub ui_favorites_migrated: usize,
    pub checkin_providers_migrated: usize,
    pub checkin_accounts_migrated: usize,
    pub checkin_records_migrated: usize,
    pub checkin_balances_migrated: usize,
    pub checkin_waf_cookies_migrated: usize,
    pub log_entries_migrated: usize,
}

impl std::fmt::Display for LegacyMigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Migrated: {} favorites, {} providers, {} accounts, {} records, {} balances, {} waf_cookies, {} logs",
            self.ui_favorites_migrated,
            self.checkin_providers_migrated,
            self.checkin_accounts_migrated,
            self.checkin_records_migrated,
            self.checkin_balances_migrated,
            self.checkin_waf_cookies_migrated,
            self.log_entries_migrated
        )
    }
}

/// Check if legacy data migration marker exists
/// Returns true if migration was already performed
pub fn is_legacy_migration_done(conn: &Connection) -> MigrationResult<bool> {
    let result: Result<i32, _> = conn.query_row(
        "SELECT COUNT(*) FROM migrations WHERE name = 'legacy_json_import'",
        [],
        |row| row.get(0),
    );

    match result {
        Ok(count) => Ok(count > 0),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        Err(e) => Err(MigrationError::Database(e.to_string())),
    }
}

/// Mark legacy migration as complete
pub fn mark_legacy_migration_done(conn: &Connection) -> MigrationResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO migrations (version, name, applied_at) VALUES (?, ?, ?)",
        rusqlite::params![0, "legacy_json_import", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;
    Ok(())
}

/// Legacy JSON file paths (relative to home directory)
pub mod legacy_paths {
    pub const UI_STATE: &str = ".ccr/ui_state.json";
    pub const CHECKIN_PROVIDERS: &str = ".ccr/checkin/providers.json";
    pub const CHECKIN_ACCOUNTS: &str = ".ccr/checkin/accounts.json";
    pub const CHECKIN_RECORDS: &str = ".ccr/checkin/records.json";
    pub const CHECKIN_BALANCES: &str = ".ccr/checkin/balances.json";
    pub const CHECKIN_WAF_COOKIES: &str = ".ccr/checkin/waf_cookies.json";
    pub const LOG_DIR: &str = ".ccr/logs";
}

/// Check if any legacy JSON files exist
pub fn has_legacy_data(home_dir: &Path) -> bool {
    let paths = [
        legacy_paths::UI_STATE,
        legacy_paths::CHECKIN_PROVIDERS,
        legacy_paths::CHECKIN_ACCOUNTS,
        legacy_paths::CHECKIN_RECORDS,
        legacy_paths::CHECKIN_BALANCES,
        legacy_paths::CHECKIN_WAF_COOKIES,
    ];

    for path in paths {
        if home_dir.join(path).exists() {
            return true;
        }
    }

    // Check for log files
    let log_dir = home_dir.join(legacy_paths::LOG_DIR);
    if log_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&log_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                return true;
            }
        }
    }

    false
}

// 鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺?
// Legacy Data Structures (for JSON deserialization)
// 鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺?

/// Legacy UI State JSON structure
#[derive(Debug, Deserialize)]
struct LegacyUiState {
    #[serde(default)]
    favorites: Vec<FavoriteCommand>,
    // history is skipped per design.md
}

/// Legacy balance snapshot for JSON import
/// Supports both old format (balance/total_consumed) and new format (total_quota/used_quota/remaining_quota)
#[derive(Debug, Serialize, Deserialize)]
struct LegacyBalanceSnapshot {
    account_id: String,
    /// Old format: single balance value
    #[serde(default)]
    balance: Option<f64>,
    /// Currency/unit
    #[serde(default)]
    currency: Option<String>,
    /// Total quota (new format)
    #[serde(default)]
    total_quota: Option<f64>,
    /// Total consumed/used quota
    #[serde(default, alias = "total_consumed")]
    used_quota: Option<f64>,
    /// Remaining quota (new format)
    #[serde(default)]
    remaining_quota: Option<f64>,
    /// Record timestamp (supports both checked_at and recorded_at)
    #[serde(default, alias = "recorded_at")]
    checked_at: Option<DateTime<Utc>>,
}

/// Legacy WAF cookie for JSON import
/// NOTE: Not imported due to format incompatibility (account_id vs provider_id)
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct LegacyWafCookie {
    account_id: String,
    waf_cookie: String,
    expires_at: DateTime<Utc>,
}

// 鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺?
// Legacy Data Import Functions
// 鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺愨晲鈺?

/// Import legacy JSON data into SQLite
/// Skips UI history per design.md requirement
pub fn import_legacy_data(
    conn: &Connection,
    home_dir: &Path,
) -> MigrationResult<LegacyMigrationStatus> {
    // Import all legacy data and construct status in one go
    let status = LegacyMigrationStatus {
        ui_favorites_migrated: import_ui_favorites(conn, home_dir)?,
        checkin_providers_migrated: import_checkin_providers(conn, home_dir)?,
        checkin_accounts_migrated: import_checkin_accounts(conn, home_dir)?,
        checkin_records_migrated: import_checkin_records(conn, home_dir)?,
        checkin_balances_migrated: import_checkin_balances(conn, home_dir)?,
        checkin_waf_cookies_migrated: import_waf_cookies(conn, home_dir)?,
        log_entries_migrated: 0, // Logs are not migrated - they are runtime generated
    };

    Ok(status)
}

/// Import UI favorites from legacy JSON (skip history)
fn import_ui_favorites(conn: &Connection, home_dir: &Path) -> MigrationResult<usize> {
    let path = home_dir.join(legacy_paths::UI_STATE);
    if !path.exists() {
        debug!("No legacy UI state file found at {:?}", path);
        return Ok(0);
    }

    let content = fs::read_to_string(&path).map_err(|e| MigrationError::Io(e.to_string()))?;
    let legacy_state: LegacyUiState = match serde_json::from_str(&content) {
        Ok(state) => state,
        Err(e) => {
            warn!("Failed to parse legacy UI state: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;
    for favorite in legacy_state.favorites {
        match ui_state_repo::insert_favorite(conn, &favorite) {
            Ok(_) => count += 1,
            Err(e) => {
                warn!("Failed to import favorite {}: {}", favorite.id, e);
            }
        }
    }

    info!(
        "Imported {} UI favorites (history skipped per design)",
        count
    );
    Ok(count)
}

/// Import checkin providers from legacy JSON
fn import_checkin_providers(conn: &Connection, home_dir: &Path) -> MigrationResult<usize> {
    let path = home_dir.join(legacy_paths::CHECKIN_PROVIDERS);
    if !path.exists() {
        debug!("No legacy providers file found at {:?}", path);
        return Ok(0);
    }

    let content = fs::read_to_string(&path).map_err(|e| MigrationError::Io(e.to_string()))?;
    let providers: Vec<CheckinProvider> = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to parse legacy providers: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;
    for provider in providers {
        // Use INSERT OR IGNORE to avoid duplicates
        match checkin_repo::insert_provider(conn, &provider) {
            Ok(_) => count += 1,
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                debug!("Provider {} already exists, skipping", provider.id);
            }
            Err(e) => {
                warn!("Failed to import provider {}: {}", provider.id, e);
            }
        }
    }

    info!("Imported {} checkin providers", count);
    Ok(count)
}

/// Import checkin accounts from legacy JSON
fn import_checkin_accounts(conn: &Connection, home_dir: &Path) -> MigrationResult<usize> {
    let path = home_dir.join(legacy_paths::CHECKIN_ACCOUNTS);
    if !path.exists() {
        debug!("No legacy accounts file found at {:?}", path);
        return Ok(0);
    }

    let content = fs::read_to_string(&path).map_err(|e| MigrationError::Io(e.to_string()))?;
    let accounts: Vec<CheckinAccount> = match serde_json::from_str(&content) {
        Ok(a) => a,
        Err(e) => {
            warn!("Failed to parse legacy accounts: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;
    for account in accounts {
        // Use INSERT OR IGNORE to avoid duplicates
        match checkin_repo::insert_account(conn, &account) {
            Ok(_) => count += 1,
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                debug!("Account {} already exists, skipping", account.id);
            }
            Err(e) => {
                warn!("Failed to import account {}: {}", account.id, e);
            }
        }
    }

    info!("Imported {} checkin accounts", count);
    Ok(count)
}

/// Import checkin records from legacy JSON
fn import_checkin_records(conn: &Connection, home_dir: &Path) -> MigrationResult<usize> {
    let path = home_dir.join(legacy_paths::CHECKIN_RECORDS);
    if !path.exists() {
        debug!("No legacy records file found at {:?}", path);
        return Ok(0);
    }

    let content = fs::read_to_string(&path).map_err(|e| MigrationError::Io(e.to_string()))?;
    let records: Vec<CheckinRecord> = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            warn!("Failed to parse legacy records: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;
    for record in records {
        match checkin_repo::insert_record(conn, &record) {
            Ok(_) => count += 1,
            Err(e) => {
                warn!("Failed to import record {}: {}", record.id, e);
            }
        }
    }

    info!("Imported {} checkin records", count);
    Ok(count)
}

/// Import checkin balances from legacy JSON
fn import_checkin_balances(conn: &Connection, home_dir: &Path) -> MigrationResult<usize> {
    let path = home_dir.join(legacy_paths::CHECKIN_BALANCES);
    if !path.exists() {
        debug!("No legacy balances file found at {:?}", path);
        return Ok(0);
    }

    let content = fs::read_to_string(&path).map_err(|e| MigrationError::Io(e.to_string()))?;
    let balances: Vec<LegacyBalanceSnapshot> = match serde_json::from_str(&content) {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to parse legacy balances: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;
    for legacy in balances {
        // Convert legacy format to new BalanceSnapshot format
        // - Old format uses "balance" for remaining balance
        // - New format uses total_quota/used_quota/remaining_quota
        let total_quota = legacy.total_quota.unwrap_or(0.0);
        let used_quota = legacy.used_quota.unwrap_or(0.0);
        let remaining_quota = legacy.remaining_quota.unwrap_or({
            // Fallback: use legacy balance field or compute from total - used
            legacy.balance.unwrap_or({
                if total_quota > 0.0 {
                    total_quota - used_quota
                } else {
                    0.0
                }
            })
        });
        let currency = legacy.currency.unwrap_or_else(|| "USD".to_string());
        let recorded_at = legacy.checked_at.unwrap_or_else(Utc::now);

        let snapshot = BalanceSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: legacy.account_id,
            total_quota,
            used_quota,
            remaining_quota,
            currency,
            recorded_at,
        };

        match checkin_repo::insert_balance(conn, &snapshot) {
            Ok(_) => count += 1,
            Err(e) => {
                warn!(
                    "Failed to import balance for {}: {}",
                    snapshot.account_id, e
                );
            }
        }
    }

    info!("Imported {} balance snapshots", count);
    Ok(count)
}

/// Import WAF cookies from legacy JSON
/// NOTE: Legacy WAF cookies used account_id but new schema uses provider_id
/// This import is skipped as the format is incompatible (cookies are ephemeral anyway)
fn import_waf_cookies(_conn: &Connection, home_dir: &Path) -> MigrationResult<usize> {
    let path = home_dir.join(legacy_paths::CHECKIN_WAF_COOKIES);
    if !path.exists() {
        debug!("No legacy WAF cookies file found at {:?}", path);
        return Ok(0);
    }

    // WAF cookies format changed: legacy uses account_id, new uses provider_id
    // Since cookies are ephemeral and short-lived, we skip migration
    // They will be re-fetched when needed
    warn!(
        "Legacy WAF cookies found at {:?} but migration is skipped (format incompatible)",
        path
    );
    info!("WAF cookies will be re-fetched on next provider access");
    Ok(0)
}

/// Run migration v2: Add extra_config column to checkin_accounts
/// Stores CDK credentials, OAuth tokens, and other extensible config as JSON
pub fn run_migration_v2(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 2)? {
        debug!("Migration v2 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v2: add extra_config to checkin_accounts");

    // ALTER TABLE to add extra_config column for existing databases
    // New databases already have this column from CREATE_TABLES_SQL
    conn.execute_batch(
        "ALTER TABLE checkin_accounts ADD COLUMN extra_config TEXT NOT NULL DEFAULT '{}'",
    )
    .or_else(|e| {
        // Column may already exist if DB was freshly created with v2 schema
        if e.to_string().contains("duplicate column name") {
            debug!("Column extra_config already exists, skipping ALTER TABLE");
            Ok(())
        } else {
            Err(e)
        }
    })
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    // Record migration
    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![2, "add_extra_config", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v2 completed successfully");
    Ok(())
}

/// Run migration v3: Add extracted columns to usage_records + model_pricing table
/// Enables efficient aggregation queries without JSON parsing
pub fn run_migration_v3(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 3)? {
        debug!("Migration v3 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v3: usage_records extracted columns + model_pricing");

    // 涓?usage_records 娣诲姞鎻愬彇鍒楋紙骞傜瓑锛氬拷鐣?duplicate column 閿欒锛?
    let alter_stmts = [
        "ALTER TABLE usage_records ADD COLUMN model TEXT",
        "ALTER TABLE usage_records ADD COLUMN input_tokens INTEGER DEFAULT 0",
        "ALTER TABLE usage_records ADD COLUMN output_tokens INTEGER DEFAULT 0",
        "ALTER TABLE usage_records ADD COLUMN cache_read_tokens INTEGER DEFAULT 0",
        "ALTER TABLE usage_records ADD COLUMN cost_usd REAL DEFAULT 0",
    ];
    for stmt in &alter_stmts {
        conn.execute_batch(stmt)
            .or_else(|e| {
                if e.to_string().contains("duplicate column name") {
                    debug!("Column already exists, skipping: {}", stmt);
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }

    // 鍒涘缓绱㈠紩
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_usage_records_model ON usage_records (model);
         CREATE INDEX IF NOT EXISTS idx_usage_records_recorded_at ON usage_records (recorded_at);",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    // 鍒涘缓 model_pricing 琛?
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS model_pricing (
            model_id TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            input_cost_per_million REAL NOT NULL,
            output_cost_per_million REAL NOT NULL,
            cache_read_cost_per_million REAL NOT NULL DEFAULT 0
        );",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    // 棰勭疆妯″瀷浠锋牸
    conn.execute_batch(
        "INSERT OR IGNORE INTO model_pricing VALUES ('claude-sonnet-4-20250514','Claude Sonnet 4',3,15,0.3);
         INSERT OR IGNORE INTO model_pricing VALUES ('claude-opus-4-20250514','Claude Opus 4',15,75,1.5);
         INSERT OR IGNORE INTO model_pricing VALUES ('claude-haiku-3-5-20241022','Claude Haiku 3.5',0.8,4,0.08);
         INSERT OR IGNORE INTO model_pricing VALUES ('gpt-4.1','GPT-4.1',2,8,0.5);
         INSERT OR IGNORE INTO model_pricing VALUES ('gemini-2.5-pro','Gemini 2.5 Pro',1.25,10,0.315);
         INSERT OR IGNORE INTO model_pricing VALUES ('gemini-2.5-flash','Gemini 2.5 Flash',0.15,0.6,0.0375);",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    // 鍥炲～鐜版湁璁板綍锛氫粠 record_json 鎻愬彇瀛楁
    backfill_usage_records(conn)?;

    // 璁板綍杩佺Щ
    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![3, "usage_extracted_columns", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v3 completed successfully");
    Ok(())
}

/// 鍥炲～ usage_records 鐨勬彁鍙栧垪锛堜粠 record_json 瑙ｆ瀽锛?
fn backfill_usage_records(conn: &Connection) -> MigrationResult<()> {
    // 璇诲彇鎵€鏈夐渶瑕佸洖濉殑璁板綍锛坢odel 涓?NULL 鐨勶級
    let mut select_stmt = conn
        .prepare("SELECT id, record_json FROM usage_records WHERE model IS NULL")
        .map_err(|e| MigrationError::Database(e.to_string()))?;

    let rows: Vec<(String, String)> = select_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| MigrationError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        return Ok(());
    }

    info!(
        "Backfilling {} usage records with extracted fields",
        rows.len()
    );

    // 鍔犺浇瀹氫环琛?
    let mut pricing_stmt = conn
        .prepare(
            "SELECT model_id, input_cost_per_million, output_cost_per_million, cache_read_cost_per_million FROM model_pricing",
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    let pricing: std::collections::HashMap<String, (f64, f64, f64)> = pricing_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                (
                    row.get::<_, f64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                ),
            ))
        })
        .map_err(|e| MigrationError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    let mut update_stmt = conn
        .prepare(
            "UPDATE usage_records SET model=?1, input_tokens=?2, output_tokens=?3, cache_read_tokens=?4, cost_usd=?5 WHERE id=?6",
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;

    for (id, json_str) in &rows {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
            // 鎻愬彇 model
            let model = json
                .get("model")
                .or_else(|| json.get("message").and_then(|m| m.get("model")))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // 鎻愬彇 usage
            let usage = json
                .get("usage")
                .or_else(|| json.get("message").and_then(|m| m.get("usage")));

            let (input, output, cache) = if let Some(u) = usage {
                (
                    u.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
                    u.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
                    u.get("cache_read_input_tokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
            } else {
                (0, 0, 0)
            };

            // 璁＄畻璐圭敤锛氬尮閰嶅畾浠疯〃锛堟ā绯婂尮閰嶅墠缂€锛?
            let cost = pricing
                .iter()
                .find(|(k, _)| model.starts_with(k.as_str()) || k.starts_with(model))
                .map(|(_, (ic, oc, cc))| {
                    (input as f64 * ic + output as f64 * oc + cache as f64 * cc) / 1_000_000.0
                })
                .unwrap_or(0.0);

            let _ = update_stmt.execute(rusqlite::params![model, input, output, cache, cost, id]);
        }
    }

    info!("Backfill complete");
    Ok(())
}

/// Run migration v4: Add composite indexes for usage analytics pagination/filtering
pub fn run_migration_v4(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 4)? {
        debug!("Migration v4 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v4: add usage composite indexes");
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_usage_records_platform_model_recorded_at_id
             ON usage_records (platform, model, recorded_at DESC, id DESC);
         CREATE INDEX IF NOT EXISTS idx_usage_records_platform_recorded_at_id
             ON usage_records (platform, recorded_at DESC, id DESC);",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![4, "usage_composite_indexes", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v4 completed successfully");
    Ok(())
}

/// Run migration v5: Create usage_daily_agg pre-aggregation table
/// Enables fast heatmap and trend queries without scanning usage_records
pub fn run_migration_v5(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 5)? {
        debug!("Migration v5 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v5: usage_daily_agg pre-aggregation table");

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS usage_daily_agg (
            date TEXT NOT NULL,
            platform TEXT NOT NULL,
            request_count INTEGER DEFAULT 0,
            input_tokens INTEGER DEFAULT 0,
            output_tokens INTEGER DEFAULT 0,
            cache_read_tokens INTEGER DEFAULT 0,
            cost_usd REAL DEFAULT 0,
            PRIMARY KEY (date, platform)
        );",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    // Backfill from existing usage_records
    conn.execute_batch(
        "INSERT OR REPLACE INTO usage_daily_agg (date, platform, request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd)
         SELECT DATE(recorded_at), platform, COUNT(*),
                COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cost_usd),0)
         FROM usage_records
         GROUP BY DATE(recorded_at), platform;",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![5, "usage_daily_agg", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v5 completed successfully");
    Ok(())
}

/// Run migration v6: Add SSH host and known_hosts tables
pub fn run_migration_v6(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 6)? {
        debug!("Migration v6 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v6: ssh hosts and known hosts tables");

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS ssh_hosts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 22,
            username TEXT NOT NULL,
            identity_file TEXT,
            remote_home TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_connected_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_ssh_hosts_updated_at
            ON ssh_hosts (updated_at DESC);

        CREATE INDEX IF NOT EXISTS idx_ssh_hosts_host_port
            ON ssh_hosts (host, port);

        CREATE TABLE IF NOT EXISTS ssh_known_hosts (
            host TEXT NOT NULL,
            port INTEGER NOT NULL,
            key_type TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            confirmed_at TEXT NOT NULL,
            PRIMARY KEY (host, port)
        );

        CREATE INDEX IF NOT EXISTS idx_ssh_known_hosts_confirmed_at
            ON ssh_known_hosts (confirmed_at DESC);",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![6, "ssh_hosts_tables", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v6 completed successfully");
    Ok(())
}

fn table_has_column(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> MigrationResult<bool> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| MigrationError::Database(e.to_string()))?;

    while let Some(row) = rows
        .next()
        .map_err(|e| MigrationError::Database(e.to_string()))?
    {
        let name: String = row
            .get(1)
            .map_err(|e| MigrationError::Database(e.to_string()))?;
        if name == column_name {
            return Ok(true);
        }
    }

    Ok(false)
}
/// Run migration v7: extend log_entries for monitoring feed
pub fn run_migration_v7(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 7)? {
        debug!("Migration v7 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v7: monitoring log columns");

    if !table_has_column(conn, "log_entries", "channel")? {
        conn.execute("ALTER TABLE log_entries ADD COLUMN channel TEXT", [])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    if !table_has_column(conn, "log_entries", "event_type")? {
        conn.execute("ALTER TABLE log_entries ADD COLUMN event_type TEXT", [])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    if !table_has_column(conn, "log_entries", "correlation_id")? {
        conn.execute("ALTER TABLE log_entries ADD COLUMN correlation_id TEXT", [])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_log_entries_channel ON log_entries (channel)",
        [],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![7, "monitoring_log_columns", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v7 completed successfully");
    Ok(())
}

/// Run migration v8: add error_code column to checkin_records
pub fn run_migration_v8(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 8)? {
        debug!("Migration v8 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v8: checkin_records error_code column");

    if !table_has_column(conn, "checkin_records", "error_code")? {
        conn.execute("ALTER TABLE checkin_records ADD COLUMN error_code TEXT", [])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }

    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![8, "checkin_records_error_code", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v8 completed successfully");
    Ok(())
}

/// Run migration v9: Add claude_profiles table for Claude Code profile management
pub fn run_migration_v9(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 9)? {
        debug!("Migration v9 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v9: claude_profiles table");

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS claude_profiles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            snapshot_json TEXT NOT NULL,
            tags TEXT,
            is_current INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_claude_profiles_name
            ON claude_profiles (name);

        CREATE INDEX IF NOT EXISTS idx_claude_profiles_is_current
            ON claude_profiles (is_current);",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![9, "claude_profiles_table", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v9 completed successfully");
    Ok(())
}

/// Run migration v10: remove raw_response column from checkin_balances
pub fn run_migration_v10(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 10)? {
        debug!("Migration v10 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v10: remove raw_response from checkin_balances");

    if table_has_column(conn, "checkin_balances", "raw_response")? {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE checkin_balances_new (
                 id TEXT PRIMARY KEY,
                 account_id TEXT NOT NULL,
                 total_quota REAL NOT NULL,
                 used_quota REAL NOT NULL,
                 remaining_quota REAL NOT NULL,
                 currency TEXT NOT NULL,
                 recorded_at TEXT NOT NULL
             );

             INSERT INTO checkin_balances_new (
                 id, account_id, total_quota, used_quota, remaining_quota, currency, recorded_at
             )
             SELECT
                 id, account_id, total_quota, used_quota, remaining_quota, currency, recorded_at
             FROM checkin_balances;

             DROP TABLE checkin_balances;
             ALTER TABLE checkin_balances_new RENAME TO checkin_balances;

             CREATE INDEX IF NOT EXISTS idx_checkin_balances_account_id
                 ON checkin_balances (account_id);
             CREATE INDEX IF NOT EXISTS idx_checkin_balances_recorded_at
                 ON checkin_balances (recorded_at DESC);
             COMMIT;",
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    }

    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![10, "checkin_balances_remove_raw_response", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v10 completed successfully");
    Ok(())
}

/// Run migration v11: extend usage archive metadata and durable checkpoint/session tables
pub fn run_migration_v11(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 11)? {
        debug!("Migration v11 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v11: usage archive durability tables");

    if !table_has_column(conn, "usage_sources", "source_state")? {
        conn.execute(
            "ALTER TABLE usage_sources ADD COLUMN source_state TEXT NOT NULL DEFAULT 'live'",
            [],
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    if !table_has_column(conn, "usage_sources", "file_size")? {
        conn.execute("ALTER TABLE usage_sources ADD COLUMN file_size INTEGER", [])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    if !table_has_column(conn, "usage_sources", "modified_at")? {
        conn.execute("ALTER TABLE usage_sources ADD COLUMN modified_at TEXT", [])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    if !table_has_column(conn, "usage_sources", "last_seen_at")? {
        conn.execute("ALTER TABLE usage_sources ADD COLUMN last_seen_at TEXT", [])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    if !table_has_column(conn, "usage_sources", "raw_deleted_at")? {
        conn.execute(
            "ALTER TABLE usage_sources ADD COLUMN raw_deleted_at TEXT",
            [],
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    }

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_usage_sources_platform_state
             ON usage_sources (platform, source_state);

         CREATE TABLE IF NOT EXISTS usage_history_cursor (
             platform TEXT PRIMARY KEY,
             recent_window_days INTEGER NOT NULL DEFAULT 30,
             last_history_file_path TEXT,
             last_history_file_modified_at TEXT,
             last_history_offset INTEGER NOT NULL DEFAULT 0,
             recent_completed_at TEXT,
             history_completed_at TEXT,
             updated_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS usage_codex_checkpoint (
             source_id TEXT PRIMARY KEY,
             session_id TEXT NOT NULL,
             project_path TEXT NOT NULL,
             model TEXT,
             last_line_number INTEGER NOT NULL DEFAULT 0,
             input_tokens INTEGER NOT NULL DEFAULT 0,
             cached_input_tokens INTEGER NOT NULL DEFAULT 0,
             output_tokens INTEGER NOT NULL DEFAULT 0,
             prefers_turn_completed INTEGER NOT NULL DEFAULT 0,
             updated_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS usage_session_archive (
             archive_id TEXT PRIMARY KEY,
             session_id TEXT NOT NULL,
             platform TEXT NOT NULL,
             title TEXT,
             cwd TEXT NOT NULL,
             file_path TEXT NOT NULL,
             file_hash TEXT,
             message_count INTEGER NOT NULL DEFAULT 0,
             created_at TEXT NOT NULL,
             updated_at TEXT NOT NULL,
             source_state TEXT NOT NULL DEFAULT 'live',
             last_seen_at TEXT,
             raw_deleted_at TEXT,
             archived_at TEXT NOT NULL
         );

         CREATE UNIQUE INDEX IF NOT EXISTS idx_usage_session_archive_file_path
             ON usage_session_archive (file_path);
         CREATE INDEX IF NOT EXISTS idx_usage_session_archive_platform_created_at
             ON usage_session_archive (platform, created_at DESC);
         CREATE INDEX IF NOT EXISTS idx_usage_session_archive_platform_state
             ON usage_session_archive (platform, source_state);",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        params![11, "usage_archive_durability_tables", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v11 completed successfully");
    Ok(())
}

/// Run migration v13: add catalog-backed usage pricing fields and reprice usage records.
pub fn run_migration_v13(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 13)? && has_usage_pricing_columns(conn)? {
        debug!("Migration v13 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v13: usage pricing catalog reprice");

    add_usage_pricing_column(
        conn,
        "cache_creation_tokens",
        "ALTER TABLE usage_records ADD COLUMN cache_creation_tokens INTEGER DEFAULT 0",
    )?;
    add_usage_pricing_column(
        conn,
        "cost_with_cache_usd",
        "ALTER TABLE usage_records ADD COLUMN cost_with_cache_usd REAL DEFAULT 0",
    )?;
    add_usage_pricing_column(
        conn,
        "cost_without_cache_usd",
        "ALTER TABLE usage_records ADD COLUMN cost_without_cache_usd REAL DEFAULT 0",
    )?;
    add_usage_pricing_column(
        conn,
        "pricing_status",
        "ALTER TABLE usage_records ADD COLUMN pricing_status TEXT NOT NULL DEFAULT 'unpriced'",
    )?;
    add_usage_pricing_column(
        conn,
        "pricing_source",
        "ALTER TABLE usage_records ADD COLUMN pricing_source TEXT",
    )?;

    reprice_usage_records(conn)?;
    refresh_usage_daily_agg(conn)?;

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO migrations (version, name, applied_at) VALUES (?, ?, ?)",
        rusqlite::params![13, "usage_pricing_catalog_reprice", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v13 completed successfully");
    Ok(())
}

fn has_usage_pricing_columns(conn: &Connection) -> MigrationResult<bool> {
    for column in [
        "cache_creation_tokens",
        "cost_with_cache_usd",
        "cost_without_cache_usd",
        "pricing_status",
        "pricing_source",
    ] {
        if !table_has_column(conn, "usage_records", column)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn add_usage_pricing_column(
    conn: &Connection,
    column_name: &str,
    alter_sql: &str,
) -> MigrationResult<()> {
    if !table_has_column(conn, "usage_records", column_name)? {
        conn.execute(alter_sql, [])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    Ok(())
}

fn reprice_usage_records(conn: &Connection) -> MigrationResult<()> {
    let catalog = ModelRateCatalog::official();
    let mut stmt = conn
        .prepare(
            "SELECT id, COALESCE(model, 'unknown'), input_tokens, output_tokens,
                    cache_read_tokens, cache_creation_tokens, record_json
             FROM usage_records",
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2).unwrap_or(0),
                row.get::<_, i64>(3).unwrap_or(0),
                row.get::<_, i64>(4).unwrap_or(0),
                row.get::<_, i64>(5).unwrap_or(0),
                row.get::<_, String>(6).unwrap_or_default(),
            ))
        })
        .map_err(|e| MigrationError::Database(e.to_string()))?
        .filter_map(|row| row.ok())
        .collect::<Vec<_>>();

    drop(stmt);

    let mut update_stmt = conn
        .prepare(
            "UPDATE usage_records
             SET cache_creation_tokens = ?1,
                 cost_usd = ?2,
                 cost_with_cache_usd = ?3,
                 cost_without_cache_usd = ?4,
                 pricing_status = ?5,
                 pricing_source = ?6
             WHERE id = ?7",
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;

    for (id, model, input, output, cache_read, stored_cache_creation, record_json) in rows {
        let cache_creation = if stored_cache_creation > 0 {
            stored_cache_creation
        } else {
            extract_cache_creation_tokens_from_json(&record_json)
        };
        let pricing = catalog.calculate(&model, input, output, cache_read, cache_creation);

        update_stmt
            .execute(rusqlite::params![
                cache_creation,
                pricing.cost_with_cache_usd,
                pricing.cost_with_cache_usd,
                pricing.cost_without_cache_usd,
                pricing.pricing_status,
                pricing.pricing_source,
                id
            ])
            .map_err(|e| MigrationError::Database(e.to_string()))?;
    }

    Ok(())
}

fn extract_cache_creation_tokens_from_json(record_json: &str) -> i64 {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(record_json) else {
        return 0;
    };

    let usage = json
        .get("usage")
        .or_else(|| json.get("message").and_then(|message| message.get("usage")));

    usage
        .and_then(|usage| {
            usage
                .get("cache_creation_input_tokens")
                .or_else(|| usage.get("cache_creation_tokens"))
                .or_else(|| usage.get("cache_write_input_tokens"))
                .and_then(|value| value.as_i64())
        })
        .unwrap_or(0)
}

fn refresh_usage_daily_agg(conn: &Connection) -> MigrationResult<()> {
    conn.execute("DELETE FROM usage_daily_agg", [])
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    conn.execute_batch(
        "INSERT OR REPLACE INTO usage_daily_agg (
             date, platform, request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd
         )
         SELECT DATE(recorded_at), platform, COUNT(*),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(cost_usd), 0)
         FROM usage_records
         GROUP BY DATE(recorded_at), platform;",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;
    Ok(())
}

/*
 * ========================================================================
 * 步骤14：claude_observer 模块持久化结构
 * ========================================================================
 * 目标表：
 * 1) user_settings              通用键值表，承载订阅模式/月费等 UI 偏好
 * 2) claude_tool_calls          Claude Code JSONL 中的工具调用事件
 * 3) claude_tool_calls_ingest_state
 *                                每个 jsonl 文件的增量游标 (mtime + offset)
 * 操作：
 * 1) 一次性建好三表，CREATE TABLE IF NOT EXISTS 保证可重入
 * 2) 给热点查询加复合索引（按时间窗口聚合 + 按 tool_name 排序）
 */
pub fn run_migration_v14(conn: &Connection) -> MigrationResult<()> {
    if is_migration_applied(conn, 14)? {
        debug!("Migration v14 already applied, skipping");
        return Ok(());
    }

    info!("Running migration v14: claude_observer tables");

    // 14.1 通用键值表（首批用于订阅模式 / 月费）
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS user_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    // 14.2 工具调用事实表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS claude_tool_calls (
            session_id TEXT NOT NULL,
            seq INTEGER NOT NULL,
            ts TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            success INTEGER,
            duration_ms INTEGER,
            cost_usd REAL,
            project_path TEXT,
            PRIMARY KEY (session_id, seq)
        );

        CREATE INDEX IF NOT EXISTS idx_claude_tool_calls_ts
            ON claude_tool_calls (ts DESC);
        CREATE INDEX IF NOT EXISTS idx_claude_tool_calls_tool_name_ts
            ON claude_tool_calls (tool_name, ts DESC);
        CREATE INDEX IF NOT EXISTS idx_claude_tool_calls_project_ts
            ON claude_tool_calls (project_path, ts DESC);",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    // 14.3 jsonl 文件增量游标
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS claude_tool_calls_ingest_state (
            file_path TEXT PRIMARY KEY,
            file_mtime_ns INTEGER NOT NULL,
            last_offset INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        );",
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    // 14.4 写入迁移标记
    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![14, "claude_observer_tables", now],
    )
    .map_err(|e| MigrationError::Database(e.to_string()))?;

    info!("Migration v14 completed successfully");
    Ok(())
}

fn table_exists(conn: &Connection, table_name: &str) -> MigrationResult<bool> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table_name],
            |row| row.get(0),
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    Ok(count > 0)
}

/// Run all migrations (schema + legacy data import)
/// This is the main entry point called during initialization
pub fn run_all_migrations(conn: &Connection, home_dir: &Path) -> MigrationResult<()> {
    // Step 1: Run schema migration (v1 - initial tables)
    run_initial_migration(conn)?;

    // Step 1.5: Run v2 migration (extra_config column)
    run_migration_v2(conn)?;

    // Step 1.6: Run v3 migration (usage extracted columns + model_pricing)
    run_migration_v3(conn)?;

    // Step 1.7: Run v4 migration (usage composite indexes)
    run_migration_v4(conn)?;

    // Step 1.8: Run v5 migration (usage daily aggregation table)
    run_migration_v5(conn)?;

    // Step 1.9: Run v6 migration (ssh host and known_hosts tables)
    run_migration_v6(conn)?;

    // Step 1.10: Run v7 migration (monitoring log columns)
    run_migration_v7(conn)?;

    // Step 1.11: Run v8 migration (checkin_records error_code column)
    run_migration_v8(conn)?;

    // Step 1.12: Run v9 migration (claude_profiles table)
    run_migration_v9(conn)?;

    // Step 1.13: Run v10 migration (drop raw_response from checkin_balances)
    run_migration_v10(conn)?;

    // Step 1.14: Run v11 migration (usage archive durability tables)
    run_migration_v11(conn)?;

    // Step 1.15: Run v14 migration (claude_observer tables: user_settings, claude_tool_calls)
    run_migration_v14(conn)?;

    // Step 2: Import legacy data if not done and files exist
    if !is_legacy_migration_done(conn)? {
        if has_legacy_data(home_dir) {
            info!("Legacy JSON files detected, starting migration...");

            match import_legacy_data(conn, home_dir) {
                Ok(status) => {
                    info!("Legacy data migration completed: {}", status);
                    mark_legacy_migration_done(conn)?;
                }
                Err(e) => {
                    error!("Legacy data migration failed: {}", e);
                    // Don't mark as done so it can be retried
                    return Err(e);
                }
            }
        } else {
            info!("No legacy JSON files found, marking migration as complete");
            mark_legacy_migration_done(conn)?;
        }
    } else {
        debug!("Legacy data migration already completed");
    }

    // Step 3: Recalculate usage costs after all legacy/live rows are present.
    run_migration_v13(conn)?;

    Ok(())
}

pub fn migrate_usage_archive_from_legacy_dbs(
    conn: &Connection,
    home_dir: &Path,
    legacy_ui_db_path: &Path,
) -> MigrationResult<()> {
    const LEGACY_USAGE_IMPORT_VERSION: i32 = 1001;
    const LEGACY_SESSION_SEED_VERSION: i32 = 1002;

    /*
     * ========================================================================
     * 步骤1：迁移旧 ccr-ui usage 数据
     * ========================================================================
     * 目标：
     * 1) 将 ~/.ccr-ui/ccr-ui.db 中既有的 usage 表搬迁到新的 archive 库
     * 2) 仅迁移 usage 相关表，不触碰旧库中的其他 UI/checkin/ssh 数据
     */
    info!("开始迁移旧 usage archive 数据...");
    if !is_migration_applied(conn, LEGACY_USAGE_IMPORT_VERSION)? {
        if legacy_ui_db_path.exists() {
            let legacy_conn = Connection::open(legacy_ui_db_path)
                .map_err(|e| MigrationError::Database(e.to_string()))?;

            if table_exists(&legacy_conn, "usage_sources")? {
                let tx = conn
                    .unchecked_transaction()
                    .map_err(|e| MigrationError::Database(e.to_string()))?;

                {
                    let mut stmt = legacy_conn
                        .prepare(
                            "SELECT id, platform, file_path, file_hash, last_offset, updated_at
                             FROM usage_sources",
                        )
                        .map_err(|e| MigrationError::Database(e.to_string()))?;
                    let rows = stmt
                        .query_map([], |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, String>(2)?,
                                row.get::<_, String>(3)?,
                                row.get::<_, i64>(4)?,
                                row.get::<_, String>(5)?,
                            ))
                        })
                        .map_err(|e| MigrationError::Database(e.to_string()))?;

                    for row in rows {
                        let (id, platform, file_path, file_hash, last_offset, updated_at) =
                            row.map_err(|e| MigrationError::Database(e.to_string()))?;
                        tx.execute(
                            "INSERT OR IGNORE INTO usage_sources (
                                id, platform, file_path, file_hash, last_offset,
                                source_state, file_size, modified_at, last_seen_at, raw_deleted_at, updated_at
                             ) VALUES (?1, ?2, ?3, ?4, ?5, 'live', NULL, NULL, ?6, NULL, ?6)",
                            params![id, platform, file_path, file_hash, last_offset, updated_at],
                        )
                        .map_err(|e| MigrationError::Database(e.to_string()))?;
                    }
                }

                if table_exists(&legacy_conn, "usage_records")? {
                    let mut stmt = legacy_conn
                        .prepare(
                            "SELECT id, platform, project_path, record_json, recorded_at, source_id,
                                    model, input_tokens, output_tokens, cache_read_tokens, cost_usd
                             FROM usage_records",
                        )
                        .map_err(|e| MigrationError::Database(e.to_string()))?;
                    let rows = stmt
                        .query_map([], |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, String>(2)?,
                                row.get::<_, String>(3)?,
                                row.get::<_, String>(4)?,
                                row.get::<_, String>(5)?,
                                row.get::<_, Option<String>>(6)?,
                                row.get::<_, i64>(7).unwrap_or(0),
                                row.get::<_, i64>(8).unwrap_or(0),
                                row.get::<_, i64>(9).unwrap_or(0),
                                row.get::<_, f64>(10).unwrap_or(0.0),
                            ))
                        })
                        .map_err(|e| MigrationError::Database(e.to_string()))?;

                    for row in rows {
                        let (
                            id,
                            platform,
                            project_path,
                            record_json,
                            recorded_at,
                            source_id,
                            model,
                            input_tokens,
                            output_tokens,
                            cache_read_tokens,
                            cost_usd,
                        ) = row.map_err(|e| MigrationError::Database(e.to_string()))?;
                        tx.execute(
                            "INSERT OR IGNORE INTO usage_records (
                                id, platform, project_path, record_json, recorded_at, source_id,
                                model, input_tokens, output_tokens, cache_read_tokens, cost_usd
                             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                            params![
                                id,
                                platform,
                                project_path,
                                record_json,
                                recorded_at,
                                source_id,
                                model,
                                input_tokens,
                                output_tokens,
                                cache_read_tokens,
                                cost_usd
                            ],
                        )
                        .map_err(|e| MigrationError::Database(e.to_string()))?;
                    }
                }

                if table_exists(&legacy_conn, "usage_daily_agg")? {
                    let mut stmt = legacy_conn
                        .prepare(
                            "SELECT date, platform, request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd
                             FROM usage_daily_agg",
                        )
                        .map_err(|e| MigrationError::Database(e.to_string()))?;
                    let rows = stmt
                        .query_map([], |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, i64>(2)?,
                                row.get::<_, i64>(3)?,
                                row.get::<_, i64>(4)?,
                                row.get::<_, i64>(5)?,
                                row.get::<_, f64>(6)?,
                            ))
                        })
                        .map_err(|e| MigrationError::Database(e.to_string()))?;

                    for row in rows {
                        let (
                            date,
                            platform,
                            request_count,
                            input_tokens,
                            output_tokens,
                            cache_read_tokens,
                            cost_usd,
                        ) = row.map_err(|e| MigrationError::Database(e.to_string()))?;
                        tx.execute(
                            "INSERT OR IGNORE INTO usage_daily_agg (
                                date, platform, request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd
                             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            params![
                                date,
                                platform,
                                request_count,
                                input_tokens,
                                output_tokens,
                                cache_read_tokens,
                                cost_usd
                            ],
                        )
                        .map_err(|e| MigrationError::Database(e.to_string()))?;
                    }
                }

                tx.commit()
                    .map_err(|e| MigrationError::Database(e.to_string()))?;
            }
        }

        conn.execute(
            INSERT_MIGRATION_SQL,
            params![
                LEGACY_USAGE_IMPORT_VERSION,
                "usage_archive_import_from_legacy_ui_db",
                Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    info!("旧 usage archive 数据迁移完成");

    /*
     * ========================================================================
     * 步骤2：播种最小 session 摘要归档
     * ========================================================================
     * 目标：
     * 1) 从 ~/.ccr/data.db 的 sessions 表恢复最小 session 摘要
     * 2) 为首页概览提供 durable archive，不再依赖原始 session 文件仍然存在
     */
    info!("开始播种 session 摘要归档...");
    if !is_migration_applied(conn, LEGACY_SESSION_SEED_VERSION)? {
        let session_db_path = home_dir.join(".ccr").join("data.db");
        if session_db_path.exists() {
            let session_conn = Connection::open(&session_db_path)
                .map_err(|e| MigrationError::Database(e.to_string()))?;

            if table_exists(&session_conn, "sessions")? {
                let archived_at = Utc::now().to_rfc3339();
                let tx = conn
                    .unchecked_transaction()
                    .map_err(|e| MigrationError::Database(e.to_string()))?;
                let mut stmt = session_conn
                    .prepare(
                        "SELECT id, platform, title, cwd, file_path, file_hash, created_at, updated_at, message_count, indexed_at
                         FROM sessions",
                    )
                    .map_err(|e| MigrationError::Database(e.to_string()))?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5)?,
                            row.get::<_, String>(6)?,
                            row.get::<_, String>(7)?,
                            row.get::<_, i64>(8)?,
                            row.get::<_, String>(9)?,
                        ))
                    })
                    .map_err(|e| MigrationError::Database(e.to_string()))?;

                for row in rows {
                    let (
                        session_id,
                        platform,
                        title,
                        cwd,
                        file_path,
                        file_hash,
                        created_at,
                        updated_at,
                        message_count,
                        indexed_at,
                    ) = row.map_err(|e| MigrationError::Database(e.to_string()))?;
                    let source_state = if Path::new(&file_path).exists() {
                        "live"
                    } else {
                        "missing"
                    };
                    let raw_deleted_at = if source_state == "missing" {
                        Some(Utc::now().to_rfc3339())
                    } else {
                        None
                    };

                    tx.execute(
                        "INSERT OR IGNORE INTO usage_session_archive (
                            archive_id, session_id, platform, title, cwd, file_path, file_hash,
                            message_count, created_at, updated_at, source_state, last_seen_at, raw_deleted_at, archived_at
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                        params![
                            format!("{platform}:{session_id}:{file_path}"),
                            session_id,
                            platform,
                            title,
                            cwd,
                            file_path,
                            file_hash,
                            message_count,
                            created_at,
                            updated_at,
                            source_state,
                            indexed_at,
                            raw_deleted_at,
                            archived_at
                        ],
                    )
                    .map_err(|e| MigrationError::Database(e.to_string()))?;
                }

                tx.commit()
                    .map_err(|e| MigrationError::Database(e.to_string()))?;
            }
        }

        conn.execute(
            INSERT_MIGRATION_SQL,
            params![
                LEGACY_SESSION_SEED_VERSION,
                "usage_session_archive_seed_from_ccr_store",
                Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;
    }
    info!("session 摘要归档播种完成");

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use tempfile::TempDir;

    fn create_legacy_usage_db(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE usage_sources (
                 id TEXT PRIMARY KEY,
                 platform TEXT NOT NULL,
                 file_path TEXT NOT NULL,
                 file_hash TEXT NOT NULL,
                 last_offset INTEGER NOT NULL DEFAULT 0,
                 updated_at TEXT NOT NULL
             );
             CREATE TABLE usage_records (
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
             CREATE TABLE usage_daily_agg (
                 date TEXT NOT NULL,
                 platform TEXT NOT NULL,
                 request_count INTEGER DEFAULT 0,
                 input_tokens INTEGER DEFAULT 0,
                 output_tokens INTEGER DEFAULT 0,
                 cache_read_tokens INTEGER DEFAULT 0,
                 cost_usd REAL DEFAULT 0,
                 PRIMARY KEY (date, platform)
             );",
        )
        .unwrap();

        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO usage_sources (id, platform, file_path, file_hash, last_offset, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                "legacy-source-1",
                "codex",
                "C:/tmp/legacy-rollout.jsonl",
                "hash-legacy",
                256_i64,
                now
            ],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO usage_records (
                id, platform, project_path, record_json, recorded_at, source_id,
                model, input_tokens, output_tokens, cache_read_tokens, cost_usd
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                "legacy-record-1",
                "codex",
                "D:/Documents/Code/Github/ccr",
                "{\"model\":\"gpt-5.4\"}",
                Utc::now().to_rfc3339(),
                "legacy-source-1",
                "gpt-5.4",
                120_i64,
                40_i64,
                20_i64,
                1.25_f64
            ],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO usage_daily_agg (
                date, platform, request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["2026-04-20", "codex", 1_i64, 120_i64, 40_i64, 20_i64, 1.25_f64],
        )
        .unwrap();
    }

    fn create_legacy_session_store(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE sessions (
                 id TEXT PRIMARY KEY,
                 platform TEXT NOT NULL,
                 title TEXT,
                 cwd TEXT NOT NULL,
                 file_path TEXT NOT NULL,
                 file_hash TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 updated_at TEXT NOT NULL,
                 message_count INTEGER NOT NULL DEFAULT 0,
                 indexed_at TEXT NOT NULL
             );",
        )
        .unwrap();

        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sessions (
                id, platform, title, cwd, file_path, file_hash, created_at, updated_at, message_count, indexed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                "session-1",
                "codex",
                "Archived Codex Session",
                "D:/Documents/Code/Github/ccr",
                "C:/tmp/deleted-session.jsonl",
                "file-hash-1",
                now,
                now,
                7_i64,
                now
            ],
        )
        .unwrap();
    }

    #[test]
    fn test_initial_migration() {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"ui_favorites".to_string()));
        assert!(tables.contains(&"ui_history".to_string()));
        assert!(tables.contains(&"checkin_providers".to_string()));
        assert!(tables.contains(&"checkin_accounts".to_string()));
        assert!(tables.contains(&"checkin_records".to_string()));
        assert!(tables.contains(&"checkin_balances".to_string()));
        assert!(tables.contains(&"checkin_waf_cookies".to_string()));
        assert!(tables.contains(&"log_entries".to_string()));
        assert!(tables.contains(&"usage_sources".to_string()));
        assert!(tables.contains(&"usage_records".to_string()));
        assert!(tables.contains(&"migrations".to_string()));
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run twice - should not fail
        run_initial_migration(&conn).unwrap();
        run_initial_migration(&conn).unwrap();

        // Only one migration record
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migration_v2() {
        let conn = Connection::open_in_memory().unwrap();

        // Run v1 first
        run_initial_migration(&conn).unwrap();

        // Run v2
        run_migration_v2(&conn).unwrap();

        // Verify v2 migration recorded
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 2",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify extra_config column exists by inserting a test account
        conn.execute(
            "INSERT INTO checkin_accounts (id, provider_id, name, cookies_json_encrypted, api_user, enabled, created_at, extra_config)
             VALUES ('test', 'p1', 'Test', 'enc', 'user', 1, '2024-01-01T00:00:00Z', '{\"cdk_type\":\"test\"}')",
            [],
        )
        .unwrap();

        let extra: String = conn
            .query_row(
                "SELECT extra_config FROM checkin_accounts WHERE id = 'test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(extra, r#"{"cdk_type":"test"}"#);
    }

    #[test]
    fn test_migration_v2_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();

        // Run v2 twice - should not fail
        run_migration_v2(&conn).unwrap();
        run_migration_v2(&conn).unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 2",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migration_v13_reprices_usage_records_and_handles_legacy_v12_marker() {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();

        conn.execute("DROP TABLE usage_records", []).unwrap();
        conn.execute_batch(
            "CREATE TABLE usage_records (
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
             );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO migrations (version, name, applied_at)
             VALUES (12, 'usage_record_json_min_snapshot', ?1)",
            params![Utc::now().to_rfc3339()],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO usage_records (
                id, platform, project_path, record_json, recorded_at, source_id,
                model, input_tokens, output_tokens, cache_read_tokens, cost_usd
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                "opus-record",
                "claude",
                "D:/Documents/Code/Github/ccr",
                "{\"usage\":{\"cache_creation_input_tokens\":1000000}}",
                "2026-04-20T08:00:00Z",
                "source-opus",
                "claude-opus-4-6",
                1_000_000_i64,
                1_000_000_i64,
                1_000_000_i64,
                99.0_f64
            ],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO usage_daily_agg (
                date, platform, request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                "2026-04-20",
                "claude",
                1_i64,
                1_i64,
                1_i64,
                1_i64,
                99.0_f64
            ],
        )
        .unwrap();

        run_migration_v13(&conn).unwrap();

        let (cache_creation, cost, with_cache, no_cache, status, source): (
            i64,
            f64,
            f64,
            f64,
            String,
            String,
        ) = conn
            .query_row(
                "SELECT cache_creation_tokens, cost_usd, cost_with_cache_usd,
                        cost_without_cache_usd, pricing_status, pricing_source
                 FROM usage_records WHERE id = 'opus-record'",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .unwrap();

        assert_eq!(cache_creation, 1_000_000);
        assert!((cost - 36.75).abs() < 0.000_001);
        assert!((with_cache - 36.75).abs() < 0.000_001);
        assert!((no_cache - 40.0).abs() < 0.000_001);
        assert_eq!(status, "priced");
        assert_eq!(source, "official:anthropic");

        let daily_cost: f64 = conn
            .query_row(
                "SELECT cost_usd FROM usage_daily_agg WHERE date = '2026-04-20' AND platform = 'claude'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!((daily_cost - 36.75).abs() < 0.000_001);

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 13",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migration_v7_handles_fresh_schema() {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();

        run_migration_v7(&conn).unwrap();

        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(log_entries)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|row| row.unwrap())
            .collect();

        assert!(columns.contains(&"channel".to_string()));
        assert!(columns.contains(&"event_type".to_string()));
        assert!(columns.contains(&"correlation_id".to_string()));

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 7",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migration_v7_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();

        run_migration_v7(&conn).unwrap();
        run_migration_v7(&conn).unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 7",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
    #[test]
    fn test_legacy_migration_marker() {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();

        // Initially not done
        assert!(!is_legacy_migration_done(&conn).unwrap());

        // Mark as done
        mark_legacy_migration_done(&conn).unwrap();
        assert!(is_legacy_migration_done(&conn).unwrap());

        // Idempotent
        mark_legacy_migration_done(&conn).unwrap();
        assert!(is_legacy_migration_done(&conn).unwrap());
    }

    #[test]
    fn test_migration_v10_removes_raw_response_column() {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();

        conn.execute_batch(
            "DROP TABLE checkin_balances;
             CREATE TABLE checkin_balances (
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
                 ON checkin_balances (recorded_at DESC);",
        )
        .unwrap();

        conn.execute(
            "INSERT INTO checkin_balances (
                id, account_id, total_quota, used_quota, remaining_quota, currency, raw_response, recorded_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                "balance-1",
                "account-1",
                100.0_f64,
                25.0_f64,
                75.0_f64,
                "USD",
                "{\"secret\":true}",
                Utc::now().to_rfc3339(),
            ],
        )
        .unwrap();

        run_migration_v10(&conn).unwrap();

        assert!(!table_has_column(&conn, "checkin_balances", "raw_response").unwrap());

        let remaining: f64 = conn
            .query_row(
                "SELECT remaining_quota FROM checkin_balances WHERE id = 'balance-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 75.0);

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 10",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migrate_usage_archive_from_legacy_dbs_is_idempotent() {
        let home = TempDir::new().unwrap();
        let legacy_ui_db_path = home.path().join(".ccr-ui").join("ccr-ui.db");
        let legacy_session_db_path = home.path().join(".ccr").join("data.db");
        create_legacy_usage_db(&legacy_ui_db_path);
        create_legacy_session_store(&legacy_session_db_path);

        let archive_conn = Connection::open_in_memory().unwrap();
        run_all_migrations(&archive_conn, home.path()).unwrap();

        migrate_usage_archive_from_legacy_dbs(&archive_conn, home.path(), &legacy_ui_db_path)
            .unwrap();
        migrate_usage_archive_from_legacy_dbs(&archive_conn, home.path(), &legacy_ui_db_path)
            .unwrap();

        let usage_source_count: i64 = archive_conn
            .query_row("SELECT COUNT(*) FROM usage_sources", [], |row| row.get(0))
            .unwrap();
        let usage_record_count: i64 = archive_conn
            .query_row("SELECT COUNT(*) FROM usage_records", [], |row| row.get(0))
            .unwrap();
        let usage_daily_count: i64 = archive_conn
            .query_row("SELECT COUNT(*) FROM usage_daily_agg", [], |row| row.get(0))
            .unwrap();
        let session_archive_count: i64 = archive_conn
            .query_row("SELECT COUNT(*) FROM usage_session_archive", [], |row| {
                row.get(0)
            })
            .unwrap();
        let source_state: String = archive_conn
            .query_row(
                "SELECT source_state FROM usage_sources WHERE id = 'legacy-source-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let session_state: String = archive_conn
            .query_row(
                "SELECT source_state FROM usage_session_archive WHERE session_id = 'session-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let session_deleted_at: Option<String> = archive_conn
            .query_row(
                "SELECT raw_deleted_at FROM usage_session_archive WHERE session_id = 'session-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let usage_marker_count: i64 = archive_conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 1001",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let session_marker_count: i64 = archive_conn
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 1002",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(usage_source_count, 1);
        assert_eq!(usage_record_count, 1);
        assert_eq!(usage_daily_count, 1);
        assert_eq!(session_archive_count, 1);
        assert_eq!(source_state, "live");
        assert_eq!(session_state, "missing");
        assert!(session_deleted_at.is_some());
        assert_eq!(usage_marker_count, 1);
        assert_eq!(session_marker_count, 1);
    }
}
