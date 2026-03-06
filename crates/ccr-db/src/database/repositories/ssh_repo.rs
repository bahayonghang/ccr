//! SSH Host Repository - SQLite data access layer for SSH host configs and known_hosts fingerprints

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::ssh::{SshHost, SshKnownHost};

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
