//! Narrow, read-only activation projection. Never use HistoryEntry's permissive decoder.

use ccr_config::models::platform::Platform;
pub use ccr_types::{ProfileActivity, ProfileActivityAvailability, ProfileActivitySnapshot};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags, types::ValueRef};
use std::{collections::HashSet, fmt, path::Path};

/// Stable categories deliberately exclude SQL text, database paths and row contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileActivityError {
    PathUnavailable,
    Open,
    Query,
}

impl fmt::Display for ProfileActivityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PathUnavailable => "Cannot resolve the profile activity database location",
            Self::Open => "Cannot open the profile activity database for reading",
            Self::Query => "Cannot read recorded profile activations",
        })
    }
}

impl std::error::Error for ProfileActivityError {}

/// One filtered pass for all requested names. Memory is bounded by the request,
/// not by history length. No schema initialization, pool or credential columns.
pub fn read_profile_activity(
    path: &Path,
    platform: Platform,
    names: &[String],
) -> Result<ProfileActivitySnapshot, ProfileActivityError> {
    match path.try_exists() {
        Ok(false) => return Ok(ProfileActivitySnapshot::default()),
        Err(_) => return Err(ProfileActivityError::Open),
        Ok(true) => {}
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| ProfileActivityError::Open)?;
    read_from_connection(&conn, platform, names)
}

fn read_from_connection(
    conn: &Connection,
    platform: Platform,
    names: &[String],
) -> Result<ProfileActivitySnapshot, ProfileActivityError> {
    let mut schema = conn
        .prepare("SELECT name FROM pragma_table_info('history')")
        .map_err(|_| ProfileActivityError::Query)?;
    let columns = schema
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|_| ProfileActivityError::Query)?
        .collect::<rusqlite::Result<HashSet<_>>>()
        .map_err(|_| ProfileActivityError::Query)?;
    if ["timestamp", "operation", "result", "to_config", "extra"]
        .iter()
        .any(|name| !columns.contains(*name))
    {
        return Ok(ProfileActivitySnapshot {
            availability: ProfileActivityAvailability::MissingSchema,
            ..Default::default()
        });
    }
    let mut snapshot = ProfileActivitySnapshot {
        availability: ProfileActivityAvailability::Ready,
        ..Default::default()
    };
    let names: HashSet<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|name| !name.trim().is_empty())
        .collect();
    if names.is_empty() {
        return Ok(snapshot);
    }
    let mut stmt = conn
        .prepare(
            "SELECT to_config, timestamp FROM history
             WHERE operation = 'Switch' AND result = 'Success' AND extra = ?1",
        )
        .map_err(|_| ProfileActivityError::Query)?;
    let mut rows = stmt
        .query([platform.short_name()])
        .map_err(|_| ProfileActivityError::Query)?;
    while let Some(row) = rows.next().map_err(|_| ProfileActivityError::Query)? {
        // Non-text names cannot identify a requested profile. No coercion or aliases.
        let name = match row.get_ref(0).map_err(|_| ProfileActivityError::Query)? {
            ValueRef::Text(bytes) => match std::str::from_utf8(bytes) {
                Ok(name) if names.contains(name) => name,
                _ => continue,
            },
            _ => continue,
        };
        let activity = snapshot.profiles.entry(name.to_owned()).or_default();
        let timestamp = match row.get_ref(1).map_err(|_| ProfileActivityError::Query)? {
            ValueRef::Text(bytes) => std::str::from_utf8(bytes)
                .ok()
                .and_then(|text| DateTime::parse_from_rfc3339(text).ok())
                .map(|instant| instant.with_timezone(&Utc)),
            _ => None,
        };
        if let Some(timestamp) = timestamp {
            // UTC comparison handles mixed offsets; SQL MAX(text) does not.
            let index = activity
                .recent_activations
                .partition_point(|existing| *existing >= timestamp);
            if index < 3 {
                activity.recent_activations.insert(index, timestamp);
                activity.recent_activations.truncate(3);
            }
        } else {
            activity.invalid_timestamps += 1;
        }
    }
    Ok(snapshot)
}
