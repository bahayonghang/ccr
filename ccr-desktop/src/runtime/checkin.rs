//! Local check-in accounts already stored in SQLite.
//! Opening this reader does not create a database, import accounts, call a provider,
//! or change the database.

mod operations;
pub use operations::*;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ccr_core::core::log_redact::redact_log_text;
use ccr_db::{database::repositories::checkin_repo, models::checkin::BalanceSnapshot};
use rusqlite::OpenFlags;

use super::{ConfigSource, Services};
use ccr_app::runtime::{AdmissionError, EnvironmentId, RequestKey, TaskSubscription};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckinError {
    Unsupported,
    Storage,
}

impl CheckinError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => {
                "Check-in accounts are read only for the local environment. / 只在本地环境读取签到账户"
            }
            Self::Storage => {
                "The local check-in database could not be read. / 无法读取本地签到数据库"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredNumbers {
    pub remaining: String,
    pub total: String,
    pub used: String,
    pub currency: Option<String>,
    pub recorded_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoredBalance {
    Missing,
    Unusable,
    Numbers(StoredNumbers),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckinAccountRow {
    pub id: String,
    pub name: String,
    pub provider_name: Option<String>,
    pub enabled: bool,
    pub last_checkin_at: Option<String>,
    pub balance: StoredBalance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckinSnapshot {
    NoDatabase,
    Accounts(Vec<CheckinAccountRow>),
}

fn finite_number(value: f64) -> Option<String> {
    value.is_finite().then(|| format!("{value}"))
}

fn stored_balance(balance: &BalanceSnapshot) -> StoredBalance {
    let Some(remaining) = finite_number(balance.remaining_quota) else {
        return StoredBalance::Unusable;
    };
    let Some(total) = finite_number(balance.total_quota) else {
        return StoredBalance::Unusable;
    };
    let Some(used) = finite_number(balance.used_quota) else {
        return StoredBalance::Unusable;
    };
    let currency = {
        let currency = redact_log_text(balance.currency.trim());
        if currency.is_empty() {
            None
        } else {
            Some(currency)
        }
    };
    StoredBalance::Numbers(StoredNumbers {
        remaining,
        total,
        used,
        currency,
        recorded_at: balance.recorded_at.to_rfc3339(),
    })
}

fn project_rows(
    accounts: Vec<ccr_db::models::checkin::CheckinAccount>,
    providers: HashMap<String, String>,
    balances: HashMap<String, BalanceSnapshot>,
) -> Vec<CheckinAccountRow> {
    accounts
        .into_iter()
        .map(|account| {
            let provider_name = providers
                .get(&account.provider_id)
                .map(|name| redact_log_text(name));
            let balance = balances
                .get(&account.id)
                .map(stored_balance)
                .unwrap_or(StoredBalance::Missing);
            CheckinAccountRow {
                id: account.id,
                name: redact_log_text(&account.name),
                provider_name,
                enabled: account.enabled,
                last_checkin_at: account.last_checkin_at.map(|time| time.to_rfc3339()),
                balance,
            }
        })
        .collect()
}

fn read_connection(conn: &rusqlite::Connection) -> Result<CheckinSnapshot, CheckinError> {
    let accounts = checkin_repo::get_all_accounts(conn).map_err(|_| CheckinError::Storage)?;
    let providers = checkin_repo::get_all_providers(conn)
        .map_err(|_| CheckinError::Storage)?
        .into_iter()
        .map(|provider| (provider.id, provider.name))
        .collect::<HashMap<_, _>>();
    let mut balances = HashMap::new();
    for balance in
        checkin_repo::get_latest_balances_for_all(conn).map_err(|_| CheckinError::Storage)?
    {
        balances
            .entry(balance.account_id.clone())
            .or_insert(balance);
    }
    Ok(CheckinSnapshot::Accounts(project_rows(
        accounts, providers, balances,
    )))
}

fn existing_file(path: PathBuf) -> Option<PathBuf> {
    path.is_file().then_some(path)
}

fn database_path(source: &ConfigSource) -> Result<Option<PathBuf>, CheckinError> {
    match source {
        ConfigSource::Benchmark(layout) => Ok(existing_file(layout.path("ui/ccr-ui.db"))),
        ConfigSource::Local => {
            let home = dirs::home_dir().ok_or(CheckinError::Storage)?;
            Ok(existing_file(home.join(".ccr-ui").join("ccr-ui.db")))
        }
        ConfigSource::Fixture { registry, .. } => {
            if !registry.is_absolute() {
                return Err(CheckinError::Storage);
            }
            let root = registry.parent().ok_or(CheckinError::Storage)?;
            Ok(existing_file(root.join("checkin.db")))
        }
    }
}

fn read_existing(path: &Path) -> Result<CheckinSnapshot, CheckinError> {
    let conn = rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| CheckinError::Storage)?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|_| CheckinError::Storage)?;
    read_connection(&conn)
}

fn read_source(source: &ConfigSource) -> Result<CheckinSnapshot, CheckinError> {
    let Some(path) = database_path(source)? else {
        return Ok(CheckinSnapshot::NoDatabase);
    };
    read_existing(&path)
}

impl Services {
    pub fn checkin_request(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<CheckinSnapshot, CheckinError>, AdmissionError> {
        let source = self.source.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.executor.read(key, move |_, _| {
            if !local {
                return Err(CheckinError::Unsupported);
            }
            read_source(&source)
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn balance(remaining: f64, total: f64, used: f64, currency: &str) -> BalanceSnapshot {
        let mut balance =
            BalanceSnapshot::new("account".into(), total, used, remaining, currency.into());
        balance.recorded_at = chrono::DateTime::parse_from_rfc3339("2024-04-04T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        balance
    }

    #[test]
    fn stored_numbers_keep_real_values_and_reject_non_finite() {
        let stored = stored_balance(&balance(7.5, 10.0, 2.5, "USD"));
        assert_eq!(
            stored,
            StoredBalance::Numbers(StoredNumbers {
                remaining: "7.5".into(),
                total: "10".into(),
                used: "2.5".into(),
                currency: Some("USD".into()),
                recorded_at: "2024-04-04T00:00:00+00:00".into(),
            })
        );
        assert_eq!(
            stored_balance(&balance(f64::NAN, 1.0, 0.0, "USD")),
            StoredBalance::Unusable
        );
        assert_eq!(
            stored_balance(&balance(1.0, f64::INFINITY, 0.0, "USD")),
            StoredBalance::Unusable
        );
    }

    #[test]
    fn projection_drops_cookies_provider_url_and_extra_config() {
        let mut provider = ccr_db::models::checkin::CheckinProvider::new(
            "Fixture provider".into(),
            "https://example.invalid/checkin-secret-sentinel".into(),
        );
        provider.id = "provider-1".into();
        let mut account = ccr_db::models::checkin::CheckinAccount::new(
            provider.id.clone(),
            "Desk account".into(),
            "checkin-secret-sentinel".into(),
            "checkin-secret-sentinel".into(),
        );
        account.id = "account-1".into();
        account.extra_config = r#"{"token":"checkin-secret-sentinel"}"#.into();
        let mut snapshot = balance(7.5, 10.0, 2.5, "USD");
        snapshot.account_id = account.id.clone();
        let rows = project_rows(
            vec![account],
            HashMap::from([(provider.id, provider.name)]),
            HashMap::from([(snapshot.account_id.clone(), snapshot)]),
        );
        let rendered = format!("{rows:?}");
        assert!(rendered.contains("Desk account"));
        assert!(rendered.contains("Fixture provider"));
        assert!(rendered.contains("7.5"));
        assert!(!rendered.contains("checkin-secret-sentinel"));
        assert!(!rendered.contains("example.invalid"));
        assert!(!rendered.contains("extra_config"));
        assert!(!rendered.contains("cookies"));
    }

    #[test]
    fn missing_fixture_file_does_not_create_a_database() {
        let root = tempfile::tempdir().unwrap();
        let registry = root.path().join("registry.toml");
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: registry.clone(),
            delay: std::time::Duration::ZERO,
        };
        let snapshot = read_source(&source).unwrap();
        assert_eq!(snapshot, CheckinSnapshot::NoDatabase);
        assert!(!root.path().join("checkin.db").exists());
        assert!(!root.path().join("checkin.db-wal").exists());
        assert!(!root.path().join("checkin.db-shm").exists());
    }

    #[test]
    fn reading_an_existing_database_does_not_rewrite_it() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("checkin.db");
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.pragma_update(None, "journal_mode", "DELETE").unwrap();
            let mode: String = conn
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .unwrap();
            assert_eq!(mode, "delete");
        }
        let before = std::fs::read(&path).unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        };
        let error = read_source(&source).unwrap_err();
        assert_eq!(error, CheckinError::Storage);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!root.path().join("checkin.db-wal").exists());
        assert!(!root.path().join("checkin.db-shm").exists());
    }
}
