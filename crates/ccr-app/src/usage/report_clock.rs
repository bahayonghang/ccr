//! A report timestamp and calendar zone, never an elapsed/process clock.
use ccr_usage::ReportTimezone;
use chrono::{DateTime, Local, NaiveDate, Utc};
#[derive(Clone, Copy, Debug)]
pub struct ReportClock {
    pub as_of: DateTime<Utc>,
    pub timezone: ReportTimezone,
}
impl ReportClock {
    pub fn live() -> Self {
        Self {
            as_of: Utc::now(),
            timezone: ReportTimezone::Local,
        }
    }
    pub fn fixed_utc(as_of: DateTime<Utc>) -> Self {
        Self {
            as_of,
            timezone: ReportTimezone::Utc,
        }
    }
    pub fn date(self) -> NaiveDate {
        match self.timezone {
            ReportTimezone::Utc => self.as_of.date_naive(),
            ReportTimezone::Local => self.as_of.with_timezone(&Local).date_naive(),
        }
    }
    pub fn window(self, days: i64) -> (NaiveDate, NaiveDate) {
        let end = self.date();
        let start = end
            .checked_sub_signed(
                chrono::Duration::try_days(days.saturating_sub(1).max(0)).unwrap_or_default(),
            )
            .unwrap_or(end);
        (start, end)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn fixed_utc_window_uses_manifest_date_across_year_boundary() {
        let clock = ReportClock::fixed_utc(
            DateTime::parse_from_rfc3339("2000-01-01T00:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        );
        assert_eq!(clock.date().to_string(), "2000-01-01");
        let (start, end) = clock.window(7);
        assert_eq!(start.to_string(), "1999-12-26");
        assert_eq!(end.to_string(), "2000-01-01");
        let live = ReportClock::live();
        assert_eq!(live.timezone, ReportTimezone::Local);
        assert!((Utc::now() - live.as_of).num_seconds().abs() < 2);
    }
}
