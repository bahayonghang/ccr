// Checkin Repository - SQLite data access layer for checkin module
// Replaces JSON file storage for providers, accounts, records, and balances

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use tracing::debug;

use crate::models::checkin::{
    account::CheckinAccount,
    balance::BalanceSnapshot,
    provider::CheckinProvider,
    record::{CheckinRecord, CheckinStatus},
};

// ═══════════════════════════════════════════════════════════
// Provider Operations
// ═══════════════════════════════════════════════════════════

/// Insert a new provider
pub fn insert_provider(
    conn: &Connection,
    provider: &CheckinProvider,
) -> Result<(), rusqlite::Error> {
    let created_at = provider.created_at.to_rfc3339();
    let updated_at = provider.updated_at.map(|dt| dt.to_rfc3339());

    conn.execute(
        "INSERT INTO checkin_providers (id, name, base_url, checkin_path, balance_path,
         user_info_path, auth_header, auth_prefix, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            provider.id,
            provider.name,
            provider.base_url,
            provider.checkin_path,
            provider.balance_path,
            provider.user_info_path,
            provider.auth_header,
            provider.auth_prefix,
            if provider.enabled { 1 } else { 0 },
            created_at,
            updated_at,
        ],
    )?;

    debug!("Inserted provider: {} ({})", provider.name, provider.id);
    Ok(())
}

/// Get all providers
pub fn get_all_providers(conn: &Connection) -> Result<Vec<CheckinProvider>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, base_url, checkin_path, balance_path, user_info_path,
                auth_header, auth_prefix, enabled, created_at, updated_at
         FROM checkin_providers
         ORDER BY name ASC",
    )?;

    let providers = stmt
        .query_map([], row_to_provider)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(providers)
}

/// Get provider by ID
pub fn get_provider_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<CheckinProvider>, rusqlite::Error> {
    conn.query_row(
        "SELECT id, name, base_url, checkin_path, balance_path, user_info_path,
                auth_header, auth_prefix, enabled, created_at, updated_at
         FROM checkin_providers WHERE id = ?1",
        params![id],
        row_to_provider,
    )
    .optional()
}

/// Update provider
pub fn update_provider(
    conn: &Connection,
    provider: &CheckinProvider,
) -> Result<bool, rusqlite::Error> {
    let updated_at = Utc::now().to_rfc3339();

    let affected = conn.execute(
        "UPDATE checkin_providers SET
         name = ?1, base_url = ?2, checkin_path = ?3, balance_path = ?4,
         user_info_path = ?5, auth_header = ?6, auth_prefix = ?7, enabled = ?8, updated_at = ?9
         WHERE id = ?10",
        params![
            provider.name,
            provider.base_url,
            provider.checkin_path,
            provider.balance_path,
            provider.user_info_path,
            provider.auth_header,
            provider.auth_prefix,
            if provider.enabled { 1 } else { 0 },
            updated_at,
            provider.id,
        ],
    )?;

    Ok(affected > 0)
}

/// Delete provider by ID
pub fn delete_provider(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
    let deleted = conn.execute("DELETE FROM checkin_providers WHERE id = ?1", params![id])?;
    Ok(deleted > 0)
}

// ═══════════════════════════════════════════════════════════
// Account Operations
// ═══════════════════════════════════════════════════════════

/// Insert a new account
pub fn insert_account(conn: &Connection, account: &CheckinAccount) -> Result<(), rusqlite::Error> {
    let created_at = account.created_at.to_rfc3339();
    let updated_at = account.updated_at.map(|dt| dt.to_rfc3339());
    let last_checkin_at = account.last_checkin_at.map(|dt| dt.to_rfc3339());
    let last_balance_check_at = account.last_balance_check_at.map(|dt| dt.to_rfc3339());

    conn.execute(
        "INSERT INTO checkin_accounts (id, provider_id, name, cookies_json_encrypted, api_user,
         enabled, created_at, updated_at, last_checkin_at, last_balance_check_at, extra_config)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            account.id,
            account.provider_id,
            account.name,
            account.cookies_json_encrypted,
            account.api_user,
            if account.enabled { 1 } else { 0 },
            created_at,
            updated_at,
            last_checkin_at,
            last_balance_check_at,
            account.extra_config,
        ],
    )?;

    debug!("Inserted account: {} ({})", account.name, account.id);
    Ok(())
}

