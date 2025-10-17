// 🎯 CCR 服务层模块
// 封装业务逻辑,提供统一的业务操作接口
//
// 服务层职责:
// - 📦 封装业务逻辑(配置管理、设置管理、历史记录等)
// - 🔄 协调多个 Manager 的操作
// - 📝 提供事务性操作(备份+修改+历史记录)
// - ✅ 统一错误处理和验证

pub mod backup_service;
pub mod config_service;
pub mod history_service;
pub mod settings_service;
pub mod sync_service;

// Service 层为将来扩展准备,部分功能暂未在命令层使用
#[allow(unused_imports)]
pub use backup_service::BackupService;
#[allow(unused_imports)]
pub use config_service::ConfigService;
#[allow(unused_imports)]
pub use history_service::HistoryService;
#[allow(unused_imports)]
pub use settings_service::SettingsService;
#[allow(unused_imports)]
pub use sync_service::SyncService;
