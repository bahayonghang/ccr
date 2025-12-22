// 签到服务
// 负责签到业务逻辑，包括执行签到、查询余额等

use crate::core::crypto::CryptoManager;
use crate::managers::checkin::{AccountManager, BalanceManager, ProviderManager, RecordManager};
use crate::models::checkin::{
    BalanceHistoryResponse, BalanceSnapshot, CheckinProvider, CheckinRecord,
    CheckinRecordsResponse, CheckinStatus,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum CheckinServiceError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Account error: {0}")]
    AccountError(String),
    #[error("Crypto error: {0}")]
    CryptoError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Record error: {0}")]
    RecordError(String),
    #[error("Balance error: {0}")]
    BalanceError(String),
}

pub type Result<T> = std::result::Result<T, CheckinServiceError>;

/// new-api 标准签到响应
#[derive(Debug, Deserialize)]
struct NewApiCheckinResponse {
    success: Option<bool>,
    message: Option<String>,
    data: Option<serde_json::Value>,
}

/// new-api 标准用户信息响应
#[derive(Debug, Deserialize)]
struct NewApiDashboardResponse {
    #[allow(dead_code)]
    success: Option<bool>,
    #[allow(dead_code)]
    message: Option<String>,
    data: Option<DashboardData>,
}

#[derive(Debug, Deserialize)]
struct DashboardData {
    #[serde(default)]
    quota: f64,
    #[serde(default)]
    used_quota: f64,
    #[serde(default)]
    #[allow(dead_code)]
    request_count: i64,
}

/// 签到执行结果
#[derive(Debug, Clone, Serialize)]
pub struct CheckinExecutionResult {
    pub account_id: String,
    pub account_name: String,
    pub provider_name: String,
    pub status: CheckinStatus,
    pub message: Option<String>,
    pub reward: Option<String>,
    pub balance: Option<f64>,
}

/// 签到服务
pub struct CheckinService {
    /// 签到数据目录
    checkin_dir: PathBuf,
    /// HTTP 客户端
    client: Client,
}

