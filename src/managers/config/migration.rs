// 📊 迁移状态信息

use std::path::PathBuf;

/// 📊 迁移状态信息
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// 是否已启用统一模式
    pub is_unified_mode: bool,

    /// Legacy 配置是否存在
    pub legacy_config_exists: bool,

    /// Legacy 配置路径
    pub legacy_config_path: PathBuf,

    /// 统一配置路径(如果存在)
    pub unified_config_path: Option<PathBuf>,

    /// Legacy 配置节数量
    pub legacy_section_count: usize,

    /// 是否应该迁移
    pub should_migrate: bool,
}
