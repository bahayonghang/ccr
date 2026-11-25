//! Local check-in accounts already stored in SQLite.
//! Opening this reader does not create a database, import accounts, call a provider,
//! or change the database.
use ccr_app::environments::EnvironmentPolicy;
use ccr_app::runtime::EnvironmentId;

#[cfg(feature = "native-checkin-fixture")]
pub(super) mod fixture;
mod operations;
pub use operations::*;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ccr_core::core::log_redact::redact_log_text;
use ccr_db::{database::repositories::checkin_repo, models::checkin::BalanceSnapshot};
use rusqlite::OpenFlags;

use super::{ConfigSource, Services};
use ccr_app::runtime::{AdmissionError, RequestKey, TaskSubscription};

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
    pub provider_id: String,
    pub provider_name: Option<String>,
    pub enabled: bool,
    pub last_checkin_at: Option<String>,
    pub last_balance_check_at: Option<String>,
    pub balance: StoredBalance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckinProviderRow {
    pub id: String,
    pub name: String,
    pub builtin_id: Option<String>,
    pub updated_at: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BalanceGroup {
    pub accounts: usize,
    pub remaining: f64,
    pub total: f64,
    pub used: f64,
    pub from: chrono::DateTime<chrono::Utc>,
    pub to: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SavedBalances {
    pub groups: BTreeMap<String, BalanceGroup>,
    pub missing: usize,
    pub unusable: usize,
    pub unknown_unit: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CheckinData {
    pub accounts: Vec<CheckinAccountRow>,
    pub providers: Vec<CheckinProviderRow>,
    pub balances: SavedBalances,
    pub valid_waf_providers: std::collections::BTreeSet<String>,
    pub records: Vec<RecordSummary>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CheckinSnapshot {
    NoDatabase,
    Accounts(CheckinData),
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

fn project_data(
    accounts: Vec<ccr_db::models::checkin::CheckinAccount>,
    providers: Vec<ccr_db::models::checkin::CheckinProvider>,
    balances: HashMap<String, BalanceSnapshot>,
) -> CheckinData {
    let providers: Vec<_> = providers
        .into_iter()
        .map(|provider| CheckinProviderRow {
            updated_at: provider.updated_at.map(|t| t.to_rfc3339()),
            builtin_id: provider.builtin_id.clone().or_else(|| {
                ccr_checkin::managers::checkin::builtin_providers::resolve_builtin_for_provider(
                    &provider,
                )
                .map(|builtin| builtin.id)
            }),
            id: provider.id,
            name: redact_log_text(&provider.name),
            enabled: provider.enabled,
        })
        .collect();
    let names: HashMap<_, _> = providers.iter().map(|p| (&p.id, &p.name)).collect();
    let mut summary = SavedBalances::default();
    let accounts = accounts
        .into_iter()
        .map(|account| {
            let latest = balances.get(&account.id);
            let balance = latest.map(stored_balance).unwrap_or(StoredBalance::Missing);
            match (&balance, latest) {
                (StoredBalance::Missing, _) => summary.missing += 1,
                (StoredBalance::Unusable, _) => summary.unusable += 1,
                (StoredBalance::Numbers(numbers), Some(row)) => {
                    if let Some(unit) = &numbers.currency {
                        let group = summary.groups.entry(unit.clone()).or_insert(BalanceGroup {
                            accounts: 0,
                            remaining: 0.,
                            total: 0.,
                            used: 0.,
                            from: row.recorded_at,
                            to: row.recorded_at,
                        });
                        group.accounts += 1;
                        group.remaining += row.remaining_quota;
                        group.total += row.total_quota;
                        group.used += row.used_quota;
                        group.from = group.from.min(row.recorded_at);
                        group.to = group.to.max(row.recorded_at);
                    } else {
                        summary.unknown_unit += 1;
                    }
                }
                _ => {}
            }
            CheckinAccountRow {
                id: account.id,
                name: redact_log_text(&account.name),
                provider_name: names.get(&account.provider_id).map(|name| (*name).clone()),
                provider_id: account.provider_id,
                enabled: account.enabled,
                last_checkin_at: account.last_checkin_at.map(|time| time.to_rfc3339()),
                last_balance_check_at: account.last_balance_check_at.map(|time| time.to_rfc3339()),
                balance,
            }
        })
        .collect();
    CheckinData {
        accounts,
        providers,
        balances: summary,
        valid_waf_providers: Default::default(),
        records: Vec::new(),
    }
}

fn read_connection(conn: &rusqlite::Connection) -> Result<CheckinSnapshot, CheckinError> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| CheckinError::Storage)?;
    let accounts = checkin_repo::get_all_accounts(&tx).map_err(|_| CheckinError::Storage)?;
    let providers = checkin_repo::get_all_providers(&tx).map_err(|_| CheckinError::Storage)?;
    let mut balances = HashMap::<String, BalanceSnapshot>::new();
    for balance in
        checkin_repo::get_latest_balances_for_all(&tx).map_err(|_| CheckinError::Storage)?
    {
        // The shared reader can return multiple rows at the latest instant.
        let entry = balances.entry(balance.account_id.clone());
        match entry {
            std::collections::hash_map::Entry::Occupied(mut current) => {
                if balance.id > current.get().id {
                    current.insert(balance);
                }
            }
            std::collections::hash_map::Entry::Vacant(current) => {
                current.insert(balance);
            }
        }
    }
    let records = checkin_repo::get_all_records(&tx, 20).map_err(|_| CheckinError::Storage)?;
    let cached = checkin_repo::valid_waf_provider_ids(&tx).map_err(|_| CheckinError::Storage)?;
    tx.commit().map_err(|_| CheckinError::Storage)?;
    let mut data = project_data(accounts, providers, balances);
    data.valid_waf_providers = cached.into_iter().collect();
    data.records = records
        .into_iter()
        .map(|r| RecordSummary {
            account_id: r.account_id,
            status: r.status,
            at: r.checked_in_at.to_rfc3339(),
        })
        .collect();
    Ok(CheckinSnapshot::Accounts(data))
}

fn existing_file(path: PathBuf) -> Result<Option<PathBuf>, CheckinError> {
    match std::fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_file() => Ok(Some(path)),
        Ok(_) => Err(CheckinError::Storage),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(CheckinError::Storage),
    }
}

fn database_path(source: &ConfigSource) -> Result<Option<PathBuf>, CheckinError> {
    match source {
        ConfigSource::Benchmark(layout) => existing_file(layout.path("ui/ccr-ui.db")),
        ConfigSource::Local => {
            let home = dirs::home_dir().ok_or(CheckinError::Storage)?;
            existing_file(home.join(".ccr-ui").join("ccr-ui.db"))
        }
        ConfigSource::Fixture { registry, .. } => {
            if !registry.is_absolute() {
                return Err(CheckinError::Storage);
            }
            let root = registry.parent().ok_or(CheckinError::Storage)?;
            existing_file(root.join("checkin.db"))
        }
    }
}

fn open_reader(path: &Path) -> Result<rusqlite::Connection, CheckinError> {
    let conn = rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| CheckinError::Storage)?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|_| CheckinError::Storage)?;
    Ok(conn)
}

