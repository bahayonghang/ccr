// Claude Profile Repository — SQLite 数据访问层
// 提供 claude_profiles 表的完整 CRUD 操作

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use tracing::debug;

use crate::models::claude_profile::ClaudeProfile;

// ═══════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════

/// SELECT 列列表 — 所有查询共用，保持与 row_to_profile 的列顺序一致
const PROFILE_COLUMNS: &str =
    "id, name, description, snapshot_json, tags, is_current, enabled, created_at, updated_at";

// ═══════════════════════════════════════════════════════════
// Profile Operations
// ═══════════════════════════════════════════════════════════

/// 插入新 Profile
#[allow(dead_code)]
pub fn insert_profile(conn: &Connection, profile: &ClaudeProfile) -> Result<(), rusqlite::Error> {
    let created_at = profile.created_at.to_rfc3339();
    let updated_at = profile.updated_at.to_rfc3339();

    conn.execute(
        "INSERT INTO claude_profiles (id, name, description, snapshot_json, tags,
         is_current, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            profile.id,
            profile.name,
            profile.description,
            profile.snapshot_json,
            profile.tags,
            if profile.is_current { 1 } else { 0 },
            if profile.enabled { 1 } else { 0 },
            created_at,
            updated_at,
        ],
    )?;

    debug!("Inserted claude profile: {} ({})", profile.name, profile.id);
    Ok(())
}

/// 按名称查询
#[allow(dead_code)]
pub fn get_profile_by_name(
    conn: &Connection,
    name: &str,
) -> Result<Option<ClaudeProfile>, rusqlite::Error> {
    conn.query_row(
        &format!("SELECT {PROFILE_COLUMNS} FROM claude_profiles WHERE name = ?1"),
        params![name],
        row_to_profile,
    )
    .optional()
}

/// 列出全部 Profile
#[allow(dead_code)]
pub fn get_all_profiles(conn: &Connection) -> Result<Vec<ClaudeProfile>, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PROFILE_COLUMNS} FROM claude_profiles ORDER BY created_at DESC"
    ))?;

    let profiles = stmt
        .query_map([], row_to_profile)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(profiles)
}

/// 获取当前激活的 Profile
#[allow(dead_code)]
pub fn get_current_profile(conn: &Connection) -> Result<Option<ClaudeProfile>, rusqlite::Error> {
    conn.query_row(
        &format!("SELECT {PROFILE_COLUMNS} FROM claude_profiles WHERE is_current = 1"),
        [],
        row_to_profile,
    )
    .optional()
}

/// 更新 Profile（按原始名称查找，更新为 profile 中的新值）
#[allow(dead_code)]
pub fn update_profile(
    conn: &Connection,
    name: &str,
    profile: &ClaudeProfile,
) -> Result<bool, rusqlite::Error> {
    let updated_at = Utc::now().to_rfc3339();

    let affected = conn.execute(
        "UPDATE claude_profiles SET
         name = ?1, description = ?2, snapshot_json = ?3, tags = ?4,
         enabled = ?5, updated_at = ?6
         WHERE name = ?7",
        params![
            profile.name,
            profile.description,
            profile.snapshot_json,
            profile.tags,
            if profile.enabled { 1 } else { 0 },
            updated_at,
            name,
        ],
    )?;

    Ok(affected > 0)
}

/// 删除 Profile
#[allow(dead_code)]
pub fn delete_profile(conn: &Connection, name: &str) -> Result<bool, rusqlite::Error> {
    let deleted = conn.execute("DELETE FROM claude_profiles WHERE name = ?1", params![name])?;
    Ok(deleted > 0)
}

/// 设为当前 Profile（事务内：先清除所有 is_current，再设置目标）
#[allow(dead_code)]
pub fn set_current_profile(conn: &Connection, name: &str) -> Result<bool, rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE claude_profiles SET is_current = 0", [])?;
    let affected = tx.execute(
        "UPDATE claude_profiles SET is_current = 1, updated_at = ?1 WHERE name = ?2",
        params![Utc::now().to_rfc3339(), name],
    )?;
    tx.commit()?;
    Ok(affected > 0)
}

/// 清除当前标记
#[allow(dead_code)]
pub fn clear_current_profile(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute("UPDATE claude_profiles SET is_current = 0", [])?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// Row Conversion Helper
// ═══════════════════════════════════════════════════════════

fn row_to_profile(row: &rusqlite::Row) -> Result<ClaudeProfile, rusqlite::Error> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let description: Option<String> = row.get(2)?;
    let snapshot_json: String = row.get(3)?;
    let tags: Option<String> = row.get(4)?;
    let is_current: i32 = row.get(5)?;
    let enabled: i32 = row.get(6)?;
    let created_at_str: String = row.get(7)?;
    let updated_at_str: String = row.get(8)?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(ClaudeProfile {
        id,
        name,
        description,
        snapshot_json,
        tags,
        is_current: is_current != 0,
        enabled: enabled != 0,
        created_at,
        updated_at,
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
    fn test_profile_crud() {
        let conn = setup_test_db();

        // Insert
        let profile = ClaudeProfile::new(
            "test-profile".to_string(),
            r#"{"settings":{},"mcp_servers":{}}"#.to_string(),
        );
        insert_profile(&conn, &profile).unwrap();

        // Get all
        let profiles = get_all_profiles(&conn).unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "test-profile");

        // Get by name
        let fetched = get_profile_by_name(&conn, "test-profile").unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, profile.id);

        // Delete
        let deleted = delete_profile(&conn, "test-profile").unwrap();
        assert!(deleted);

        let profiles = get_all_profiles(&conn).unwrap();
        assert_eq!(profiles.len(), 0);
    }

    #[test]
    fn test_current_profile() {
        let conn = setup_test_db();

        let p1 = ClaudeProfile::new("profile-1".to_string(), "{}".to_string());
        let p2 = ClaudeProfile::new("profile-2".to_string(), "{}".to_string());
        insert_profile(&conn, &p1).unwrap();
        insert_profile(&conn, &p2).unwrap();

        // Set current
        set_current_profile(&conn, "profile-1").unwrap();
        let current = get_current_profile(&conn).unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().name, "profile-1");

        // Switch current
        set_current_profile(&conn, "profile-2").unwrap();
        let current = get_current_profile(&conn).unwrap();
        assert_eq!(current.unwrap().name, "profile-2");

        // Clear current
        clear_current_profile(&conn).unwrap();
        let current = get_current_profile(&conn).unwrap();
        assert!(current.is_none());
    }

    #[test]
    fn test_update_profile() {
        let conn = setup_test_db();

        let mut profile = ClaudeProfile::new("original".to_string(), "{}".to_string());
        profile.description = Some("original desc".to_string());
        insert_profile(&conn, &profile).unwrap();

        // Update
        let mut updated = profile.clone();
        updated.description = Some("updated desc".to_string());
        updated.snapshot_json = r#"{"settings":{"model":"opus"}}"#.to_string();
        let ok = update_profile(&conn, "original", &updated).unwrap();
        assert!(ok);

        let fetched = get_profile_by_name(&conn, "original").unwrap().unwrap();
        assert_eq!(fetched.description, Some("updated desc".to_string()));
        assert!(fetched.snapshot_json.contains("opus"));
    }
}
