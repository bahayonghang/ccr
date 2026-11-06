//! SSH Host Repository - SQLite data access layer for SSH host configs and known_hosts fingerprints

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::ssh::{SshHost, SshKnownHost};

/// Names only. Host, user, identity file, and remote home stay in the row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavedHostNames {
    Unavailable,
    Names(Vec<String>),
}

/// Read saved SSH host labels without selecting secret-bearing columns.
///
/// A missing file is left missing. A locked or unreadable database is unavailable.
pub fn read_saved_host_names(path: &std::path::Path) -> SavedHostNames {
    if !path.is_file() {
        return SavedHostNames::Unavailable;
    }
    let Ok(conn) = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    else {
        return SavedHostNames::Unavailable;
    };
    match list_host_names(&conn) {
        Ok(names) => SavedHostNames::Names(names),
        Err(_) => SavedHostNames::Unavailable,
    }
}

pub fn list_host_names(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT name FROM ssh_hosts ORDER BY updated_at DESC, name ASC")?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(names
        .into_iter()
        .filter(|name| !name.trim().is_empty())
        .collect())
}

/// Opens the saved-host store without legacy imports or resolving a user home.
/// A read never creates a database. Explicit host edits initialize owner schemas.
pub fn open_host_store(path: &std::path::Path, write: bool) -> Result<Connection, String> {
    if write && let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "ssh_database_unavailable".to_string())?;
    }
    let flags = if write {
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
    } else {
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
    };
    let conn = Connection::open_with_flags(path, flags)
        .map_err(|_| "ssh_database_unavailable".to_string())?;
    if write {
        super::super::migrations::run_initial_migration(&conn)
            .map_err(|_| "ssh_database_unavailable".to_string())?;
        super::super::migrations::run_migration_v6(&conn)
            .map_err(|_| "ssh_database_unavailable".to_string())?;
    }
    Ok(conn)
}

pub fn list_hosts(conn: &Connection) -> Result<Vec<SshHost>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, host, port, username, identity_file, remote_home, created_at, updated_at, last_connected_at
         FROM ssh_hosts
         ORDER BY updated_at DESC",
    )?;

    stmt.query_map([], row_to_ssh_host)?
        .collect::<Result<Vec<_>, _>>()
}

pub fn get_host(conn: &Connection, id: &str) -> Result<Option<SshHost>, rusqlite::Error> {
    conn.query_row(
        "SELECT id, name, host, port, username, identity_file, remote_home, created_at, updated_at, last_connected_at
         FROM ssh_hosts WHERE id = ?1",
        params![id],
        row_to_ssh_host,
    )
    .optional()
}

pub fn insert_host(conn: &Connection, host: &SshHost) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO ssh_hosts (id, name, host, port, username, identity_file, remote_home, created_at, updated_at, last_connected_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            host.id,
            host.name,
            host.host,
            host.port as i64,
            host.username,
            host.identity_file,
            host.remote_home,
            host.created_at.to_rfc3339(),
            host.updated_at.to_rfc3339(),
            host.last_connected_at.map(|v| v.to_rfc3339()),
        ],
    )?;
    Ok(())
}

pub fn update_host(conn: &Connection, host: &SshHost) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute(
        "UPDATE ssh_hosts
         SET name = ?2, host = ?3, port = ?4, username = ?5, identity_file = ?6, remote_home = ?7, updated_at = ?8, last_connected_at = ?9
         WHERE id = ?1",
        params![
            host.id,
            host.name,
            host.host,
            host.port as i64,
            host.username,
            host.identity_file,
            host.remote_home,
            host.updated_at.to_rfc3339(),
            host.last_connected_at.map(|v| v.to_rfc3339()),
        ],
    )?;
    Ok(affected > 0)
}