/// Get all accounts
pub fn get_all_accounts(conn: &Connection) -> Result<Vec<CheckinAccount>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, name, cookies_json_encrypted, api_user, enabled,
                created_at, updated_at, last_checkin_at, last_balance_check_at, extra_config
         FROM checkin_accounts
         ORDER BY created_at DESC",
    )?;

    let accounts = stmt
        .query_map([], row_to_account)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(accounts)
}

/// Get accounts by provider ID
pub fn get_accounts_by_provider(
    conn: &Connection,
    provider_id: &str,
) -> Result<Vec<CheckinAccount>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, name, cookies_json_encrypted, api_user, enabled,
                created_at, updated_at, last_checkin_at, last_balance_check_at, extra_config
         FROM checkin_accounts
         WHERE provider_id = ?1
         ORDER BY created_at DESC",
    )?;

    let accounts = stmt
        .query_map(params![provider_id], row_to_account)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(accounts)
}

/// Get enabled accounts
pub fn get_enabled_accounts(conn: &Connection) -> Result<Vec<CheckinAccount>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, name, cookies_json_encrypted, api_user, enabled,
                created_at, updated_at, last_checkin_at, last_balance_check_at, extra_config
         FROM checkin_accounts
         WHERE enabled = 1
         ORDER BY created_at DESC",
    )?;

    let accounts = stmt
        .query_map([], row_to_account)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(accounts)
}

/// Get account by ID
pub fn get_account_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<CheckinAccount>, rusqlite::Error> {
    conn.query_row(
        "SELECT id, provider_id, name, cookies_json_encrypted, api_user, enabled,
                created_at, updated_at, last_checkin_at, last_balance_check_at, extra_config
         FROM checkin_accounts WHERE id = ?1",
        params![id],
        row_to_account,
    )
    .optional()
}

/// Update account
pub fn update_account(
    conn: &Connection,
    account: &CheckinAccount,
) -> Result<bool, rusqlite::Error> {
    let updated_at = Utc::now().to_rfc3339();
    let last_checkin_at = account.last_checkin_at.map(|dt| dt.to_rfc3339());
    let last_balance_check_at = account.last_balance_check_at.map(|dt| dt.to_rfc3339());

    let affected = conn.execute(
        "UPDATE checkin_accounts SET
         provider_id = ?1, name = ?2, cookies_json_encrypted = ?3, api_user = ?4,
         enabled = ?5, updated_at = ?6, last_checkin_at = ?7, last_balance_check_at = ?8,
         extra_config = ?9
         WHERE id = ?10",
        params![
            account.provider_id,
            account.name,
            account.cookies_json_encrypted,
            account.api_user,
            if account.enabled { 1 } else { 0 },
            updated_at,
            last_checkin_at,
            last_balance_check_at,
            account.extra_config,
            account.id,
        ],
    )?;

    Ok(affected > 0)
}

/// Delete account by ID
pub fn delete_account(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
    let deleted = conn.execute("DELETE FROM checkin_accounts WHERE id = ?1", params![id])?;
    Ok(deleted > 0)
}

/// Delete accounts by provider ID
#[allow(dead_code)]
pub fn delete_accounts_by_provider(
    conn: &Connection,
    provider_id: &str,
) -> Result<usize, rusqlite::Error> {
    let deleted = conn.execute(
        "DELETE FROM checkin_accounts WHERE provider_id = ?1",
        params![provider_id],
    )?;
    Ok(deleted)
}

// ═══════════════════════════════════════════════════════════
// Record Operations
// ═══════════════════════════════════════════════════════════