impl CheckinService {
    /// 创建新的签到服务
    pub fn new(checkin_dir: PathBuf) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            checkin_dir,
            client,
        }
    }

    /// 获取默认签到目录
    pub fn default_checkin_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            CheckinServiceError::ProviderError("Cannot find home directory".to_string())
        })?;
        Ok(home.join(".ccr").join("checkin"))
    }

    /// 执行单个账号签到
    pub async fn checkin(&self, account_id: &str) -> Result<CheckinExecutionResult> {
        let provider_manager = ProviderManager::new(&self.checkin_dir);
        let account_manager = AccountManager::new(&self.checkin_dir);
        let record_manager = RecordManager::new(&self.checkin_dir);
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 获取账号信息
        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        // 获取提供商信息
        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::ProviderError(e.to_string()))?;

        // 检查今日是否已签到
        let already_checked = record_manager
            .has_checked_in_today(account_id)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))?;

        if already_checked {
            let record = CheckinRecord::already_checked_in(
                account_id.to_string(),
                Some("今日已签到".to_string()),
            );
            record_manager
                .add(record)
                .map_err(|e| CheckinServiceError::RecordError(e.to_string()))?;

            return Ok(CheckinExecutionResult {
                account_id: account_id.to_string(),
                account_name: account.name.clone(),
                provider_name: provider.name.clone(),
                status: CheckinStatus::AlreadyCheckedIn,
                message: Some("今日已签到".to_string()),
                reward: None,
                balance: None,
            });
        }

        // 解密 API Key
        let api_key = crypto
            .decrypt(&account.api_key_encrypted)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 执行签到请求
        let checkin_result = self.do_checkin(&provider, &api_key).await;

        // 记录签到结果
        let (record, result) = match checkin_result {
            Ok((message, reward)) => {
                let record = CheckinRecord::success(
                    account_id.to_string(),
                    Some(message.clone()),
                    reward.clone(),
                );

                let result = CheckinExecutionResult {
                    account_id: account_id.to_string(),
                    account_name: account.name.clone(),
                    provider_name: provider.name.clone(),
                    status: CheckinStatus::Success,
                    message: Some(message),
                    reward,
                    balance: None,
                };

                (record, result)
            }
            Err(e) => {
                let error_msg = e.to_string();
                let record = CheckinRecord::failed(account_id.to_string(), error_msg.clone());

                let result = CheckinExecutionResult {
                    account_id: account_id.to_string(),
                    account_name: account.name.clone(),
                    provider_name: provider.name.clone(),
                    status: CheckinStatus::Failed,
                    message: Some(error_msg),
                    reward: None,
                    balance: None,
                };

                (record, result)
            }
        };

        // 保存签到记录
        record_manager
            .add(record)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))?;

        // 更新账号最后签到时间
        let _ = account_manager.update_checkin_time(account_id);

        Ok(result)
    }

    /// 执行签到 HTTP 请求
    async fn do_checkin(
        &self,
        provider: &CheckinProvider,
        api_key: &str,
    ) -> Result<(String, Option<String>)> {
        let url = format!(
            "{}{}",
            provider.base_url.trim_end_matches('/'),
            provider.checkin_path
        );

        let auth_value = if provider.auth_prefix.is_empty() {
            api_key.to_string()
        } else {
            format!("{} {}", provider.auth_prefix, api_key)
        };

        let response = self
            .client
            .post(&url)
            .header(&provider.auth_header, auth_value)
            .send()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CheckinServiceError::ApiError(format!(
                "HTTP {}: {}",
                response.status().as_u16(),
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown error")
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;

        // 尝试解析 new-api 标准响应
        if let Ok(api_response) = serde_json::from_str::<NewApiCheckinResponse>(&body) {
            let success = api_response.success.unwrap_or(true);
            let message = api_response
                .message
                .unwrap_or_else(|| "签到成功".to_string());

            if !success && message.contains("已") {
                // 已签到的情况
                return Ok((message, None));
            }

            // 尝试从 data 中提取奖励信息
            let reward = api_response.data.and_then(|d| {
                if let Some(reward_str) = d.get("reward").and_then(|v| v.as_str()) {
                    Some(reward_str.to_string())
                } else {
                    d.get("points")
                        .and_then(|v| v.as_i64())
                        .map(|points| format!("+{} 积分", points))
                }
            });

            Ok((message, reward))
        } else {
            // 非标准响应，返回原始响应
            Ok((body, None))
        }
    }

    /// 查询账号余额
    pub async fn query_balance(&self, account_id: &str) -> Result<BalanceSnapshot> {
        let provider_manager = ProviderManager::new(&self.checkin_dir);
        let account_manager = AccountManager::new(&self.checkin_dir);
        let balance_manager = BalanceManager::new(&self.checkin_dir);
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 获取账号信息
        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        // 获取提供商信息
        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::ProviderError(e.to_string()))?;

        // 解密 API Key
        let api_key = crypto
            .decrypt(&account.api_key_encrypted)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 查询余额
        let snapshot = self
            .do_query_balance(&provider, &api_key, account_id)
            .await?;

        // 保存余额快照
        balance_manager
            .add(snapshot.clone())
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))?;

        // 更新账号最后余额查询时间
        let _ = account_manager.update_balance_time(account_id);

        Ok(snapshot)
    }

    /// 执行余额查询 HTTP 请求
    async fn do_query_balance(
        &self,
        provider: &CheckinProvider,
        api_key: &str,
        account_id: &str,
    ) -> Result<BalanceSnapshot> {
        let url = format!(
            "{}{}",
            provider.base_url.trim_end_matches('/'),
            provider.balance_path
        );

        let auth_value = if provider.auth_prefix.is_empty() {
            api_key.to_string()
        } else {
            format!("{} {}", provider.auth_prefix, api_key)
        };

        let response = self
            .client
            .get(&url)
            .header(&provider.auth_header, auth_value)
            .send()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CheckinServiceError::ApiError(format!(
                "HTTP {}: {}",
                response.status().as_u16(),
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown error")
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;

        // 尝试解析 new-api 标准响应
        if let Ok(api_response) = serde_json::from_str::<NewApiDashboardResponse>(&body)
            && let Some(data) = api_response.data
        {
            // quota 和 used_quota 通常是 token 数量（整数），转换为金额需要除以倍率
            // 这里假设倍率为 500000 (即 $1 = 500000 tokens)
            let quota_rate = 500000.0;
            let total_quota = data.quota / quota_rate;
            let used_quota = data.used_quota / quota_rate;
            let remaining_quota = total_quota - used_quota;

            return Ok(BalanceSnapshot::new(
                account_id.to_string(),
                total_quota,
                used_quota,
                remaining_quota,
                "USD".to_string(),
            )
            .with_raw_response(body));
        }

        // 非标准响应，尝试其他格式
        Err(CheckinServiceError::ApiError(
            "Unable to parse balance response".to_string(),
        ))
    }

    /// 批量签到
    pub async fn batch_checkin(&self, account_ids: &[String]) -> Vec<CheckinExecutionResult> {
        let mut results = Vec::with_capacity(account_ids.len());

        for account_id in account_ids {
            let result = self.checkin(account_id).await;
            match result {
                Ok(r) => results.push(r),
                Err(e) => {
                    results.push(CheckinExecutionResult {
                        account_id: account_id.clone(),
                        account_name: "Unknown".to_string(),
                        provider_name: "Unknown".to_string(),
                        status: CheckinStatus::Failed,
                        message: Some(e.to_string()),
                        reward: None,
                        balance: None,
                    });
                }
            }
        }

        results
    }

    /// 签到所有启用的账号
    pub async fn checkin_all(&self) -> Vec<CheckinExecutionResult> {
        let account_manager = AccountManager::new(&self.checkin_dir);

        let enabled_accounts = match account_manager.get_enabled_accounts() {
            Ok(accounts) => accounts,
            Err(e) => {
                tracing::error!("Failed to get enabled accounts: {}", e);
                return vec![];
            }
        };

        let account_ids: Vec<String> = enabled_accounts.iter().map(|a| a.id.clone()).collect();
        self.batch_checkin(&account_ids).await
    }

    /// 获取账号签到记录
    pub fn get_checkin_records(
        &self,
        account_id: &str,
        limit: Option<usize>,
    ) -> Result<CheckinRecordsResponse> {
        let record_manager = RecordManager::new(&self.checkin_dir);
        record_manager
            .get_by_account(account_id, limit)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))
    }

    /// 获取所有签到记录
    pub fn get_all_records(&self, limit: Option<usize>) -> Result<CheckinRecordsResponse> {
        let record_manager = RecordManager::new(&self.checkin_dir);
        record_manager
            .get_all(limit)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))
    }

    /// 获取账号余额历史
    pub fn get_balance_history(
        &self,
        account_id: &str,
        limit: Option<usize>,
    ) -> Result<BalanceHistoryResponse> {
        let balance_manager = BalanceManager::new(&self.checkin_dir);
        balance_manager
            .get_history(account_id, limit)
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))
    }

    /// 获取账号最新余额
    #[allow(dead_code)]
    pub fn get_latest_balance(&self, account_id: &str) -> Result<Option<BalanceSnapshot>> {
        let balance_manager = BalanceManager::new(&self.checkin_dir);
        balance_manager
            .get_latest(account_id)
            .map_err(|e| CheckinServiceError::BalanceError(e.to_string()))
    }

    /// 获取今日签到统计
    pub fn get_today_stats(&self) -> Result<TodayCheckinStats> {
        let account_manager = AccountManager::new(&self.checkin_dir);
        let record_manager = RecordManager::new(&self.checkin_dir);

        let all_accounts = account_manager
            .load_all()
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        let enabled_accounts: Vec<_> = all_accounts.iter().filter(|a| a.enabled).collect();
        let account_ids: Vec<String> = enabled_accounts.iter().map(|a| a.id.clone()).collect();

        let stats = record_manager
            .get_today_stats(&account_ids)
            .map_err(|e| CheckinServiceError::RecordError(e.to_string()))?;

        Ok(TodayCheckinStats {
            total_accounts: stats.total,
            checked_in: stats.checked_in,
            not_checked_in: stats.not_checked_in,
            failed: stats.failed,
        })
    }

    /// 测试账号连接
    pub async fn test_connection(&self, account_id: &str) -> Result<bool> {
        let provider_manager = ProviderManager::new(&self.checkin_dir);
        let account_manager = AccountManager::new(&self.checkin_dir);
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 获取账号信息
        let account = account_manager
            .get(account_id)
            .map_err(|e| CheckinServiceError::AccountError(e.to_string()))?;

        // 获取提供商信息
        let provider = provider_manager
            .get(&account.provider_id)
            .map_err(|e| CheckinServiceError::ProviderError(e.to_string()))?;

        // 解密 API Key
        let api_key = crypto
            .decrypt(&account.api_key_encrypted)
            .map_err(|e| CheckinServiceError::CryptoError(e.to_string()))?;

        // 使用 user_info_path 测试连接
        let url = format!(
            "{}{}",
            provider.base_url.trim_end_matches('/'),
            provider.user_info_path
        );

        let auth_value = if provider.auth_prefix.is_empty() {
            api_key.to_string()
        } else {
            format!("{} {}", provider.auth_prefix, api_key)
        };

        let response = self
            .client
            .get(&url)
            .header(&provider.auth_header, auth_value)
            .send()
            .await
            .map_err(|e| CheckinServiceError::NetworkError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

/// 今日签到统计
#[derive(Debug, Clone, Serialize)]
pub struct TodayCheckinStats {
    /// 总账号数
    pub total_accounts: usize,
    /// 今日已签到数
    pub checked_in: usize,
    /// 今日未签到数
    pub not_checked_in: usize,
    /// 今日签到失败数
    pub failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, CheckinService) {
        let temp_dir = TempDir::new().unwrap();
        let service = CheckinService::new(temp_dir.path().to_path_buf());
        (temp_dir, service)
    }

    #[test]
    fn test_default_checkin_dir() {
        let dir = CheckinService::default_checkin_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.ends_with("checkin"));
    }

    #[test]
    fn test_get_today_stats_empty() {
        let (_temp_dir, service) = setup();
        let stats = service.get_today_stats().unwrap();
        assert_eq!(stats.total_accounts, 0);
        assert_eq!(stats.checked_in, 0);
    }
}