pub fn delete_host(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute("DELETE FROM ssh_hosts WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

pub fn set_last_connected_at(
    conn: &Connection,
    id: &str,
    connected_at: DateTime<Utc>,
) -> Result<bool, rusqlite::Error> {
    let affected = conn.execute(
        "UPDATE ssh_hosts SET last_connected_at = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, connected_at.to_rfc3339(), connected_at.to_rfc3339()],
    )?;
    Ok(affected > 0)
}

pub fn get_known_host(
    conn: &Connection,
    host: &str,
    port: u16,
) -> Result<Option<SshKnownHost>, rusqlite::Error> {
    conn.query_row(
        "SELECT host, port, key_type, fingerprint, confirmed_at
         FROM ssh_known_hosts
         WHERE host = ?1 AND port = ?2",
        params![host, port as i64],
        row_to_known_host,
    )
    .optional()
}

pub fn upsert_known_host(
    conn: &Connection,
    known_host: &SshKnownHost,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO ssh_known_hosts (host, port, key_type, fingerprint, confirmed_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(host, port) DO UPDATE SET
           key_type = excluded.key_type,
           fingerprint = excluded.fingerprint,
           confirmed_at = excluded.confirmed_at",
        params![
            known_host.host,
            known_host.port as i64,
            known_host.key_type,
            known_host.fingerprint,
            known_host.confirmed_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn row_to_ssh_host(row: &rusqlite::Row) -> Result<SshHost, rusqlite::Error> {
    let created_at: String = row.get(7)?;
    let updated_at: String = row.get(8)?;
    let last_connected_at: Option<String> = row.get(9)?;

    Ok(SshHost {
        id: row.get(0)?,
        name: row.get(1)?,
        host: row.get(2)?,
        port: row.get::<_, i64>(3)? as u16,
        username: row.get(4)?,
        identity_file: row.get(5)?,
        remote_home: row.get(6)?,
        created_at: parse_dt(&created_at),
        updated_at: parse_dt(&updated_at),
        last_connected_at: last_connected_at.map(|v| parse_dt(&v)),
    })
}

fn row_to_known_host(row: &rusqlite::Row) -> Result<SshKnownHost, rusqlite::Error> {
    let confirmed_at: String = row.get(4)?;
    Ok(SshKnownHost {
        host: row.get(0)?,
        port: row.get::<_, i64>(1)? as u16,
        key_type: row.get(2)?,
        fingerprint: row.get(3)?,
        confirmed_at: parse_dt(&confirmed_at),
    })
}

fn parse_dt(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .map(|v| v.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::{SavedHostNames, insert_host, list_host_names, read_saved_host_names};
    use crate::models::ssh::SshHost;
    use chrono::{TimeZone, Utc};
    use rusqlite::Connection;

    fn at(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0)
            .single()
            .unwrap_or_else(|| panic!("timestamp"))
    }

    #[test]
    fn list_host_names_returns_labels_without_secret_columns() {
        let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
        let path = dir.path().join("hosts.db");
        let conn = Connection::open(&path).unwrap_or_else(|error| panic!("{error}"));
        conn.execute_batch(
            "CREATE TABLE ssh_hosts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT NOT NULL,
                identity_file TEXT,
                remote_home TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_connected_at TEXT
            );",
        )
        .unwrap_or_else(|error| panic!("{error}"));
        let older = SshHost {
            id: "older".into(),
            name: "lab".into(),
            host: "secret-host.example".into(),
            port: 22,
            username: "secret-user".into(),
            identity_file: Some("C:/Keys/secret-id".into()),
            remote_home: Some("/home/secret-user".into()),
            created_at: at(1_700_000_000),
            updated_at: at(1_700_000_000),
            last_connected_at: None,
        };
        let newer = SshHost {
            id: "newer".into(),
            name: "edge".into(),
            host: "secret-host.example".into(),
            port: 22,
            username: "secret-user".into(),
            identity_file: None,
            remote_home: None,
            created_at: at(1_700_000_100),
            updated_at: at(1_700_000_100),
            last_connected_at: None,
        };
        let blank = SshHost {
            id: "blank".into(),
            name: "   ".into(),
            host: "secret-host.example".into(),
            port: 22,
            username: "secret-user".into(),
            identity_file: Some("secret-file".into()),
            remote_home: None,
            created_at: at(1_700_000_200),
            updated_at: at(1_700_000_200),
            last_connected_at: None,
        };
        insert_host(&conn, &older).unwrap_or_else(|error| panic!("{error}"));
        insert_host(&conn, &newer).unwrap_or_else(|error| panic!("{error}"));
        insert_host(&conn, &blank).unwrap_or_else(|error| panic!("{error}"));

        let names = list_host_names(&conn).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(names, vec!["edge".to_string(), "lab".to_string()]);
        let rendered = names.join(" ");
        assert!(!rendered.contains("secret"));
        assert_eq!(
            read_saved_host_names(&path),
            SavedHostNames::Names(vec!["edge".into(), "lab".into()])
        );

        let missing = dir.path().join("missing.db");
        assert_eq!(read_saved_host_names(&missing), SavedHostNames::Unavailable);
        assert!(!missing.exists());
    }
}