/// Insert a new checkin record
pub fn insert_record(conn: &Connection, record: &CheckinRecord) -> Result<(), rusqlite::Error> {
    let checked_in_at = record.checked_in_at.to_rfc3339();
    let status_str = match record.status {
        CheckinStatus::Success => "success",
        CheckinStatus::AlreadyCheckedIn => "already_checked_in",
        CheckinStatus::Failed => "failed",
    };

    conn.execute(
        "INSERT INTO checkin_records (id, account_id, status, message, reward,
         balance_before, balance_after, checked_in_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record.id,
            record.account_id,
            status_str,
            record.message,
            record.reward,
            record.balance_before,
            record.balance_after,
            checked_in_at,
        ],
    )?;

    debug!(
        "Inserted checkin record: {} ({})",
        record.account_id, record.id
    );
    Ok(())
}

/// Get records by account ID with limit
pub fn get_records_by_account(
    conn: &Connection,
    account_id: &str,
    limit: usize,
) -> Result<Vec<CheckinRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, status, message, reward, balance_before, balance_after, checked_in_at
         FROM checkin_records
         WHERE account_id = ?1
         ORDER BY checked_in_at DESC
         LIMIT ?2",
    )?;

    let records = stmt
        .query_map(params![account_id, limit as i64], row_to_record)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(records)
}

/// Get all records with limit
pub fn get_all_records(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<CheckinRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, status, message, reward, balance_before, balance_after, checked_in_at
         FROM checkin_records
         ORDER BY checked_in_at DESC
         LIMIT ?1",
    )?;

    let records = stmt
        .query_map(params![limit as i64], row_to_record)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(records)
}

/// Get records with SQL-level pagination and optional filters
/// Returns (records, total_count)
#[allow(dead_code)]
pub fn get_records_paginated(
    conn: &Connection,
    status: Option<&str>,
    account_id: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<(Vec<CheckinRecord>, usize), rusqlite::Error> {
    // Build dynamic WHERE clause
    let mut conditions = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(s) = status {
        conditions.push(format!("status = ?{}", param_values.len() + 1));
        param_values.push(Box::new(s.to_string()));
    }
    if let Some(aid) = account_id {
        conditions.push(format!("account_id = ?{}", param_values.len() + 1));
        param_values.push(Box::new(aid.to_string()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Count total
    let count_sql = format!("SELECT COUNT(*) FROM checkin_records {}", where_clause);
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let total: i64 = conn.query_row(&count_sql, params_ref.as_slice(), |row| row.get(0))?;
    let total = total as usize;

    // Fetch page
    let offset = (page.saturating_sub(1)) * page_size;
    let select_sql = format!(
        "SELECT id, account_id, status, message, reward, balance_before, balance_after, checked_in_at
         FROM checkin_records {}
         ORDER BY checked_in_at DESC
         LIMIT ?{} OFFSET ?{}",
        where_clause,
        param_values.len() + 1,
        param_values.len() + 2,
    );

    let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = param_values;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));
    let all_params_ref: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&select_sql)?;
    let records = stmt
        .query_map(all_params_ref.as_slice(), row_to_record)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok((records, total))
}

/// Get records with SQL-level pagination and advanced optional filters.
/// Supports provider_id and keyword search without falling back to in-memory filtering.
/// Returns (records, total_count).
pub fn get_records_paginated_advanced(
    conn: &Connection,
    status: Option<&str>,
    account_id: Option<&str>,
    provider_id: Option<&str>,
    keyword: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<(Vec<CheckinRecord>, usize), rusqlite::Error> {
    let mut conditions = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(s) = status {
        conditions.push(format!("r.status = ?{}", param_values.len() + 1));
        param_values.push(Box::new(s.to_string()));
    }
    if let Some(aid) = account_id {
        conditions.push(format!("r.account_id = ?{}", param_values.len() + 1));
        param_values.push(Box::new(aid.to_string()));
    }
    if let Some(pid) = provider_id {
        conditions.push(format!("a.provider_id = ?{}", param_values.len() + 1));
        param_values.push(Box::new(pid.to_string()));
    }
    if let Some(raw_keyword) = keyword.map(str::trim).filter(|v| !v.is_empty()) {
        let keyword_pattern = format!("%{}%", raw_keyword.to_lowercase());
        let p1 = param_values.len() + 1;
        let p2 = p1 + 1;
        let p3 = p2 + 1;
        let p4 = p3 + 1;
        conditions.push(format!(
            "(LOWER(r.account_id) LIKE ?{p1} OR LOWER(COALESCE(r.message, '')) LIKE ?{p2} OR LOWER(a.name) LIKE ?{p3} OR LOWER(COALESCE(p.name, '')) LIKE ?{p4})"
        ));
        param_values.push(Box::new(keyword_pattern.clone()));
        param_values.push(Box::new(keyword_pattern.clone()));
        param_values.push(Box::new(keyword_pattern.clone()));
        param_values.push(Box::new(keyword_pattern));
    }

    let from_clause = "FROM checkin_records r
         LEFT JOIN checkin_accounts a ON a.id = r.account_id
         LEFT JOIN checkin_providers p ON p.id = a.provider_id";
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) {} {}", from_clause, where_clause);
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let total: i64 = conn.query_row(&count_sql, params_ref.as_slice(), |row| row.get(0))?;
    let total = total as usize;

    let offset = (page.saturating_sub(1)) * page_size;
    let select_sql = format!(
        "SELECT r.id, r.account_id, r.status, r.message, r.reward, r.balance_before, r.balance_after, r.checked_in_at
         {} {}
         ORDER BY r.checked_in_at DESC
         LIMIT ?{} OFFSET ?{}",
        from_clause,
        where_clause,
        param_values.len() + 1,
        param_values.len() + 2,
    );

    let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = param_values;
    all_params.push(Box::new(page_size as i64));
    all_params.push(Box::new(offset as i64));
    let all_params_ref: Vec<&dyn rusqlite::types::ToSql> =
        all_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&select_sql)?;
    let records = stmt
        .query_map(all_params_ref.as_slice(), row_to_record)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok((records, total))
}

