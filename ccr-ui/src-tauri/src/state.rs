//! AppState —— Tauri 应用全局状态。
//!
//! 持有 SQLite 连接池、HTTP 客户端、内存缓存和执行环境注册表。

use std::collections::{HashMap, VecDeque};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use ccr_core::core::AtomicWriter;
use ccr_db::database::pool::DbPool;
use ccr_db::services::log_persistence::{LogPersistenceService, LogStorageConfig};
use chrono::{DateTime, Utc};
use lru::LruCache;
use tokio::sync::{Notify, RwLock};

use crate::checkin_jobs::CheckinJobSnapshot;
use crate::events::{EventLog, EventLogStats};
use crate::platform::EnvironmentRegistry;
use crate::session_index_jobs::{SessionIndexJobSnapshot, SessionIndexJobStatus};
use crate::usage_jobs::{UsageImportJobSnapshot, UsageImportJobStatus};

pub const DEFAULT_CACHE_MAX_ENTRIES: usize = 1000;
pub const DEFAULT_SSH_STATE_TTL_SECS: i64 = 30 * 60;
pub const DEFAULT_SSH_PASSWORD_TTL_SECS: u64 = 10 * 60;
const METRIC_SAMPLE_CAPACITY: usize = 2048;
const DESKTOP_SHELL_PREFS_FILE: &str = "desktop-shell.json";

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

/// Tray 点击锚点（物理像素）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TrayAnchor {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Tray 紧凑面板的手动位置（物理像素）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TrayPanelManualPosition {
    pub x: i32,
    pub y: i32,
}

/// Tray 紧凑面板的定位模式
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrayPanelPlacementMode {
    #[default]
    Anchored,
    Manual,
}

/// Tray 紧凑面板的持久化布局状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TrayPanelPlacementState {
    #[serde(default)]
    pub placement_mode: TrayPanelPlacementMode,
    #[serde(default)]
    pub manual_position: Option<TrayPanelManualPosition>,
}

impl Default for TrayPanelPlacementState {
    fn default() -> Self {
        Self {
            placement_mode: TrayPanelPlacementMode::Anchored,
            manual_position: None,
        }
    }
}

/// Tauri managed state —— 通过 `app.manage(AppState::new(...))` 注册。
pub struct AppState {
    /// SQLite 连接池（来自 ccr-db）
    pub db_pool: DbPool,

    /// Usage archive SQLite 连接池（durable analytics，位于 ~/.ccr/analytics/usage.db）
    pub usage_db_pool: DbPool,

    /// HTTP 客户端（复用连接池，用于 CheckIn 等外部请求）
    pub http_client: reqwest::Client,

    /// 内存缓存层（LRU + TTL）
    pub cache: RwLock<LruCache<String, CacheEntry>>,

    /// 缓存填充中的请求去重表
    inflight_cache_keys: RwLock<HashMap<String, Arc<Notify>>>,

    /// 执行环境注册表（Local / WSL / SSH）
    pub env_registry: RwLock<EnvironmentRegistry>,

    /// SSH 连接运行时状态（仅内存）
    pub ssh_runtime_states: RwLock<HashMap<String, SshRuntimeState>>,

    /// SSH 密码缓存（仅内存，不持久化）
    pub ssh_password_cache: RwLock<HashMap<String, SshPasswordEntry>>,

    pub monitoring_logs: LogPersistenceService,

    /// 签到任务快照
    pub checkin_jobs: RwLock<HashMap<String, CheckinJobSnapshot>>,

    /// Usage 导入后台任务快照
    pub usage_import_jobs: RwLock<HashMap<String, UsageImportJobSnapshot>>,

    /// 当前活跃的 Usage 导入任务 ID
    active_usage_import_job_id: RwLock<Option<String>>,

    /// Session 索引后台任务快照
    pub session_index_jobs: RwLock<HashMap<String, SessionIndexJobSnapshot>>,

    /// 当前活跃的 Session 索引任务 ID
    active_session_index_job_id: RwLock<Option<String>>,

    /// 桌面壳层偏好（持久化到 ~/.ccr/desktop-shell.json）
    pub settings: Mutex<DesktopShellPreferences>,

    /// 退出确认标志 —— 用于打破 CloseRequested 事件循环
    pub exit_confirmed: AtomicBool,

    /// 强制退出标志 —— 用于 tray quit 等路径绕过 hide-to-tray 逻辑
    pub force_exit_requested: AtomicBool,

    /// 最近一次 tray 事件的图标锚点，用于定位紧凑面板
    pub tray_anchor: Mutex<Option<TrayAnchor>>,

