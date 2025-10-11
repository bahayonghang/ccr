// ⚠️ CCR 统一错误处理模块
// 🔧 提供统一的错误类型、错误码和用户友好的错误消息
//
// 设计原则:
// - 📊 结构化错误类型（使用 thiserror）
// - 🔢 每种错误对应唯一退出码
// - 💬 提供用户友好的错误消息
// - 🚨 区分致命错误和可恢复错误

use thiserror::Error;

/// 🔢 错误退出码常量
///
/// 每种错误类型对应唯一的退出码，便于脚本判断错误类型
///
/// 退出码范围:
/// - 10-19: 配置相关错误
/// - 20-29: 设置相关错误
/// - 30-39: 文件锁相关错误
/// - 40-49: 序列化相关错误
/// - 50-59: IO 相关错误
/// - 80-89: 历史记录相关错误
/// - 90-99: 验证相关错误
pub mod exit_codes {
    /// ⚙️ 配置文件错误
    pub const CONFIG_ERROR: i32 = 10;

    /// 📁 配置文件缺失
    pub const CONFIG_MISSING: i32 = 11;

    /// 🔍 配置节不存在
    pub const CONFIG_SECTION_NOT_FOUND: i32 = 12;

    /// 📋 配置格式无效
    pub const CONFIG_FORMAT_INVALID: i32 = 14;

    /// 📝 设置文件错误
    pub const SETTINGS_ERROR: i32 = 20;

    /// 📁 设置文件缺失
    pub const SETTINGS_MISSING: i32 = 21;

    /// 🔒 文件锁错误
    pub const FILE_LOCK_ERROR: i32 = 30;

    /// ⏱️ 获取锁超时
    pub const LOCK_TIMEOUT: i32 = 31;

    /// 📄 JSON 错误
    pub const JSON_ERROR: i32 = 40;

    /// 📄 TOML 错误
    pub const TOML_ERROR: i32 = 41;

    /// 💾 IO 错误
    pub const IO_ERROR: i32 = 50;

    /// 📚 历史记录错误
    pub const HISTORY_ERROR: i32 = 80;

    /// ✅ 验证错误
    pub const VALIDATION_ERROR: i32 = 90;
}

/// ❌ CCR 错误类型枚举
///
/// 涵盖所有可能的错误情况，每种错误都有专门的退出码
#[derive(Error, Debug)]
pub enum CcrError {
    /// ⚙️ 配置文件相关错误
    #[error("配置文件错误: {0}")]
    ConfigError(String),

    /// 📁 配置文件缺失
    #[error("配置文件不存在: {0}")]
    ConfigMissing(String),

    /// 🔍 配置节不存在
    #[error("配置节 '{0}' 不存在")]
    ConfigSectionNotFound(String),

    /// 📋 配置格式无效
    #[error("配置格式无效: {0}")]
    ConfigFormatInvalid(String),

    /// 📝 设置文件相关错误
    #[error("设置文件错误: {0}")]
    SettingsError(String),

    /// 📁 设置文件缺失
    #[error("设置文件不存在: {0}")]
    SettingsMissing(String),

    /// 🔒 文件锁相关错误
    #[error("文件锁错误: {0}")]
    FileLockError(String),

    /// ⏱️ 获取锁超时
    #[error("获取文件锁超时: {0}")]
    LockTimeout(String),

