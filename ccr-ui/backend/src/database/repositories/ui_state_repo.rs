// UI State Repository - SQLite data access layer for UI favorites and history
// Replaces JSON file storage at ~/.ccr/ui_state.json

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use tracing::debug;

use crate::models::ui_state::{CommandHistory, FavoriteCommand};

/// Insert a favorite command into the database
pub fn insert_favorite(
    conn: &Connection,
    favorite: &FavoriteCommand,
) -> Result<(), rusqlite::Error> {
    let args_json = serde_json::to_string(&favorite.args).unwrap_or_else(|_| "[]".to_string());
    let created_at = favorite.created_at.to_rfc3339();

    conn.execute(
        "INSERT INTO ui_favorites (id, command, args_json, display_name, module, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            favorite.id,
            favorite.command,
            args_json,
            favorite.display_name,
            favorite.module,
            created_at,
        ],
    )?;

    debug!("Inserted favorite: {} ({})", favorite.command, favorite.id);
    Ok(())
}

/// Find a favorite by command and args (for duplicate detection)
pub fn find_favorite_by_command_args(
    conn: &Connection,
    command: &str,
    args: &[String],
) -> Result<Option<FavoriteCommand>, rusqlite::Error> {
    let args_json = serde_json::to_string(args).unwrap_or_else(|_| "[]".to_string());

    conn.query_row(
        "SELECT id, command, args_json, display_name, module, created_at
         FROM ui_favorites
         WHERE command = ?1 AND args_json = ?2",
        params![command, args_json],
        row_to_favorite,
    )
    .optional()
}

/// Get all favorites ordered by creation date (newest first)
pub fn get_all_favorites(conn: &Connection) -> Result<Vec<FavoriteCommand>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, command, args_json, display_name, module, created_at
         FROM ui_favorites
         ORDER BY created_at DESC",
    )?;

    let favorites = stmt
        .query_map([], row_to_favorite)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(favorites)
}

/// Delete a favorite by ID
pub fn delete_favorite(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
    let deleted = conn.execute("DELETE FROM ui_favorites WHERE id = ?1", params![id])?;
    Ok(deleted > 0)
}

/// Insert a command history entry
pub fn insert_history(conn: &Connection, history: &CommandHistory) -> Result<(), rusqlite::Error> {
    let args_json = serde_json::to_string(&history.args).unwrap_or_else(|_| "[]".to_string());
    let executed_at = history.executed_at.to_rfc3339();

    conn.execute(
        "INSERT INTO ui_history (id, full_command, command, args_json, success, executed_at, duration_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            history.id,
            history.full_command,
            history.command,
            args_json,
            if history.success { 1 } else { 0 },
            executed_at,
            history.duration_ms as i64,
        ],
    )?;

    debug!(
        "Inserted history: {} ({})",
        history.full_command, history.id
    );
    Ok(())
}

/// Get command history with optional limit
pub fn get_history(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<CommandHistory>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, full_command, command, args_json, success, executed_at, duration_ms
         FROM ui_history
         ORDER BY executed_at DESC
         LIMIT ?1",
    )?;

    let history = stmt
        .query_map(params![limit as i64], row_to_history)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(history)
}

/// Clear all command history
pub fn clear_history(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let deleted = conn.execute("DELETE FROM ui_history", [])?;
    debug!("Cleared {} history entries", deleted);
    Ok(deleted)
}

/// Get total count of favorites
#[allow(dead_code)]
pub fn count_favorites(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM ui_favorites", [], |row| row.get(0))
}

/// Get total count of history entries
#[allow(dead_code)]
pub fn count_history(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM ui_history", [], |row| row.get(0))
}

// Helper function to convert a row to FavoriteCommand
fn row_to_favorite(row: &rusqlite::Row) -> Result<FavoriteCommand, rusqlite::Error> {
    let id: String = row.get(0)?;
    let command: String = row.get(1)?;
    let args_json: String = row.get(2)?;
    let display_name: Option<String> = row.get(3)?;
    let module: String = row.get(4)?;
    let created_at_str: String = row.get(5)?;

    let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(FavoriteCommand {
        id,
        command,
        args,
        display_name,
        module,
        created_at,
    })
}

