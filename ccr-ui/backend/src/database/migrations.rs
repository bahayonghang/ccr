// Database migrations for unified SQLite storage
// Handles schema creation and data migration from legacy JSON files

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

use super::repositories::{checkin_repo, ui_state_repo};
use super::schema::{CREATE_TABLES_SQL, INSERT_MIGRATION_SQL, SCHEMA_VERSION};
use crate::models::checkin::balance::BalanceSnapshot;
use crate::models::checkin::{CheckinAccount, CheckinProvider, CheckinRecord};
use crate::models::ui_state::FavoriteCommand;

/// Result type for migration operations
pub type MigrationResult<T> = Result<T, MigrationError>;

/// Migration-specific errors
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[allow(dead_code)]
    #[error("Migration already applied: version {0}")]
    AlreadyApplied(i32),
}

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
        Err(e) => Err(e.into()),
    }
}

/// Run initial schema migration (version 1)
/// Creates all tables and indexes as defined in schema.rs
pub fn run_initial_migration(conn: &Connection) -> MigrationResult<()> {
    // Check if already applied
    if is_migration_applied(conn, SCHEMA_VERSION)? {
        info!(
            "Migration version {} already applied, skipping",
            SCHEMA_VERSION
        );
        return Ok(());
    }

    info!(
        "Running initial schema migration (version {})",
        SCHEMA_VERSION
    );

    // Execute schema creation in a transaction
    conn.execute_batch(CREATE_TABLES_SQL)?;

    // Record migration
    let now = Utc::now().to_rfc3339();
    conn.execute(
        INSERT_MIGRATION_SQL,
        rusqlite::params![SCHEMA_VERSION, "initial_schema", now],
    )?;

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
        Err(e) => Err(e.into()),
    }
}

/// Mark legacy migration as complete
pub fn mark_legacy_migration_done(conn: &Connection) -> MigrationResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO migrations (version, name, applied_at) VALUES (?, ?, ?)",
        rusqlite::params![0, "legacy_json_import", now],
    )?;
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

// ═══════════════════════════════════════════════════════════
// Legacy Data Structures (for JSON deserialization)
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// Legacy Data Import Functions
// ═══════════════════════════════════════════════════════════

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

    let content = fs::read_to_string(&path)?;
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

    let content = fs::read_to_string(&path)?;
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

    let content = fs::read_to_string(&path)?;
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

    let content = fs::read_to_string(&path)?;
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

    let content = fs::read_to_string(&path)?;
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
            raw_response: None,
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

/// Run all migrations (schema + legacy data import)
/// This is the main entry point called during initialization
pub fn run_all_migrations(conn: &Connection, home_dir: &Path) -> MigrationResult<()> {
    // Step 1: Run schema migration
    run_initial_migration(conn)?;

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

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rusqlite::Connection;

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
}
