// 📊 platform 命令共享数据结构

use serde::{Deserialize, Serialize};

/// 📊 平台列表 JSON 输出结构
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformListOutput {
    pub config_file: String,
    pub default_platform: String,
    pub current_platform: String,
    pub platforms: Vec<PlatformListItem>,
}

/// 📋 单个平台信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformListItem {
    pub name: String,
    pub is_current: bool,
    pub is_default: bool,
    pub enabled: bool,
    pub current_profile: Option<String>,
    pub description: String,
}

/// 📊 平台详情 JSON 输出结构
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformInfoOutput {
    pub name: String,
    pub display_name: String,
    pub is_current: bool,
    pub enabled: bool,
    pub current_profile: Option<String>,
    pub description: Option<String>,
    pub paths: PlatformPathsOutput,
    pub profiles: Vec<String>,
}

/// 📁 平台路径信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformPathsOutput {
    pub platform_dir: String,
    pub profiles_file: String,
    pub history_file: String,
    pub backups_dir: String,
}