    /// 最近一次 tray 菜单里可切换账号列表（按菜单顺序）
    pub tray_switch_accounts: Mutex<Vec<String>>,

    /// tray 紧凑面板是否正处于原生拖拽中
    pub tray_panel_drag_active: AtomicBool,

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

/// 桌面壳层偏好
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DesktopShellPreferences {
    #[serde(default = "default_confirm_before_exit")]
    pub confirm_before_exit: bool,
    #[serde(default)]
    pub close_to_tray: bool,
    #[serde(default = "default_open_panel_on_tray_click")]
    pub open_panel_on_tray_click: bool,
    #[serde(default)]
    pub tray_panel: TrayPanelPlacementState,
}

fn default_confirm_before_exit() -> bool {
    true
}

fn default_open_panel_on_tray_click() -> bool {
    true
}

impl Default for DesktopShellPreferences {
    fn default() -> Self {
        Self {
            confirm_before_exit: true,
            close_to_tray: false,
            open_panel_on_tray_click: true,
            tray_panel: TrayPanelPlacementState::default(),
        }
    }
}

impl DesktopShellPreferences {
    fn default_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
        Ok(home.join(".ccr").join(DESKTOP_SHELL_PREFS_FILE))
    }

    pub fn load() -> Result<Self, String> {
        Self::load_from_path(&Self::default_path()?)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| format!("读取桌面壳层偏好失败: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("解析桌面壳层偏好失败: {e}"))
    }

    pub fn save(&self) -> Result<(), String> {
        Self::save_to_path(&Self::default_path()?, self)
    }

    pub fn save_to_path(path: &Path, value: &Self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(value)
            .map_err(|e| format!("序列化桌面壳层偏好失败: {e}"))?;
        AtomicWriter::new(path)
            .write_string(&content)
            .map_err(|e| format!("写入桌面壳层偏好失败: {e}"))
    }
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
    pub fn new(db_pool: DbPool, usage_db_pool: DbPool) -> Self {
        let http_client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        let settings = DesktopShellPreferences::load().unwrap_or_else(|error| {
            tracing::warn!("[app] failed to load desktop shell preferences: {error}");
            DesktopShellPreferences::default()
        });

        let cache_cap =
            NonZeroUsize::new(DEFAULT_CACHE_MAX_ENTRIES).expect("cache capacity must be non-zero");

        Self {
            db_pool,
            usage_db_pool,
            http_client,
            cache: RwLock::new(LruCache::new(cache_cap)),
            inflight_cache_keys: RwLock::new(HashMap::new()),
            env_registry: RwLock::new(EnvironmentRegistry::new()),
            ssh_runtime_states: RwLock::new(HashMap::new()),
            ssh_password_cache: RwLock::new(HashMap::new()),
            monitoring_logs: LogPersistenceService::new(LogStorageConfig::default()),
            checkin_jobs: RwLock::new(HashMap::new()),
            usage_import_jobs: RwLock::new(HashMap::new()),
            active_usage_import_job_id: RwLock::new(None),
            session_index_jobs: RwLock::new(HashMap::new()),
            active_session_index_job_id: RwLock::new(None),
            settings: Mutex::new(settings),
            exit_confirmed: AtomicBool::new(false),
            force_exit_requested: AtomicBool::new(false),
            tray_anchor: Mutex::new(None),
            tray_switch_accounts: Mutex::new(Vec::new()),
            tray_panel_drag_active: AtomicBool::new(false),
            event_log: EventLog::with_limits(500, 10 * 1024, 5 * 1024 * 1024),
            command_durations_ms: Mutex::new(VecDeque::new()),
            db_query_durations_ms: Mutex::new(VecDeque::new()),
        }
    }

    /// 读取当前桌面壳层偏好快照
    pub fn desktop_shell_preferences(&self) -> Result<DesktopShellPreferences, String> {
        self.settings
            .lock()
            .map(|settings| settings.clone())
            .map_err(|e| format!("Failed to access desktop shell preferences: {e}"))
    }

    /// 更新并持久化桌面壳层偏好
    pub fn update_desktop_shell_preferences<F>(
        &self,
        updater: F,
    ) -> Result<DesktopShellPreferences, String>
    where
        F: FnOnce(&mut DesktopShellPreferences),
    {
        let mut settings = self
            .settings
            .lock()
            .map_err(|e| format!("Failed to access desktop shell preferences: {e}"))?;
        updater(&mut settings);
        settings.save()?;
        Ok(settings.clone())
    }

    /// 记录最近一次 tray 图标锚点
    pub fn set_tray_anchor(&self, anchor: Option<TrayAnchor>) -> Result<(), String> {
        let mut tray_anchor = self
            .tray_anchor
            .lock()
            .map_err(|e| format!("Failed to access tray anchor: {e}"))?;
        *tray_anchor = anchor;
        Ok(())
    }

    /// 获取最近一次 tray 图标锚点
    pub fn tray_anchor(&self) -> Result<Option<TrayAnchor>, String> {
        self.tray_anchor
            .lock()
            .map(|anchor| anchor.clone())
            .map_err(|e| format!("Failed to access tray anchor: {e}"))
    }

    /// 覆盖 tray 菜单里的可切换账号映射
    pub fn set_tray_switch_accounts(&self, accounts: Vec<String>) -> Result<(), String> {
        let mut switch_accounts = self
            .tray_switch_accounts
            .lock()
            .map_err(|e| format!("Failed to access tray switch accounts: {e}"))?;
        *switch_accounts = accounts;
        Ok(())
    }

    /// 通过菜单下标解析账号名
    pub fn tray_switch_account_name(&self, index: usize) -> Result<Option<String>, String> {
        self.tray_switch_accounts
            .lock()
            .map(|accounts| accounts.get(index).cloned())
            .map_err(|e| format!("Failed to access tray switch accounts: {e}"))
    }

    /// 标记 tray 面板是否处于拖拽过程，避免拖动期间被 blur 自动隐藏。
    pub fn set_tray_panel_drag_active(&self, active: bool) {
        self.tray_panel_drag_active
            .store(active, std::sync::atomic::Ordering::SeqCst);
    }

    /// 读取 tray 面板拖拽标记
    pub fn tray_panel_drag_active(&self) -> bool {
        self.tray_panel_drag_active
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// 将 tray 面板位置切换为手动模式并持久化
    pub fn set_tray_panel_manual_position(
        &self,
        position: TrayPanelManualPosition,
    ) -> Result<DesktopShellPreferences, String> {
        self.update_desktop_shell_preferences(|settings| {
            settings.tray_panel.placement_mode = TrayPanelPlacementMode::Manual;
            settings.tray_panel.manual_position = Some(position);
        })
    }

    /// 清理 tray 面板手动定位，回退到 anchored 模式
    pub fn reset_tray_panel_manual_position(&self) -> Result<DesktopShellPreferences, String> {
        self.update_desktop_shell_preferences(|settings| {
            settings.tray_panel.placement_mode = TrayPanelPlacementMode::Anchored;
            settings.tray_panel.manual_position = None;
        })
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

    /// 删除指定缓存值
    pub async fn cache_remove(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.pop(key);
    }

    /// 注册缓存填充任务；已存在时返回等待中的 notifier。
    pub async fn begin_cache_fill(&self, key: &str) -> CacheFillRegistration {
        let mut inflight = self.inflight_cache_keys.write().await;

        if let Some(notify) = inflight.get(key) {
            return CacheFillRegistration::Wait(Arc::clone(notify));
        }

        let notify = Arc::new(Notify::new());
        inflight.insert(key.to_string(), notify);
        CacheFillRegistration::Leader
    }

    /// 结束缓存填充任务并唤醒等待者。
    pub async fn finish_cache_fill(&self, key: &str) {
        let mut inflight = self.inflight_cache_keys.write().await;
        if let Some(notify) = inflight.remove(key) {
            notify.notify_waiters();
        }
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

    pub async fn update_checkin_job<F>(
        &self,
        job_id: &str,
        mutator: F,
    ) -> Option<CheckinJobSnapshot>
    where
        F: FnOnce(&mut CheckinJobSnapshot),
    {
        let mut jobs = self.checkin_jobs.write().await;
        let snapshot = jobs.get_mut(job_id)?;
        mutator(snapshot);
        Some(snapshot.clone())
    }

    pub async fn insert_usage_import_job(&self, snapshot: UsageImportJobSnapshot) {
        let mut jobs = self.usage_import_jobs.write().await;
        jobs.insert(snapshot.job_id.clone(), snapshot.clone());
        let mut active = self.active_usage_import_job_id.write().await;
        *active = Some(snapshot.job_id);
    }

    pub async fn get_usage_import_job(&self, job_id: &str) -> Option<UsageImportJobSnapshot> {
        let jobs = self.usage_import_jobs.read().await;
        jobs.get(job_id).cloned()
    }

    pub async fn get_active_usage_import_job(&self) -> Option<UsageImportJobSnapshot> {
        let active_job_id = self.active_usage_import_job_id.read().await.clone()?;
        let jobs = self.usage_import_jobs.read().await;
        let snapshot = jobs.get(&active_job_id)?.clone();
        if matches!(
            snapshot.status,
            UsageImportJobStatus::Finished | UsageImportJobStatus::Failed
        ) {
            return None;
        }
        Some(snapshot)
    }

    pub async fn update_usage_import_job<F>(
        &self,
        job_id: &str,
        mutator: F,
    ) -> Option<UsageImportJobSnapshot>
    where
        F: FnOnce(&mut UsageImportJobSnapshot),
    {
        let mut jobs = self.usage_import_jobs.write().await;
        let snapshot = jobs.get_mut(job_id)?;
        mutator(snapshot);
        let cloned = snapshot.clone();
        drop(jobs);

        if matches!(
            cloned.status,
            UsageImportJobStatus::Finished | UsageImportJobStatus::Failed
        ) {
            let mut active = self.active_usage_import_job_id.write().await;
            if active.as_deref() == Some(job_id) {
                *active = None;
            }
        }

        Some(cloned)
    }

    pub async fn insert_session_index_job(&self, snapshot: SessionIndexJobSnapshot) {
        let mut jobs = self.session_index_jobs.write().await;
        jobs.insert(snapshot.job_id.clone(), snapshot.clone());
        let mut active = self.active_session_index_job_id.write().await;
        *active = Some(snapshot.job_id);
    }

    pub async fn get_session_index_job(&self, job_id: &str) -> Option<SessionIndexJobSnapshot> {
        let jobs = self.session_index_jobs.read().await;
        jobs.get(job_id).cloned()
    }

    pub async fn get_active_session_index_job(&self) -> Option<SessionIndexJobSnapshot> {
        let active_job_id = self.active_session_index_job_id.read().await.clone()?;
        let jobs = self.session_index_jobs.read().await;
        let snapshot = jobs.get(&active_job_id)?.clone();
        if matches!(
            snapshot.status,
            SessionIndexJobStatus::Finished | SessionIndexJobStatus::Failed
        ) {
            return None;
        }
        Some(snapshot)
    }

    pub async fn update_session_index_job<F>(
        &self,
        job_id: &str,
        mutator: F,
    ) -> Option<SessionIndexJobSnapshot>
    where
        F: FnOnce(&mut SessionIndexJobSnapshot),
    {
        let mut jobs = self.session_index_jobs.write().await;
        let snapshot = jobs.get_mut(job_id)?;
        mutator(snapshot);
        let cloned = snapshot.clone();
        drop(jobs);

        if matches!(
            cloned.status,
            SessionIndexJobStatus::Finished | SessionIndexJobStatus::Failed
        ) {
            let mut active = self.active_session_index_job_id.write().await;
            if active.as_deref() == Some(job_id) {
                *active = None;
            }
        }

        Some(cloned)
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

#[cfg(test)]
mod tests {
    use super::{
        DesktopShellPreferences, TrayPanelManualPosition, TrayPanelPlacementMode,
        TrayPanelPlacementState,
    };

    #[test]
    fn desktop_shell_preferences_round_trip_via_json_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let path = temp_dir.path().join("desktop-shell.json");
        let prefs = DesktopShellPreferences {
            confirm_before_exit: false,
            close_to_tray: true,
            open_panel_on_tray_click: false,
            tray_panel: TrayPanelPlacementState {
                placement_mode: TrayPanelPlacementMode::Manual,
                manual_position: Some(TrayPanelManualPosition { x: 640, y: 96 }),
            },
        };

        DesktopShellPreferences::save_to_path(&path, &prefs).expect("prefs should save");
        let loaded = DesktopShellPreferences::load_from_path(&path).expect("prefs should load");

        assert_eq!(loaded, prefs);
    }

    #[test]
    fn desktop_shell_preferences_defaults_when_file_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let path = temp_dir.path().join("missing.json");

        let loaded = DesktopShellPreferences::load_from_path(&path).expect("prefs should load");

        assert_eq!(loaded, DesktopShellPreferences::default());
    }

    #[test]
    fn desktop_shell_preferences_load_legacy_files_with_default_tray_panel_state() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let path = temp_dir.path().join("legacy.json");

        std::fs::write(
            &path,
            r#"{
                "confirm_before_exit": false,
                "close_to_tray": true,
                "open_panel_on_tray_click": false
            }"#,
        )
        .expect("legacy prefs should write");

        let loaded = DesktopShellPreferences::load_from_path(&path).expect("prefs should load");

        assert!(!loaded.confirm_before_exit);
        assert!(loaded.close_to_tray);
        assert!(!loaded.open_panel_on_tray_click);
        assert_eq!(loaded.tray_panel, TrayPanelPlacementState::default());
    }
}

pub enum CacheFillRegistration {
    Leader,
    Wait(Arc<Notify>),
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