    /// 📄 JSON 序列化/反序列化错误
    #[error("JSON 错误: {0}")]
    JsonError(#[from] serde_json::Error),

    /// 📄 TOML 序列化/反序列化错误
    #[error("TOML 错误: {0}")]
    TomlError(#[from] toml::de::Error),

    /// 💾 IO 错误
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 📚 历史记录错误
    #[error("历史记录错误: {0}")]
    HistoryError(String),

    /// ✅ 验证失败
    #[error("验证失败: {0}")]
    ValidationError(String),
}

impl CcrError {
    /// 🔢 获取错误退出码
    ///
    /// 每种错误类型对应唯一的退出码，便于脚本判断错误类型
    ///
    /// 使用 [exit_codes] 模块中定义的常量
    pub fn exit_code(&self) -> i32 {
        match self {
            CcrError::ConfigError(_) => exit_codes::CONFIG_ERROR,
            CcrError::ConfigMissing(_) => exit_codes::CONFIG_MISSING,
            CcrError::ConfigSectionNotFound(_) => exit_codes::CONFIG_SECTION_NOT_FOUND,
            CcrError::ConfigFormatInvalid(_) => exit_codes::CONFIG_FORMAT_INVALID,
            CcrError::SettingsError(_) => exit_codes::SETTINGS_ERROR,
            CcrError::SettingsMissing(_) => exit_codes::SETTINGS_MISSING,
            CcrError::FileLockError(_) => exit_codes::FILE_LOCK_ERROR,
            CcrError::LockTimeout(_) => exit_codes::LOCK_TIMEOUT,
            CcrError::JsonError(_) => exit_codes::JSON_ERROR,
            CcrError::TomlError(_) => exit_codes::TOML_ERROR,
            CcrError::IoError(_) => exit_codes::IO_ERROR,
            CcrError::HistoryError(_) => exit_codes::HISTORY_ERROR,
            CcrError::ValidationError(_) => exit_codes::VALIDATION_ERROR,
        }
    }

    /// 🚨 判断错误是否致命
    ///
    /// 致命错误需要立即终止程序执行，无法恢复
    ///
    /// 致命错误类型:
    /// - 配置文件缺失
    /// - 设置文件缺失
    /// - IO 错误
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            CcrError::ConfigMissing(_) | CcrError::SettingsMissing(_) | CcrError::IoError(_)
        )
    }

    /// 💬 生成用户友好的错误消息
    ///
    /// 返回适合直接显示给用户的错误说明和可能的解决建议
    pub fn user_message(&self) -> String {
        match self {
            CcrError::ConfigMissing(path) => {
                format!(
                    "配置文件不存在: {}\n建议: 请运行安装脚本创建配置文件，或检查配置文件路径是否正确",
                    path
                )
            }
            CcrError::ConfigSectionNotFound(name) => {
                format!(
                    "配置节 '{}' 不存在\n建议: 运行 'ccr list' 查看可用配置，或编辑 ~/.ccs_config.toml 添加新配置",
                    name
                )
            }
            CcrError::SettingsMissing(path) => {
                format!(
                    "Claude Code 设置文件不存在: {}\n建议: 请确保已安装 Claude Code，或检查 ~/.claude 目录是否存在",
                    path
                )
            }
            CcrError::LockTimeout(resource) => {
                format!(
                    "获取文件锁超时: {}\n建议: 可能有其他 ccr 进程正在运行，请稍后重试或检查是否有僵死进程",
                    resource
                )
            }
            CcrError::ValidationError(msg) => {
                format!(
                    "验证失败: {}\n建议: 运行 'ccr validate' 查看详细的验证报告",
                    msg
                )
            }
            _ => self.to_string(),
        }
    }
}

/// Result 类型别名，使用 CcrError 作为错误类型
pub type Result<T> = std::result::Result<T, CcrError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(CcrError::ConfigError("test".into()).exit_code(), 10);
        assert_eq!(CcrError::ConfigMissing("test".into()).exit_code(), 11);
        assert_eq!(CcrError::SettingsError("test".into()).exit_code(), 20);
    }

    #[test]
    fn test_is_fatal() {
        assert!(CcrError::ConfigMissing("test".into()).is_fatal());
        assert!(!CcrError::ConfigError("test".into()).is_fatal());
    }

    #[test]
    fn test_user_message() {
        let err = CcrError::ConfigSectionNotFound("test_config".into());
        let msg = err.user_message();
        assert!(msg.contains("test_config"));
        assert!(msg.contains("建议"));
    }
}