/// Get all records with advanced optional filters (no pagination).
pub fn get_records_filtered_advanced(
    conn: &Connection,
    status: Option<&str>,
    account_id: Option<&str>,
    provider_id: Option<&str>,
    keyword: Option<&str>,
) -> Result<Vec<CheckinRecord>, rusqlite::Error> {
    let mut conditions = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(s) = status {
        conditions.push(format!("r.status = ?{}", param_values.len() + 1));
        param_values.push(Box::new(s.to_string()));
    }
    if let Some(aid) = account_id {
        conditions.push(format!("r.account_id = ?{}", param_values.len() + 1));
        param_values.push(Box::new(aid.to_string()));
    }
    if let Some(pid) = provider_id {
        conditions.push(format!("a.provider_id = ?{}", param_values.len() + 1));
        param_values.push(Box::new(pid.to_string()));
    }
    if let Some(raw_keyword) = keyword.map(str::trim).filter(|v| !v.is_empty()) {
        let keyword_pattern = format!("%{}%", raw_keyword.to_lowercase());
        let p1 = param_values.len() + 1;
        let p2 = p1 + 1;
        let p3 = p2 + 1;
        let p4 = p3 + 1;
        conditions.push(format!(
            "(LOWER(r.account_id) LIKE ?{p1} OR LOWER(COALESCE(r.message, '')) LIKE ?{p2} OR LOWER(a.name) LIKE ?{p3} OR LOWER(COALESCE(p.name, '')) LIKE ?{p4})"
        ));
        param_values.push(Box::new(keyword_pattern.clone()));
        param_values.push(Box::new(keyword_pattern.clone()));
        param_values.push(Box::new(keyword_pattern.clone()));
        param_values.push(Box::new(keyword_pattern));
    }

    let from_clause = "FROM checkin_records r
         LEFT JOIN checkin_accounts a ON a.id = r.account_id
         LEFT JOIN checkin_providers p ON p.id = a.provider_id";
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "SELECT r.id, r.account_id, r.status, r.message, r.reward, r.balance_before, r.balance_after, r.checked_in_at
         {} {}
         ORDER BY r.checked_in_at DESC",
        from_clause, where_clause
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let records = stmt
        .query_map(params_ref.as_slice(), row_to_record)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(records)
}

