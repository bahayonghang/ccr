// re-export 墙条目须有真实消费方；规则见 .trellis/spec/ccr/backend/public-api-boundary.md，逐符号盘点见任务归档 07-03-arch-ccr-facade/research/inventory.md
//
// 🎯 CCR 服务层模块
// 封装业务逻辑,提供统一的业务操作接口
//
// 服务层职责:
// - 📦 封装业务逻辑(配置管理、设置管理、历史记录等)
// - 🔄 协调多个 Manager 的操作
// - 📝 提供事务性操作(备份+修改+历史记录)
// - ✅ 统一错误处理和验证

pub mod backup_service;
pub mod claude_auth_service;
pub mod config_service;
pub mod doctor_service;
pub mod grok_auth_service;
pub mod health_check;
pub mod history_service;
pub mod install_catalog;
pub mod install_detect;
pub mod install_exec;
pub mod install_plan;
pub mod install_ring_buffer;
pub mod install_service;
pub mod install_types;
pub mod multi_backup_service;
pub mod runtime_overview_service;
pub mod settings_service;
#[allow(dead_code)]
pub mod skills_service;
pub mod sync_service;
pub mod ui_service;
pub mod validate_service;

// Service 层为将来扩展准备,部分功能暂未在命令层使用
#[allow(unused_imports)]
pub use backup_service::BackupService;
#[allow(unused_imports)]
pub use ccr_codex::services::codex_session_service;
#[allow(unused_imports)]
pub use ccr_codex::{
    AuthReadSnapshot, CodexAuthService, CodexHistoryBackupPruneResult, CodexHistoryProviderBuckets,
    CodexHistorySyncOptions, CodexHistorySyncResult, CodexHistorySyncService,
    CodexHistorySyncStatus, CodexOAuthTokenService, CodexQuotaService, CodexRollingUsage,
    CodexSessionTrashService, CodexUsageRecord, CodexUsageService,
};
#[allow(unused_imports)]
pub use claude_auth_service::{ClaudeAuthItem, ClaudeAuthService};
#[allow(unused_imports)]
pub use config_service::ConfigService;
pub use grok_auth_service::{GrokAuthCurrent, GrokAuthService};
#[allow(unused_imports)]
pub use history_service::HistoryService;
#[allow(unused_imports)]
pub use multi_backup_service::MultiBackupService;
#[allow(unused_imports)]
pub use runtime_overview_service::{
    PlatformStatusCard, RuntimeOverview, RuntimeOverviewService, StatusAuthKind, StatusHealth,
};
#[allow(unused_imports)]
pub use settings_service::SettingsService;
#[allow(unused_imports)]
pub use skills_service::SkillsService;
#[allow(unused_imports)]
pub use validate_service::ValidateService;