// Helper function to convert a row to CommandHistory
fn row_to_history(row: &rusqlite::Row) -> Result<CommandHistory, rusqlite::Error> {
    let id: String = row.get(0)?;
    let full_command: String = row.get(1)?;
    let command: String = row.get(2)?;
    let args_json: String = row.get(3)?;
    let success: i32 = row.get(4)?;
    let executed_at_str: String = row.get(5)?;
    let duration_ms: i64 = row.get(6)?;

    let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
    let executed_at = DateTime::parse_from_rfc3339(&executed_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(CommandHistory {
        id,
        full_command,
        command,
        args,
        success: success != 0,
        executed_at,
        duration_ms: duration_ms as u64,
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
    fn test_insert_and_get_favorites() {
        let conn = setup_test_db();

        let favorite = FavoriteCommand {
            id: "test-id-1".to_string(),
            command: "test".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            display_name: Some("Test Command".to_string()),
            module: "test-module".to_string(),
            created_at: Utc::now(),
        };

        insert_favorite(&conn, &favorite).unwrap();

        let favorites = get_all_favorites(&conn).unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, "test-id-1");
        assert_eq!(favorites[0].command, "test");
        assert_eq!(favorites[0].args, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_find_duplicate_favorite() {
        let conn = setup_test_db();

        let favorite = FavoriteCommand {
            id: "test-id-1".to_string(),
            command: "test".to_string(),
            args: vec!["arg1".to_string()],
            display_name: None,
            module: "test-module".to_string(),
            created_at: Utc::now(),
        };

        insert_favorite(&conn, &favorite).unwrap();

        // Find by command and args
        let found = find_favorite_by_command_args(&conn, "test", &["arg1".to_string()]).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "test-id-1");

        // Not found with different args
        let not_found =
            find_favorite_by_command_args(&conn, "test", &["arg2".to_string()]).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_delete_favorite() {
        let conn = setup_test_db();

        let favorite = FavoriteCommand {
            id: "test-id-1".to_string(),
            command: "test".to_string(),
            args: vec![],
            display_name: None,
            module: "test-module".to_string(),
            created_at: Utc::now(),
        };

        insert_favorite(&conn, &favorite).unwrap();
        assert_eq!(count_favorites(&conn).unwrap(), 1);

        let deleted = delete_favorite(&conn, "test-id-1").unwrap();
        assert!(deleted);
        assert_eq!(count_favorites(&conn).unwrap(), 0);

        // Delete non-existent returns false
        let not_deleted = delete_favorite(&conn, "non-existent").unwrap();
        assert!(!not_deleted);
    }

    #[test]
    fn test_insert_and_get_history() {
        let conn = setup_test_db();

        let history = CommandHistory {
            id: "hist-1".to_string(),
            full_command: "ccr test arg1".to_string(),
            command: "test".to_string(),
            args: vec!["arg1".to_string()],
            success: true,
            executed_at: Utc::now(),
            duration_ms: 150,
        };

        insert_history(&conn, &history).unwrap();

        let entries = get_history(&conn, 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "hist-1");
        assert!(entries[0].success);
        assert_eq!(entries[0].duration_ms, 150);
    }

    #[test]
    fn test_clear_history() {
        let conn = setup_test_db();

        for i in 0..5 {
            let history = CommandHistory {
                id: format!("hist-{}", i),
                full_command: format!("ccr cmd{}", i),
                command: format!("cmd{}", i),
                args: vec![],
                success: true,
                executed_at: Utc::now(),
                duration_ms: 100,
            };
            insert_history(&conn, &history).unwrap();
        }

        assert_eq!(count_history(&conn).unwrap(), 5);

        let cleared = clear_history(&conn).unwrap();
        assert_eq!(cleared, 5);
        assert_eq!(count_history(&conn).unwrap(), 0);
    }

    #[test]
    fn test_history_limit() {
        let conn = setup_test_db();

        for i in 0..10 {
            let history = CommandHistory {
                id: format!("hist-{}", i),
                full_command: format!("ccr cmd{}", i),
                command: format!("cmd{}", i),
                args: vec![],
                success: true,
                executed_at: Utc::now(),
                duration_ms: 100,
            };
            insert_history(&conn, &history).unwrap();
        }

        let entries = get_history(&conn, 5).unwrap();
        assert_eq!(entries.len(), 5);
    }
}
