// CCR 统一错误处理模块
// 提供统一的错误类型、错误码和用户友好的错误消息

use thiserror::Error;

/// CCR 错误类型枚举
#[derive(Error, Debug)]
pub enum CcrError {
    /// 配置文件相关错误
    #[error("配置文件错误: {0}")]
    ConfigError(String),

    /// 配置文件缺失
    #[error("配置文件不存在: {0}")]
    ConfigMissing(String),

    /// 配置节不存在
    #[error("配置节 '{0}' 不存在")]
    ConfigSectionNotFound(String),

    /// 配置格式无效
    #[error("配置格式无效: {0}")]
    ConfigFormatInvalid(String),

    /// 设置文件相关错误
    #[error("设置文件错误: {0}")]
    SettingsError(String),

    /// 设置文件缺失
    #[error("设置文件不存在: {0}")]
    SettingsMissing(String),

    /// 文件锁相关错误
    #[error("文件锁错误: {0}")]
    FileLockError(String),

    /// 获取锁超时
    #[error("获取文件锁超时: {0}")]
    LockTimeout(String),

    /// JSON 序列化/反序列化错误
    #[error("JSON 错误: {0}")]
    JsonError(#[from] serde_json::Error),

    /// TOML 序列化/反序列化错误
    #[error("TOML 错误: {0}")]
    TomlError(#[from] toml::de::Error),

    /// IO 错误
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 历史记录错误
    #[error("历史记录错误: {0}")]
    HistoryError(String),

    /// 验证失败
    #[error("验证失败: {0}")]
    ValidationError(String),
}

impl CcrError {
    /// 获取错误退出码
    ///
    /// 返回一个整数错误码，用于 CLI 程序退出时返回给操作系统
    pub fn exit_code(&self) -> i32 {
        match self {
            CcrError::ConfigError(_) => 10,
            CcrError::ConfigMissing(_) => 11,
            CcrError::ConfigSectionNotFound(_) => 12,
            CcrError::ConfigFormatInvalid(_) => 14,
            CcrError::SettingsError(_) => 20,
            CcrError::SettingsMissing(_) => 21,
            CcrError::FileLockError(_) => 30,
            CcrError::LockTimeout(_) => 31,
            CcrError::JsonError(_) => 40,
            CcrError::TomlError(_) => 41,
            CcrError::IoError(_) => 50,
            CcrError::HistoryError(_) => 80,
            CcrError::ValidationError(_) => 90,
        }
    }

    /// 判断错误是否致命
    ///
    /// 致命错误需要立即终止程序执行
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            CcrError::ConfigMissing(_) | CcrError::SettingsMissing(_) | CcrError::IoError(_)
        )
    }

    /// 生成用户友好的错误消息
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
