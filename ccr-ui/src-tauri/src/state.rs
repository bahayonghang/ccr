//! AppState —— Tauri 应用全局状态。
//!
//! 持有 SQLite 连接池、HTTP 客户端、内存缓存和执行环境注册表。

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

/// SSH 连接运行时状态（仅内存持有，不持久化）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshRuntimeState {
    pub env_id: String,
    pub connected: bool,
    pub has_password: bool,
    pub last_checked_at: Option<String>,
    pub last_error: Option<String>,
}

/// SSH 密码缓存条目（仅内存）
#[derive(Debug, Clone)]
pub struct SshPasswordEntry {
    pub value: String,
    pub cached_at: Instant,
}

/// Tauri managed state —— 通过 `app.manage(AppState::new(...))` 注册。
pub struct AppState {
    /// SQLite 连接池（来自 ccr-db）
    pub db_pool: DbPool,

    /// HTTP 客户端（复用连接池，用于 CheckIn 等外部请求）
    pub http_client: reqwest::Client,

    /// 内存缓存层（LRU + TTL）
    pub cache: RwLock<LruCache<String, CacheEntry>>,

    /// 执行环境注册表（Local / WSL / SSH）
    pub env_registry: RwLock<EnvironmentRegistry>,

    /// SSH 连接运行时状态（仅内存）
    pub ssh_runtime_states: RwLock<HashMap<String, SshRuntimeState>>,

    /// SSH 密码缓存（仅内存，不持久化）
    pub ssh_password_cache: RwLock<HashMap<String, SshPasswordEntry>>,

    pub monitoring_logs: LogPersistenceService,

    /// 签到任务快照
    pub checkin_jobs: RwLock<HashMap<String, CheckinJobSnapshot>>,

    /// 应用设置
    pub settings: Mutex<AppSettings>,

    /// 退出确认标志 —— 用于打破 CloseRequested 事件循环
    pub exit_confirmed: AtomicBool,

    /// 事件日志环形缓冲区
    pub event_log: EventLog,

    /// 命令耗时采样（毫秒）
    command_durations_ms: Mutex<VecDeque<f64>>,

    /// DB 查询耗时采样（毫秒）
    db_query_durations_ms: Mutex<VecDeque<f64>>,
}

/// 缓存条目
pub struct CacheEntry {
    pub value: serde_json::Value,
    pub expires_at: Instant,
}

/// 应用设置
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub skip_exit_confirm: bool,
}

/// 运行时指标快照
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
    /// 创建新的应用状态实例
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

    /// 从缓存获取值（未过期时返回 Some）
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

    /// 设置缓存值（指定 TTL 秒数）
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

    /// 清理过期缓存
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

    /// 清理过期 SSH 运行时状态
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

    /// 清理过期 SSH 密码缓存
    pub async fn cleanup_ssh_password_cache(&self, max_age_secs: u64) {
        let mut passwords = self.ssh_password_cache.write().await;
        if max_age_secs == 0 {
            passwords.clear();
            return;
        }

        let max_age = Duration::from_secs(max_age_secs);
        passwords.retain(|_, entry| entry.cached_at.elapsed() <= max_age);
    }

    /// 插入签到任务快照
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

    /// 记录 DB 查询耗时（毫秒）
    pub fn record_db_query_duration_ms(&self, duration_ms: f64) {
        if let Ok(mut samples) = self.db_query_durations_ms.lock() {
            push_sample(&mut samples, duration_ms);
        }
    }

    /// 获取运行时指标快照
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
