//! AppState 鈥?Tauri 搴旂敤鍏ㄥ眬鐘舵€併€?
//!
//! 鎸佹湁 SQLite 杩炴帴姹犮€丠TTP 瀹㈡埛绔€佸唴瀛樼紦瀛樺拰鎵ц鐜娉ㄥ唽琛ㄣ€?

use std::collections::{HashMap, VecDeque};
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use ccr_db::database::pool::DbPool;
use ccr_db::services::log_persistence::{LogPersistenceService, LogStorageConfig};
use chrono::{DateTime, Utc};
use lru::LruCache;
use tokio::sync::RwLock;

use crate::checkin_jobs::CheckinJobSnapshot;
use crate::events::{EventLog, EventLogStats};
use crate::platform::EnvironmentRegistry;

pub const DEFAULT_CACHE_MAX_ENTRIES: usize = 1000;
pub const DEFAULT_SSH_STATE_TTL_SECS: i64 = 30 * 60;
pub const DEFAULT_SSH_PASSWORD_TTL_SECS: u64 = 10 * 60;
const METRIC_SAMPLE_CAPACITY: usize = 2048;

/// SSH 杩炴帴杩愯鏃剁姸鎬侊紙浠呭唴瀛樻寔鏈夛紝涓嶆寔涔呭寲锛?
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshRuntimeState {
    pub env_id: String,
    pub connected: bool,
    pub has_password: bool,
    pub last_checked_at: Option<String>,
    pub last_error: Option<String>,
}

/// SSH 瀵嗙爜缂撳瓨鏉＄洰锛堜粎鍐呭瓨锛?
#[derive(Debug, Clone)]
pub struct SshPasswordEntry {
    pub value: String,
    pub cached_at: Instant,
}

/// Tauri managed state 鈥?閫氳繃 `app.manage(AppState::new(...))` 娉ㄥ唽銆?
pub struct AppState {
    /// SQLite 杩炴帴姹狅紙鏉ヨ嚜 ccr-db锛?
    pub db_pool: DbPool,

    /// HTTP 瀹㈡埛绔紙澶嶇敤杩炴帴姹狅紝鐢ㄤ簬 CheckIn 绛夊閮ㄨ姹傦級
    pub http_client: reqwest::Client,

    /// 鍐呭瓨缂撳瓨灞傦紙LRU + TTL锛?
    pub cache: RwLock<LruCache<String, CacheEntry>>,

    /// 鎵ц鐜娉ㄥ唽琛紙Local / WSL / SSH锛?
    pub env_registry: RwLock<EnvironmentRegistry>,

    /// SSH 杩炴帴杩愯鏃剁姸鎬侊紙浠呭唴瀛橈級
    pub ssh_runtime_states: RwLock<HashMap<String, SshRuntimeState>>,

    /// SSH 瀵嗙爜缂撳瓨锛堜粎鍐呭瓨锛屼笉鎸佷箙鍖栵級
    pub ssh_password_cache: RwLock<HashMap<String, SshPasswordEntry>>,

    pub monitoring_logs: LogPersistenceService,

    /// ????????
    pub checkin_jobs: RwLock<HashMap<String, CheckinJobSnapshot>>,

    /// 搴旂敤璁剧疆
    pub settings: Mutex<AppSettings>,

    /// 閫€鍑虹‘璁ゆ爣蹇?鈥?鐢ㄤ簬鎵撶牬 CloseRequested 浜嬩欢鐨勫惊鐜?
    pub exit_confirmed: AtomicBool,

    /// 浜嬩欢鏃ュ織鐜舰缂撳啿鍖?
    pub event_log: EventLog,

    /// 鍛戒护鑰楁椂閲囨牱锛堟绉掞級
    command_durations_ms: Mutex<VecDeque<f64>>,

    /// DB 鏌ヨ鑰楁椂閲囨牱锛堟绉掞級
    db_query_durations_ms: Mutex<VecDeque<f64>>,
}

/// 缂撳瓨鏉＄洰
pub struct CacheEntry {
    pub value: serde_json::Value,
    pub expires_at: Instant,
}

/// 搴旂敤璁剧疆
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub skip_exit_confirm: bool,
}

/// 杩愯鏃舵寚鏍囧揩鐓?
#[derive(Debug, Clone, serde::Serialize)]
pub struct RuntimeMetricsSnapshot {
    pub cache_entries: usize,
    pub event_log_entries: usize,
    pub event_log_memory_bytes: usize,
    pub ssh_state_count: usize,
    pub ssh_password_cache_count: usize,
    pub command_p95_ms: Option<f64>,
    pub db_query_p95_ms: Option<f64>,
}

impl AppState {
    /// 鍒涘缓鏂扮殑搴旂敤鐘舵€佸疄渚?
    pub fn new(db_pool: DbPool) -> Self {
        let http_client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        let cache_cap =
            NonZeroUsize::new(DEFAULT_CACHE_MAX_ENTRIES).expect("cache capacity must be non-zero");

        Self {
            db_pool,
            http_client,
            cache: RwLock::new(LruCache::new(cache_cap)),
            env_registry: RwLock::new(EnvironmentRegistry::new()),
            ssh_runtime_states: RwLock::new(HashMap::new()),
            ssh_password_cache: RwLock::new(HashMap::new()),
            monitoring_logs: LogPersistenceService::new(LogStorageConfig::default()),
            checkin_jobs: RwLock::new(HashMap::new()),
            settings: Mutex::new(AppSettings::default()),
            exit_confirmed: AtomicBool::new(false),
            event_log: EventLog::with_limits(500, 10 * 1024, 5 * 1024 * 1024),
            command_durations_ms: Mutex::new(VecDeque::new()),
            db_query_durations_ms: Mutex::new(VecDeque::new()),
        }
    }

