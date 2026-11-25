//! Calendar and trend share one series of dated observations. Domain balance
//! days are UTC, while a check-in belongs to the user's local calendar day.
use super::*;
use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate, Utc};
use rusqlite::{Connection, OptionalExtension};

#[derive(Clone, Debug)]
pub struct DayNumbers {
    pub source: NaiveDate,
    pub unit: String,
    pub remaining: f64,
    pub total: f64,
    pub used: f64,
}

#[derive(Clone, Debug)]
pub enum DayBalance {
    Missing,
    Unusable {
        source: NaiveDate,
        unit: Option<String>,
    },
    Measured(DayNumbers),
    CarriedForward(DayNumbers),
}

impl DayBalance {
    pub fn numbers(&self) -> Option<&DayNumbers> {
        match self {
            Self::Measured(n) | Self::CarriedForward(n) => Some(n),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct HistoryDay {
    pub date: NaiveDate,
    pub checked: bool,
    pub reward: Option<String>,
    pub balance: DayBalance,
}

pub type BalanceSample = (DateTime<Utc>, DayBalance);

#[derive(Clone, Debug)]
pub struct AccountHistory {
    /// The contiguous union of the selected month and its trailing trend window.
    pub days: Vec<HistoryDay>,
    pub month: NaiveDate,
    pub trend_from: NaiveDate,
    pub trend_to: NaiveDate,
    pub current_streak: usize,
    pub samples: Vec<BalanceSample>,
}

fn timestamp(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<DateTime<Utc>> {
    let stamp: String = row.get(index)?;
    DateTime::parse_from_rfc3339(&stamp)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                index,
                rusqlite::types::Type::Text,
                Box::new(err),
            )
        })
}

fn sample(row: &rusqlite::Row<'_>) -> rusqlite::Result<BalanceSample> {
    let at = timestamp(row, 0)?;
    let source = at.date_naive();
    let unit: String = row.get(1)?;
    let unit = unit.trim();
    let remaining: f64 = row.get(2)?;
    let total: f64 = row.get(3)?;
    let used: f64 = row.get(4)?;
    let balance = if !unit.is_empty() && [remaining, total, used].into_iter().all(f64::is_finite) {
        DayBalance::Measured(DayNumbers {
            source,
            unit: redact_log_text(unit),
            remaining,
            total,
            used,
        })
    } else {
        DayBalance::Unusable {
            source,
            unit: (!unit.is_empty()).then(|| redact_log_text(unit)),
        }
    };
    Ok((at, balance))
}

fn read_samples(
    conn: &Connection,
    account: &str,
    start: NaiveDate,
    until: NaiveDate,
) -> rusqlite::Result<(Option<DayNumbers>, Vec<BalanceSample>)> {
    // The repository persists checked/recorded timestamps from DateTime<Utc>
    // as RFC3339. Bare ISO dates bound their entire UTC day, including fractions.
    let fields = "SELECT recorded_at, currency, remaining_quota, total_quota, used_quota
        FROM checkin_balances WHERE account_id = ?1";
    let before =
        format!("{fields} AND recorded_at < ?2 ORDER BY recorded_at DESC, id DESC LIMIT 1");
    let start = start.to_string();
    let previous = conn
        .query_row(&before, (account, &start), sample)
        .optional()?
        .and_then(|(_, balance)| balance.numbers().cloned());
    let within =
        format!("{fields} AND recorded_at >= ?2 AND recorded_at < ?3 ORDER BY recorded_at, id");
    let samples = conn
        .prepare(&within)?
        .query_map((account, start, until.to_string()), sample)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((previous, samples))
}

impl Services {
    pub fn checkin_account_history(
        &self,
        account: &str,
        month: NaiveDate,
        trend_days: u32,
    ) -> Result<Option<AccountHistory>, CheckinError> {
        let start = month.with_day(1).ok_or(CheckinError::Storage)?;
        let until = start
            .checked_add_months(Months::new(1))
            .ok_or(CheckinError::Storage)?;
        let today = Local::now().date_naive();
        let trend_to = if start <= today && today < until {
            today
        } else {
            until.pred_opt().ok_or(CheckinError::Storage)?
        };
        let trend_from = match trend_days {
            0 => start,
            1..=366 => trend_to
                .checked_sub_days(Days::new(u64::from(trend_days - 1)))
                .ok_or(CheckinError::Storage)?,
            _ => return Err(CheckinError::Storage),
        };
        self.read_checkin_history(|conn| {
            conn.query_row(
                "SELECT 1 FROM checkin_accounts WHERE id=?1",
                [account],
                |_| Ok(()),
            )?;
            let first = start.min(trend_from);
            let (mut last_known, samples) = read_samples(conn, account, first, until)?;
            let mut latest = BTreeMap::new();
            for (at, balance) in &samples {
                latest.insert(at.date_naive(), balance);
            }
            // Complete successful record days are also needed for the current streak.
            let mut checked = BTreeMap::new();
            let mut stmt = conn.prepare(
                "SELECT checked_in_at,reward FROM checkin_records WHERE account_id = ?1
                AND status IN ('success','already_checked_in') ORDER BY checked_in_at,id",
            )?;
            let mut records = stmt.query([account])?;
            while let Some(row) = records.next()? {
                let date = timestamp(row, 0)?.with_timezone(&Local).date_naive();
                let reward: Option<String> = row.get(1)?;
                let saved = checked.entry(date).or_insert(None);
                if let Some(reward) = reward {
                    *saved = Some(
                        ccr_checkin::services::checkin_service::redact_provider_feedback(&reward),
                    );
                }
            }
            let streak_end = if checked.contains_key(&today) {
                Some(today)
            } else {
                today.pred_opt()
            };
            let current_streak = streak_end.map_or(0, |day| {
                let mut date = Some(day);
                let mut count = 0;
                while let Some(day) = date.filter(|d| checked.contains_key(d)) {
                    count += 1;
                    date = day.pred_opt();
                }
                count
            });
            let mut days = Vec::new();
            for date in first.iter_days().take_while(|date| *date < until) {
                let balance = if let Some(observation) = latest.get(&date) {
                    last_known = observation.numbers().cloned();
                    (*observation).clone()
                } else if date <= today {
                    last_known
                        .clone()
                        .map(DayBalance::CarriedForward)
                        .unwrap_or(DayBalance::Missing)
                } else {
                    DayBalance::Missing
                };
                days.push(HistoryDay {
                    date,
                    checked: checked.contains_key(&date),
                    reward: checked.get(&date).cloned().flatten(),
                    balance,
                });
            }
            Ok(AccountHistory {
                days,
                month: start,
                trend_from,
                trend_to,
                current_streak,
                samples,
            })
        })
    }
}