/// Aggregate today's status for a set of accounts in a single query.
/// Returns (checked_in_count, failed_count).
pub fn get_today_status_counts(
    conn: &Connection,
    account_ids: &[String],
) -> Result<(usize, usize), rusqlite::Error> {
    if account_ids.is_empty() {
        return Ok((0, 0));
    }

    let today_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("Invalid time: 00:00:00");
    let today_start_str = today_start.and_utc().to_rfc3339();

    let placeholders = (0..account_ids.len())
        .map(|idx| format!("?{}", idx + 2))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "SELECT r.account_id,
                MAX(CASE
                      WHEN r.status IN ('success', 'already_checked_in') THEN 2
                      WHEN r.status = 'failed' THEN 1
                      ELSE 0
                    END) AS state
         FROM checkin_records r
         WHERE r.checked_in_at >= ?1
           AND r.account_id IN ({})
         GROUP BY r.account_id",
        placeholders
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> =
        Vec::with_capacity(account_ids.len() + 1);
    params.push(Box::new(today_start_str));
    for account_id in account_ids {
        params.push(Box::new(account_id.clone()));
    }
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_ref.as_slice(), |row| row.get::<_, i64>(1))?;

    let mut checked_in = 0usize;
    let mut failed = 0usize;
    for state in rows {
        match state? {
            2 => checked_in += 1,
            1 => failed += 1,
            _ => {}
        }
    }

    Ok((checked_in, failed))
}

/// Get today's records for an account
pub fn get_today_records(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<CheckinRecord>, rusqlite::Error> {
    let today_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("Invalid time: 00:00:00");
    let today_start_str = today_start.and_utc().to_rfc3339();

    let mut stmt = conn.prepare(
        "SELECT id, account_id, status, message, reward, balance_before, balance_after, checked_in_at
         FROM checkin_records
         WHERE account_id = ?1 AND checked_in_at >= ?2
         ORDER BY checked_in_at DESC",
    )?;

    let records = stmt
        .query_map(params![account_id, today_start_str], |row| {
            row_to_record(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(records)
}

/// Delete records by account ID
pub fn delete_records_by_account(
    conn: &Connection,
    account_id: &str,
) -> Result<usize, rusqlite::Error> {
    let deleted = conn.execute(
        "DELETE FROM checkin_records WHERE account_id = ?1",
        params![account_id],
    )?;
    Ok(deleted)
}

// ═══════════════════════════════════════════════════════════
// Balance Operations
// ═══════════════════════════════════════════════════════════

/// Insert a new balance snapshot
pub fn insert_balance(conn: &Connection, balance: &BalanceSnapshot) -> Result<(), rusqlite::Error> {
    let recorded_at = balance.recorded_at.to_rfc3339();

    conn.execute(
        "INSERT INTO checkin_balances (id, account_id, total_quota, used_quota, remaining_quota,
         currency, raw_response, recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            balance.id,
            balance.account_id,
            balance.total_quota,
            balance.used_quota,
            balance.remaining_quota,
            balance.currency,
            balance.raw_response,
            recorded_at,
        ],
    )?;

    debug!(
        "Inserted balance snapshot: {} ({})",
        balance.account_id, balance.id
    );
    Ok(())
}

/// Get latest balance for an account
pub fn get_latest_balance(
    conn: &Connection,
    account_id: &str,
) -> Result<Option<BalanceSnapshot>, rusqlite::Error> {
    conn.query_row(
        "SELECT id, account_id, total_quota, used_quota, remaining_quota, currency, raw_response, recorded_at
         FROM checkin_balances
         WHERE account_id = ?1
         ORDER BY recorded_at DESC
         LIMIT 1",
        params![account_id],
        row_to_balance,
    )
    .optional()
}

/// Get balance history for an account
pub fn get_balance_history(
    conn: &Connection,
    account_id: &str,
    limit: usize,
) -> Result<Vec<BalanceSnapshot>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, total_quota, used_quota, remaining_quota, currency, raw_response, recorded_at
         FROM checkin_balances
         WHERE account_id = ?1
         ORDER BY recorded_at DESC
         LIMIT ?2",
    )?;

    let balances = stmt
        .query_map(params![account_id, limit as i64], row_to_balance)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(balances)
}

/// Delete balances by account ID
pub fn delete_balances_by_account(
    conn: &Connection,
    account_id: &str,
) -> Result<usize, rusqlite::Error> {
    let deleted = conn.execute(
        "DELETE FROM checkin_balances WHERE account_id = ?1",
        params![account_id],
    )?;
    Ok(deleted)
}

/// Get all balances (with limit)
#[allow(dead_code)]
pub fn get_all_balances(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<BalanceSnapshot>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, total_quota, used_quota, remaining_quota, currency, raw_response, recorded_at
         FROM checkin_balances
         ORDER BY recorded_at DESC
         LIMIT ?1",
    )?;

    let balances = stmt
        .query_map(params![limit as i64], row_to_balance)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(balances)
}

