use std::{collections::HashMap, sync::Mutex};

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc};
use chrono_tz::Tz;
use rusqlite::{Connection, functions::FunctionFlags};

use crate::UsageError;

const FN_LOCAL_DATE: &str = "ccr_usage_local_date";
const LOCAL_DATE_CACHE_LIMIT: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedZone {
    Fixed(FixedOffset),
    Iana(Tz),
}

impl ResolvedZone {
    pub(crate) fn local() -> Result<Self, UsageError> {
        let name = iana_time_zone::get_timezone().map_err(|error| {
            UsageError::Query(format!("failed to resolve local IANA timezone: {error}"))
        })?;
        Self::from_iana_name(&name)
    }

    fn from_iana_name(name: &str) -> Result<Self, UsageError> {
        name.parse::<Tz>()
            .map(Self::Iana)
            .map_err(|_| UsageError::Query(format!("unknown local IANA timezone `{name}`")))
    }

    pub(crate) fn utc() -> Self {
        Self::Fixed(Utc.fix())
    }

    pub(crate) fn date_at(&self, instant: DateTime<Utc>) -> NaiveDate {
        match self {
            Self::Fixed(offset) => instant.with_timezone(offset).date_naive(),
            Self::Iana(tz) => instant.with_timezone(tz).date_naive(),
        }
    }

    pub(crate) fn local_date_start_utc(
        &self,
        date: NaiveDate,
    ) -> Result<DateTime<Utc>, UsageError> {
        let Some(midnight) = date.and_hms_opt(0, 0, 0) else {
            return Err(UsageError::Query(format!(
                "invalid local date boundary: {date}"
            )));
        };
        Ok(match self {
            Self::Fixed(offset) => resolve_local(offset, midnight),
            Self::Iana(tz) => resolve_local(tz, midnight),
        })
    }

    pub(crate) fn local_date_expr(&self, column: &str) -> String {
        match self {
            Self::Fixed(offset) => {
                let seconds = offset.local_minus_utc();
                let modifier = if seconds >= 0 {
                    format!("+{seconds} seconds")
                } else {
                    format!("{seconds} seconds")
                };
                format!("date({column}, '{modifier}')")
            }
            Self::Iana(tz) => format!("{FN_LOCAL_DATE}({column}, '{}')", tz.name()),
        }
    }
}

fn resolve_local<T: TimeZone>(timezone: &T, local: NaiveDateTime) -> DateTime<Utc> {
    use chrono::offset::LocalResult;

    match timezone.from_local_datetime(&local) {
        LocalResult::Single(value) => value.with_timezone(&Utc),
        LocalResult::Ambiguous(earliest, _) => earliest.with_timezone(&Utc),
        LocalResult::None => {
            let mut probe = local;
            for _ in 0..16 {
                probe += Duration::minutes(15);
                match timezone.from_local_datetime(&probe) {
                    LocalResult::Single(value) => return value.with_timezone(&Utc),
                    LocalResult::Ambiguous(earliest, _) => return earliest.with_timezone(&Utc),
                    LocalResult::None => {}
                }
            }
            Utc.from_utc_datetime(&local)
        }
    }
}

pub(crate) fn register_functions(conn: &Connection) -> rusqlite::Result<()> {
    let local_date_cache = Mutex::new(HashMap::<(Tz, String), Option<String>>::new());
    conn.create_scalar_function(
        FN_LOCAL_DATE,
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            let timezone = ctx.get_or_create_aux(1, |raw| -> Result<Tz, String> {
                raw.as_str()
                    .map_err(|error| error.to_string())?
                    .parse::<Tz>()
                    .map_err(|_| "unknown IANA timezone".to_string())
            })?;
            let Ok(raw) = ctx.get_raw(0).as_str() else {
                return Ok(None);
            };
            let key = (*timezone.as_ref(), raw.to_string());
            if let Some(cached) = local_date_cache
                .lock()
                .ok()
                .and_then(|cache| cache.get(&key).cloned())
            {
                return Ok(cached);
            }
            let local_date = parse_stored_timestamp(raw).map(|instant| {
                instant
                    .with_timezone(timezone.as_ref())
                    .date_naive()
                    .format("%Y-%m-%d")
                    .to_string()
            });
            if let Ok(mut cache) = local_date_cache.lock() {
                if cache.len() >= LOCAL_DATE_CACHE_LIMIT {
                    cache.clear();
                }
                cache.insert(key, local_date.clone());
            }
            Ok(local_date)
        },
    )
}

