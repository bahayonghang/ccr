// 导入/导出数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::CheckinProvider;

/// 导出格式版本
pub const EXPORT_VERSION: &str = "1.0";

/// 导出的账号数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAccount {
    /// 账号 ID (导入时可能会重新生成)
    pub id: String,
    /// 关联的提供商 ID
    pub provider_id: String,
    /// 账号备注名称
    pub name: String,
    /// Cookies JSON (加密或明文，取决于导出选项)
    pub cookies_json: String,
    /// 是否加密
    pub cookies_json_encrypted: bool,
    /// API User ID
    pub api_user: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 导出数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    /// 导出格式版本
    pub version: String,
    /// 导出时间
    pub exported_at: DateTime<Utc>,
    /// 提供商列表
    pub providers: Vec<CheckinProvider>,
    /// 账号列表
    pub accounts: Vec<ExportAccount>,
}

impl ExportData {
    /// 创建新的导出数据
    pub fn new(providers: Vec<CheckinProvider>, accounts: Vec<ExportAccount>) -> Self {
        Self {
            version: EXPORT_VERSION.to_string(),
            exported_at: Utc::now(),
            providers,
            accounts,
        }
    }

    /// 检查版本兼容性
    #[allow(dead_code)]
    pub fn is_compatible(&self) -> bool {
        // 目前只有 1.0 版本，直接返回 true
        // 将来可以添加版本兼容性检查逻辑
        self.version == EXPORT_VERSION
    }
}

/// 导出选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportOptions {
    /// 是否导出明文 Cookies (危险！需要用户确认)
    #[serde(default)]
    pub include_plaintext_keys: bool,
    /// 是否仅导出提供商 (不包含账号)
    #[serde(default)]
    pub providers_only: bool,
}

/// 导入冲突策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ImportConflictStrategy {
    /// 跳过已存在的配置
    #[default]
    Skip,
    /// 覆盖已存在的配置
    Overwrite,
}

/// 导入选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    /// 冲突处理策略
    #[serde(default)]
    pub conflict_strategy: ImportConflictStrategy,
    /// 是否仅导入提供商
    #[serde(default)]
    pub providers_only: bool,
    /// 是否仅导入账号
    #[serde(default)]
    pub accounts_only: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            conflict_strategy: ImportConflictStrategy::Skip,
            providers_only: false,
            accounts_only: false,
        }
    }
}

/// 导入预览项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreviewItem {
    /// 项目类型 ("provider" 或 "account")
    pub item_type: String,
    /// 项目名称
    pub name: String,
    /// 项目 ID
    pub id: String,
    /// 是否与现有配置冲突
    pub has_conflict: bool,
    /// 冲突的现有项目名称 (如果有)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_with: Option<String>,
}

/// 导入预览响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreviewResponse {
    /// 版本兼容性
    pub version_compatible: bool,
    /// 导出版本
    pub export_version: String,
    /// 预览项目列表
    pub items: Vec<ImportPreviewItem>,
    /// 新增的提供商数量
    pub new_providers: usize,
    /// 冲突的提供商数量
    pub conflicting_providers: usize,
    /// 新增的账号数量
    pub new_accounts: usize,
    /// 冲突的账号数量
    pub conflicting_accounts: usize,
    /// Cookies 是否加密
    pub keys_encrypted: bool,
    /// 警告消息列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 导入的提供商数量
    pub providers_imported: usize,
    /// 跳过的提供商数量
    pub providers_skipped: usize,
    /// 导入的账号数量
    pub accounts_imported: usize,
    /// 跳过的账号数量
    pub accounts_skipped: usize,
    /// 需要重新输入 Cookies 的账号数量
    pub accounts_need_reauth: usize,
    /// 警告消息列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl ImportResult {
    /// 创建成功结果
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            providers_imported: 0,
            providers_skipped: 0,
            accounts_imported: 0,
            accounts_skipped: 0,
            accounts_need_reauth: 0,
            warnings: Vec::new(),
        }
    }

    /// 创建失败结果
    #[allow(dead_code)]
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            providers_imported: 0,
            providers_skipped: 0,
            accounts_imported: 0,
            accounts_skipped: 0,
            accounts_need_reauth: 0,
            warnings: Vec::new(),
        }
    }

    /// 添加警告
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_data_version() {
        let export = ExportData::new(vec![], vec![]);
        assert_eq!(export.version, EXPORT_VERSION);
        assert!(export.is_compatible());
    }

    #[test]
    fn test_default_export_options() {
        let options = ExportOptions::default();
        assert!(!options.include_plaintext_keys);
        assert!(!options.providers_only);
    }

    #[test]
    fn test_default_import_options() {
        let options = ImportOptions::default();
        assert_eq!(options.conflict_strategy, ImportConflictStrategy::Skip);
        assert!(!options.providers_only);
        assert!(!options.accounts_only);
    }

    #[test]
    fn test_import_result() {
        let mut result = ImportResult::success("导入成功");
        result.providers_imported = 2;
        result.accounts_imported = 5;
        result.add_warning("部分账号需要重新输入 API Key");

        assert!(result.success);
        assert_eq!(result.providers_imported, 2);
        assert_eq!(result.warnings.len(), 1);
    }
}
