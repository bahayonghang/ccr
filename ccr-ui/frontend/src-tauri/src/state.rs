//! AppState — Tauri 应用全局状态。
//!
//! 持有 SQLite 连接池、HTTP 客户端、内存缓存和执行环境注册表。

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use ccr_db::database::pool::DbPool;
use tokio::sync::RwLock;

use crate::events::EventLog;
use crate::platform::EnvironmentRegistry;

/// SSH 连接运行时状态（仅内存持有，不持久化）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshRuntimeState {
    pub env_id: String,
    pub connected: bool,
    pub has_password: bool,
    pub last_checked_at: Option<String>,
    pub last_error: Option<String>,
}

/// Tauri managed state — 通过 `app.manage(AppState::new(...))` 注册。
pub struct AppState {
    /// SQLite 连接池（来自 ccr-db）
    pub db_pool: DbPool,

    /// HTTP 客户端（复用连接池，用于 CheckIn 等外部请求）
    pub http_client: reqwest::Client,

    /// 内存缓存层 — 用于高频读取的临时数据（如系统信息、平台检测结果）
    pub cache: RwLock<HashMap<String, CacheEntry>>,

    /// 执行环境注册表（Local / WSL / SSH）
    pub env_registry: RwLock<EnvironmentRegistry>,

    /// SSH 连接运行时状态（仅内存）
    pub ssh_runtime_states: RwLock<HashMap<String, SshRuntimeState>>,

    /// SSH 密码缓存（仅内存，不持久化）
    pub ssh_password_cache: RwLock<HashMap<String, String>>,

    /// 应用设置
    pub settings: Mutex<AppSettings>,

    /// 退出确认标志 — 用于打破 CloseRequested 事件的循环
    pub exit_confirmed: AtomicBool,

    /// 事件日志环形缓冲区
    pub event_log: EventLog,
}

/// 缓存条目
pub struct CacheEntry {
    #[allow(dead_code)]
    pub value: serde_json::Value,
    pub expires_at: std::time::Instant,
}

/// 应用设置
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub skip_exit_confirm: bool,
}

impl AppState {
    /// 创建新的应用状态实例
    pub fn new(db_pool: DbPool) -> Self {
        let http_client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        Self {
            db_pool,
            http_client,
            cache: RwLock::new(HashMap::new()),
            env_registry: RwLock::new(EnvironmentRegistry::new()),
            ssh_runtime_states: RwLock::new(HashMap::new()),
            ssh_password_cache: RwLock::new(HashMap::new()),
            settings: Mutex::new(AppSettings::default()),
            exit_confirmed: AtomicBool::new(false),
            event_log: EventLog::new(500),
        }
    }

    /// 从缓存获取值（未过期时返回 Some）
    #[allow(dead_code)]
    pub async fn cache_get(&self, key: &str) -> Option<serde_json::Value> {
        let cache = self.cache.read().await;
        cache.get(key).and_then(|entry| {
            if entry.expires_at > std::time::Instant::now() {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    /// 设置缓存值（指定 TTL 秒数）
    #[allow(dead_code)]
    pub async fn cache_set(&self, key: String, value: serde_json::Value, ttl_secs: u64) {
        let mut cache = self.cache.write().await;
        cache.insert(
            key,
            CacheEntry {
                value,
                expires_at: std::time::Instant::now()
                    + std::time::Duration::from_secs(ttl_secs),
            },
        );
    }

    /// 清理过期缓存
    pub async fn cache_cleanup(&self) {
        let mut cache = self.cache.write().await;
        let now = std::time::Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);
    }
}