fn parse_stored_timestamp(raw: &str) -> Option<DateTime<Utc>> {
    if let Ok(value) = DateTime::parse_from_rfc3339(raw) {
        return Some(value.with_timezone(&Utc));
    }
    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(value) = NaiveDateTime::parse_from_str(raw, format) {
            return Some(Utc.from_utc_datetime(&value));
        }
    }
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|value| Utc.from_utc_datetime(&value))
}

#[cfg(test)]
mod tests {
    use chrono::SecondsFormat;

    use super::*;

    const NEW_YORK: Tz = chrono_tz::America::New_York;

    #[test]
    fn local_bounds_use_historical_dst_offsets() {
        let zone = ResolvedZone::Iana(NEW_YORK);
        let winter = zone
            .local_date_start_utc(
                NaiveDate::from_ymd_opt(2026, 1, 15).expect("winter date should be valid"),
            )
            .expect("winter boundary should resolve");
        let summer = zone
            .local_date_start_utc(
                NaiveDate::from_ymd_opt(2026, 7, 15).expect("summer date should be valid"),
            )
            .expect("summer boundary should resolve");
        assert_eq!(
            winter.to_rfc3339_opts(SecondsFormat::Secs, true),
            "2026-01-15T05:00:00Z"
        );
        assert_eq!(
            summer.to_rfc3339_opts(SecondsFormat::Secs, true),
            "2026-07-15T04:00:00Z"
        );
    }

    #[test]
    fn spring_forward_and_fall_back_instants_keep_the_local_date() {
        let zone = ResolvedZone::Iana(NEW_YORK);
        for raw in [
            "2026-03-08T06:30:00Z",
            "2026-03-08T07:30:00Z",
            "2026-11-01T05:30:00Z",
            "2026-11-01T06:30:00Z",
        ] {
            let instant = DateTime::parse_from_rfc3339(raw)
                .expect("DST fixture timestamp should parse")
                .with_timezone(&Utc);
            let expected = if raw.contains("03-08") {
                "2026-03-08"
            } else {
                "2026-11-01"
            };
            assert_eq!(zone.date_at(instant).to_string(), expected);
        }
    }

    #[test]
    fn sqlite_function_groups_using_real_iana_offsets() {
        let conn = Connection::open_in_memory().expect("in-memory DB should open");
        register_functions(&conn).expect("timezone function should register");
        let winter: String = conn
            .query_row(
                "SELECT ccr_usage_local_date(?1, 'America/New_York')",
                ["2026-01-15T04:30:00Z"],
                |row| row.get(0),
            )
            .expect("winter local date query should succeed");
        let summer: String = conn
            .query_row(
                "SELECT ccr_usage_local_date(?1, 'America/New_York')",
                ["2026-07-15T04:30:00Z"],
                |row| row.get(0),
            )
            .expect("summer local date query should succeed");
        assert_eq!(winter, "2026-01-14");
        assert_eq!(summer, "2026-07-15");

        let tokyo: String = conn
            .query_row(
                "SELECT ccr_usage_local_date(?1, 'Asia/Tokyo')",
                ["2026-01-15T04:30:00Z"],
                |row| row.get(0),
            )
            .expect("Tokyo local date query should succeed");
        assert_eq!(tokyo, "2026-01-15");
    }

    #[test]
    fn unknown_local_iana_zone_is_an_explicit_error() {
        let error = ResolvedZone::from_iana_name("Not/AZone")
            .expect_err("unknown local zone must not fall back to a fixed offset");
        assert!(matches!(error, UsageError::Query(_)));
        assert!(error.to_string().contains("Not/AZone"));
    }
}