fn read_source(source: &ConfigSource) -> Result<CheckinSnapshot, CheckinError> {
    let Some(path) = database_path(source)? else {
        return Ok(CheckinSnapshot::NoDatabase);
    };
    read_connection(&open_reader(&path)?)
}

impl Services {
    pub fn checkin_request(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<CheckinSnapshot, CheckinError>, AdmissionError> {
        let source = self.source.clone();
        let local = EnvironmentPolicy::Independent
            .admit(&key.environment)
            .is_ok();
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
        let rows = project_data(
            vec![account],
            vec![provider],
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

    fn public_read(root: &Path) -> Result<CheckinSnapshot, CheckinError> {
        use ccr_app::runtime::{Completion, Platform, RuntimeOwner, TaskEvent};
        let runtime = RuntimeOwner::new().unwrap();
        let services = Services::new(
            runtime.handle(),
            ConfigSource::Fixture {
                profiles: root.join("profiles.toml"),
                registry: root.join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let mut result = services
            .checkin_request(RequestKey {
                environment: EnvironmentId::Local("application".into()),
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            })
            .unwrap();
        let terminal = tokio::runtime::Runtime::new().unwrap().block_on(async {
            loop {
                if let TaskEvent::Terminal(value) = result.next().await.unwrap() {
                    break value;
                }
            }
        });
        let result = match terminal.as_ref() {
            Completion::Success(snapshot) => Ok(snapshot.clone()),
            Completion::Failed(error) => Err(*error),
            _ => panic!("reader returned no snapshot"),
        };
        runtime.shutdown();
        result
    }

    #[test]
    fn directory_at_database_path_is_a_read_error() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("checkin.db")).unwrap();
        assert_eq!(public_read(root.path()), Err(CheckinError::Storage));
    }

    #[test]
    fn local_snapshot_groups_only_latest_usable_current_accounts() {
        use ccr_db::{
            database,
            models::checkin::{CheckinAccount, CheckinProvider},
        };
        let root = tempfile::tempdir().unwrap();
        let conn = rusqlite::Connection::open(root.path().join("checkin.db")).unwrap();
        database::migrations::run_all_migrations(&conn, root.path()).unwrap();
        let mut provider = CheckinProvider::new("Example".into(), "https://example.invalid".into());
        provider.builtin_id = Some("example".into());
        checkin_repo::insert_provider(&conn, &provider).unwrap();
        for id in ["a", "b", "zero", "missing", "invalid", "unknown"] {
            let mut account =
                CheckinAccount::new(provider.id.clone(), id.into(), "secret".into(), "7".into());
            account.id = id.into();
            account.enabled = id != "b";
            account.last_balance_check_at = Some(balance(0., 0., 0., "").recorded_at);
            checkin_repo::insert_account(&conn, &account).unwrap();
        }
        conn.pragma_update(None, "foreign_keys", false).unwrap();
        for (id, key, day, currency, total, used, remaining) in [
            ("a", "old", 1, "EUR", 50., 40., 10.),
            ("a", "lower", 4, "USD", 70., 20., 50.),
            ("a", "winner", 4, " USD ", 20., 6., 14.),
            ("b", "b", 2, "USD", 11., 4., 7.),
            ("zero", "c", 5, "CNY", 0., 0., 0.),
            ("invalid", "d", 1, "USD", f64::INFINITY, 1., 1.),
            ("unknown", "e", 1, "   ", 1., 1., 0.),
            ("orphan", "f", 1, "USD", 200., 100., 100.),
        ] {
            let mut row = balance(remaining, total, used, currency);
            row.account_id = id.into();
            row.id = key.into();
            row.recorded_at += chrono::Duration::days(day);
            checkin_repo::insert_balance(&conn, &row).unwrap();
        }
        let CheckinSnapshot::Accounts(data) = public_read(root.path()).unwrap() else {
            panic!("existing DB")
        };
        assert_eq!(data.providers[0].builtin_id.as_deref(), Some("example"));
        let a = data.accounts.iter().find(|row| row.id == "a").unwrap();
        assert_eq!(a.provider_id, provider.id);
        assert_eq!(
            a.last_balance_check_at.as_deref(),
            Some("2024-04-04T00:00:00+00:00")
        );
        let StoredBalance::Numbers(value) = &a.balance else {
            panic!("saved balance")
        };
        assert_eq!(
            (&value.remaining, value.currency.as_deref()),
            (&"14".to_string(), Some("USD"))
        );
        let summary = &data.balances;
        assert_eq!(data.accounts.len(), 6);
        assert_eq!(
            (summary.missing, summary.unusable, summary.unknown_unit),
            (1, 1, 1)
        );
        assert_eq!(summary.groups.len(), 2);
        let usd = &summary.groups["USD"];
        assert_eq!(
            (usd.accounts, usd.remaining, usd.total, usd.used),
            (2, 21., 31., 10.)
        );
        assert_eq!(usd.from.to_rfc3339(), "2024-04-06T00:00:00+00:00");
        assert_eq!(usd.to.to_rfc3339(), "2024-04-08T00:00:00+00:00");
        let cny = &summary.groups["CNY"];
        assert_eq!(
            (cny.accounts, cny.remaining, cny.total, cny.used),
            (1, 0., 0., 0.)
        );
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
