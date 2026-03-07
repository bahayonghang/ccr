// 🎯 CCR 服务层模块
// 封装业务逻辑,提供统一的业务操作接口
//
// 服务层职责:
// - 📦 封装业务逻辑(配置管理、设置管理、历史记录等)
// - 🔄 协调多个 Manager 的操作
// - 📝 提供事务性操作(备份+修改+历史记录)
// - ✅ 统一错误处理和验证

pub mod backup_service;
pub mod codex_auth_service;
pub mod codex_runtime_service;
pub mod codex_usage_service;
pub mod config_service;
pub mod health_check;
pub mod history_service;
pub mod multi_backup_service;
pub mod settings_service;
pub mod sync_service;
pub mod ui_service;
pub mod validate_service;

// Service 层为将来扩展准备,部分功能暂未在命令层使用
#[allow(unused_imports)]
pub use backup_service::BackupService;
#[allow(unused_imports)]
pub use codex_auth_service::CodexAuthService;
#[allow(unused_imports)]
pub use codex_runtime_service::{
    CodexAuthCacheAction, CodexRuntimeCommitPlan, CodexRuntimeService,
};
#[allow(unused_imports)]
pub use codex_usage_service::{CodexRollingUsage, CodexUsageService};
#[allow(unused_imports)]
pub use config_service::ConfigService;
#[allow(unused_imports)]
pub use history_service::HistoryService;
#[allow(unused_imports)]
pub use multi_backup_service::MultiBackupService;
#[allow(unused_imports)]
pub use settings_service::SettingsService;
#[allow(unused_imports)]
pub use sync_service::SyncService;
#[allow(unused_imports)]
pub use ui_service::UiService;
#[allow(unused_imports)]
pub use validate_service::ValidateService;