    /// 浠庣紦瀛樿幏鍙栧€硷紙鏈繃鏈熸椂杩斿洖 Some锛?
    #[allow(dead_code)]
    pub async fn cache_get(&self, key: &str) -> Option<serde_json::Value> {
        let now = Instant::now();
        let mut cache = self.cache.write().await;
        match cache
            .get(key)
            .map(|entry| (entry.value.clone(), entry.expires_at > now))
        {
            Some((value, true)) => Some(value),
            Some((_value, false)) => {
                cache.pop(key);
                None
            }
            None => None,
        }
    }

    /// 璁剧疆缂撳瓨鍊硷紙鎸囧畾 TTL 绉掓暟锛?
    #[allow(dead_code)]
    pub async fn cache_set(&self, key: String, value: serde_json::Value, ttl_secs: u64) {
        let mut cache = self.cache.write().await;
        cache.put(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + Duration::from_secs(ttl_secs),
            },
        );
    }

    /// 娓呯悊杩囨湡缂撳瓨
    pub async fn cache_cleanup(&self) {
        let now = Instant::now();
        let mut cache = self.cache.write().await;
        let expired_keys: Vec<String> = cache
            .iter()
            .filter_map(|(key, entry)| (entry.expires_at <= now).then_some(key.clone()))
            .collect();
        for key in expired_keys {
            cache.pop(&key);
        }
    }

    /// 娓呯悊杩囨湡 SSH 杩愯鏃剁姸鎬?
    pub async fn cleanup_ssh_runtime_states(&self, max_age_secs: i64) {
        let mut states = self.ssh_runtime_states.write().await;
        if max_age_secs <= 0 {
            states.clear();
            return;
        }

        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_secs);
        states.retain(|_, state| {
            state
                .last_checked_at
                .as_deref()
                .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
                .map(|ts| ts.with_timezone(&Utc) >= cutoff)
                .unwrap_or(false)
        });
    }

    /// 娓呯悊杩囨湡 SSH 瀵嗙爜缂撳瓨
    pub async fn cleanup_ssh_password_cache(&self, max_age_secs: u64) {
        let mut passwords = self.ssh_password_cache.write().await;
        if max_age_secs == 0 {
            passwords.clear();
            return;
        }

        let max_age = Duration::from_secs(max_age_secs);
        passwords.retain(|_, entry| entry.cached_at.elapsed() <= max_age);
    }

    /// 璁板綍鍛戒护鑰楁椂锛堟绉掞級
    pub async fn insert_checkin_job(&self, snapshot: CheckinJobSnapshot) {
        let mut jobs = self.checkin_jobs.write().await;
        jobs.insert(snapshot.job_id.clone(), snapshot);
    }

    pub async fn get_checkin_job(&self, job_id: &str) -> Option<CheckinJobSnapshot> {
        let jobs = self.checkin_jobs.read().await;
        jobs.get(job_id).cloned()
    }

    pub async fn update_checkin_job<F>(&self, job_id: &str, mutator: F) -> Option<CheckinJobSnapshot>
    where
        F: FnOnce(&mut CheckinJobSnapshot),
    {
        let mut jobs = self.checkin_jobs.write().await;
        let snapshot = jobs.get_mut(job_id)?;
        mutator(snapshot);
        Some(snapshot.clone())
    }

    pub fn record_command_duration_ms(&self, duration_ms: f64) {
        if let Ok(mut samples) = self.command_durations_ms.lock() {
            push_sample(&mut samples, duration_ms);
        }
    }

    /// 璁板綍 DB 鏌ヨ鑰楁椂锛堟绉掞級
    pub fn record_db_query_duration_ms(&self, duration_ms: f64) {
        if let Ok(mut samples) = self.db_query_durations_ms.lock() {
            push_sample(&mut samples, duration_ms);
        }
    }

    /// 鑾峰彇杩愯鏃舵寚鏍囧揩鐓?
    pub async fn runtime_metrics_snapshot(&self) -> RuntimeMetricsSnapshot {
        let cache_entries = self.cache.read().await.len();
        let ssh_state_count = self.ssh_runtime_states.read().await.len();
        let ssh_password_cache_count = self.ssh_password_cache.read().await.len();
        let EventLogStats {
            entries: event_log_entries,
            total_size_bytes: event_log_memory_bytes,
            ..
        } = self.event_log.stats().await;

        let command_p95_ms = self
            .command_durations_ms
            .lock()
            .ok()
            .and_then(|samples| percentile_95(&samples));
        let db_query_p95_ms = self
            .db_query_durations_ms
            .lock()
            .ok()
            .and_then(|samples| percentile_95(&samples));

        RuntimeMetricsSnapshot {
            cache_entries,
            event_log_entries,
            event_log_memory_bytes,
            ssh_state_count,
            ssh_password_cache_count,
            command_p95_ms,
            db_query_p95_ms,
        }
    }
}

fn push_sample(samples: &mut VecDeque<f64>, value: f64) {
    if samples.len() >= METRIC_SAMPLE_CAPACITY {
        samples.pop_front();
    }
    samples.push_back(value.max(0.0));
}

fn percentile_95(samples: &VecDeque<f64>) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted: Vec<f64> = samples.iter().copied().collect();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
    let idx = idx.saturating_sub(1).min(sorted.len() - 1);
    Some(sorted[idx])
}