/// Get latest balance for all accounts (one per account)
pub fn get_latest_balances_for_all(
    conn: &Connection,
) -> Result<Vec<BalanceSnapshot>, rusqlite::Error> {
    // Using window function to get latest per account
    let mut stmt = conn.prepare(
        "SELECT id, account_id, total_quota, used_quota, remaining_quota, currency, raw_response, recorded_at
         FROM checkin_balances b1
         WHERE recorded_at = (
             SELECT MAX(recorded_at) FROM checkin_balances b2 WHERE b2.account_id = b1.account_id
         )",
    )?;

    let balances = stmt
        .query_map([], row_to_balance)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(balances)
}

/// Delete old balances (older than specified days)
pub fn delete_old_balances(conn: &Connection, days: i64) -> Result<usize, rusqlite::Error> {
    let cutoff = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();
    let deleted = conn.execute(
        "DELETE FROM checkin_balances WHERE recorded_at < ?1",
        params![cutoff],
    )?;
    Ok(deleted)
}

// ═══════════════════════════════════════════════════════════
// WAF Cookies Operations
// ═══════════════════════════════════════════════════════════

/// Upsert WAF cookies for a provider
pub fn upsert_waf_cookies(
    conn: &Connection,
    provider_id: &str,
    cookies_json: &str,
    fetched_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> Result<(), rusqlite::Error> {
    let fetched_at_str = fetched_at.to_rfc3339();
    let expires_at_str = expires_at.to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO checkin_waf_cookies (provider_id, cookies_json, fetched_at, expires_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![provider_id, cookies_json, fetched_at_str, expires_at_str],
    )?;

    debug!("Upserted WAF cookies for provider: {}", provider_id);
    Ok(())
}

/// Get WAF cookies for a provider (if not expired)
pub fn get_valid_waf_cookies(
    conn: &Connection,
    provider_id: &str,
) -> Result<Option<String>, rusqlite::Error> {
    let now = Utc::now().to_rfc3339();

    conn.query_row(
        "SELECT cookies_json FROM checkin_waf_cookies
         WHERE provider_id = ?1 AND expires_at > ?2",
        params![provider_id, now],
        |row| row.get(0),
    )
    .optional()
}

/// Delete expired WAF cookies
pub fn delete_expired_waf_cookies(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let now = Utc::now().to_rfc3339();
    let deleted = conn.execute(
        "DELETE FROM checkin_waf_cookies WHERE expires_at <= ?1",
        params![now],
    )?;
    Ok(deleted)
}

/// Delete WAF cookies by provider ID
pub fn delete_waf_cookies_by_provider(
    conn: &Connection,
    provider_id: &str,
) -> Result<bool, rusqlite::Error> {
    let deleted = conn.execute(
        "DELETE FROM checkin_waf_cookies WHERE provider_id = ?1",
        params![provider_id],
    )?;
    Ok(deleted > 0)
}

// ═══════════════════════════════════════════════════════════
// Row Conversion Helpers
// ═══════════════════════════════════════════════════════════

fn row_to_provider(row: &rusqlite::Row) -> Result<CheckinProvider, rusqlite::Error> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let base_url: String = row.get(2)?;
    let checkin_path: String = row.get(3)?;
    let balance_path: String = row.get(4)?;
    let user_info_path: String = row.get(5)?;
    let auth_header: String = row.get(6)?;
    let auth_prefix: String = row.get(7)?;
    let enabled: i32 = row.get(8)?;
    let created_at_str: String = row.get(9)?;
    let updated_at_str: Option<String> = row.get(10)?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let updated_at = updated_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    Ok(CheckinProvider {
        id,
        name,
        base_url,
        checkin_path,
        balance_path,
        user_info_path,
        auth_header,
        auth_prefix,
        enabled: enabled != 0,
        created_at,
        updated_at,
    })
}

fn row_to_account(row: &rusqlite::Row) -> Result<CheckinAccount, rusqlite::Error> {
    let id: String = row.get(0)?;
    let provider_id: String = row.get(1)?;
    let name: String = row.get(2)?;
    let cookies_json_encrypted: String = row.get(3)?;
    let api_user: String = row.get(4)?;
    let enabled: i32 = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let updated_at_str: Option<String> = row.get(7)?;
    let last_checkin_at_str: Option<String> = row.get(8)?;
    let last_balance_check_at_str: Option<String> = row.get(9)?;
    let extra_config: String = row
        .get::<_, Option<String>>(10)?
        .unwrap_or_else(|| "{}".to_string());

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let updated_at = updated_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    let last_checkin_at = last_checkin_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    let last_balance_check_at = last_balance_check_at_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    Ok(CheckinAccount {
        id,
        provider_id,
        name,
        cookies_json_encrypted,
        api_user,
        enabled: enabled != 0,
        created_at,
        updated_at,
        last_checkin_at,
        last_balance_check_at,
        extra_config,
    })
}

fn row_to_record(row: &rusqlite::Row) -> Result<CheckinRecord, rusqlite::Error> {
    let id: String = row.get(0)?;
    let account_id: String = row.get(1)?;
    let status_str: String = row.get(2)?;
    let message: Option<String> = row.get(3)?;
    let reward: Option<String> = row.get(4)?;
    let balance_before: Option<f64> = row.get(5)?;
    let balance_after: Option<f64> = row.get(6)?;
    let checked_in_at_str: String = row.get(7)?;

    let status = match status_str.as_str() {
        "success" => CheckinStatus::Success,
        "already_checked_in" => CheckinStatus::AlreadyCheckedIn,
        _ => CheckinStatus::Failed,
    };

    let checked_in_at = DateTime::parse_from_rfc3339(&checked_in_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(CheckinRecord {
        id,
        account_id,
        status,
        message,
        reward,
        balance_before,
        balance_after,
        checked_in_at,
    })
}

fn row_to_balance(row: &rusqlite::Row) -> Result<BalanceSnapshot, rusqlite::Error> {
    let id: String = row.get(0)?;
    let account_id: String = row.get(1)?;
    let total_quota: f64 = row.get(2)?;
    let used_quota: f64 = row.get(3)?;
    let remaining_quota: f64 = row.get(4)?;
    let currency: String = row.get(5)?;
    let raw_response: Option<String> = row.get(6)?;
    let recorded_at_str: String = row.get(7)?;

    let recorded_at = DateTime::parse_from_rfc3339(&recorded_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(BalanceSnapshot {
        id,
        account_id,
        total_quota,
        used_quota,
        remaining_quota,
        currency,
        raw_response,
        recorded_at,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database::schema::CREATE_TABLES_SQL;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CREATE_TABLES_SQL).unwrap();
        conn
    }

    #[test]
    fn test_provider_crud() {
        let conn = setup_test_db();

        // Insert
        let provider = CheckinProvider::new(
            "Test Provider".to_string(),
            "https://api.test.com".to_string(),
        );
        insert_provider(&conn, &provider).unwrap();

        // Get all
        let providers = get_all_providers(&conn).unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].name, "Test Provider");

        // Get by ID
        let fetched = get_provider_by_id(&conn, &provider.id).unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().base_url, "https://api.test.com");

        // Delete
        let deleted = delete_provider(&conn, &provider.id).unwrap();
        assert!(deleted);

        let providers = get_all_providers(&conn).unwrap();
        assert_eq!(providers.len(), 0);
    }

    #[test]
    fn test_account_crud() {
        let conn = setup_test_db();

        // Insert provider first
        let provider =
            CheckinProvider::new("Provider".to_string(), "https://api.test.com".to_string());
        insert_provider(&conn, &provider).unwrap();

        // Insert account
        let account = CheckinAccount::new(
            provider.id.clone(),
            "Test Account".to_string(),
            "encrypted-cookies".to_string(),
            "12345".to_string(),
        );
        insert_account(&conn, &account).unwrap();

        // Get all
        let accounts = get_all_accounts(&conn).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "Test Account");

        // Get by provider
        let accounts = get_accounts_by_provider(&conn, &provider.id).unwrap();
        assert_eq!(accounts.len(), 1);

        // Get enabled
        let accounts = get_enabled_accounts(&conn).unwrap();
        assert_eq!(accounts.len(), 1);

        // Delete
        let deleted = delete_account(&conn, &account.id).unwrap();
        assert!(deleted);
    }

    #[test]
    fn test_record_operations() {
        let conn = setup_test_db();

        // Insert record
        let record = CheckinRecord::success(
            "account-1".to_string(),
            Some("签到成功".to_string()),
            Some("+10".to_string()),
        );
        insert_record(&conn, &record).unwrap();

        // Get by account
        let records = get_records_by_account(&conn, "account-1", 10).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].status, CheckinStatus::Success);

        // Get all
        let records = get_all_records(&conn, 10).unwrap();
        assert_eq!(records.len(), 1);

        // Delete by account
        let deleted = delete_records_by_account(&conn, "account-1").unwrap();
        assert_eq!(deleted, 1);
    }

    #[test]
    fn test_balance_operations() {
        let conn = setup_test_db();

        // Insert balance
        let balance = BalanceSnapshot::new(
            "account-1".to_string(),
            100.0,
            30.0,
            70.0,
            "USD".to_string(),
        );
        insert_balance(&conn, &balance).unwrap();

        // Get latest
        let latest = get_latest_balance(&conn, "account-1").unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().remaining_quota, 70.0);

        // Get history
        let history = get_balance_history(&conn, "account-1", 10).unwrap();
        assert_eq!(history.len(), 1);

        // Delete by account
        let deleted = delete_balances_by_account(&conn, "account-1").unwrap();
        assert_eq!(deleted, 1);
    }

    #[test]
    fn test_waf_cookies() {
        let conn = setup_test_db();

        let now = Utc::now();
        let expires = now + chrono::Duration::hours(1);

        // Upsert
        upsert_waf_cookies(&conn, "provider-1", r#"{"cf": "abc"}"#, now, expires).unwrap();

        // Get valid
        let cookies = get_valid_waf_cookies(&conn, "provider-1").unwrap();
        assert!(cookies.is_some());
        assert_eq!(cookies.unwrap(), r#"{"cf": "abc"}"#);

        // Upsert again (update)
        upsert_waf_cookies(&conn, "provider-1", r#"{"cf": "xyz"}"#, now, expires).unwrap();
        let cookies = get_valid_waf_cookies(&conn, "provider-1").unwrap();
        assert_eq!(cookies.unwrap(), r#"{"cf": "xyz"}"#);
    }
}
