//! 🏥 Provider 健康检查服务
//!
//! 测试 Provider 端点的连通性和 API Key 有效性。

use crate::core::error::{CcrError, Result};
use crate::core::http::HTTP_CLIENT;
use crate::managers::config::ConfigSection;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// 🏥 健康检查服务
pub struct HealthCheckService {
    timeout: Duration,
}

/// 📊 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Provider 名称
    pub provider_name: String,
    /// Base URL
    pub base_url: String,
    /// 健康状态
    pub status: HealthStatus,
    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
    /// 模型是否可用
    pub model_available: bool,
    /// 可用模型列表
    pub available_models: Vec<String>,
}

/// 🚦 健康状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// ✅ 健康
    Healthy,
    /// ⚠️ 降级（部分功能可用）
    Degraded,
    /// ❌ 不健康
    Unhealthy,
    /// ❓ 未知
    Unknown,
}

#[allow(dead_code)]
impl HealthStatus {
    /// 获取状态显示文本
    pub fn display(&self) -> &str {
        match self {
            HealthStatus::Healthy => "✅ 健康",
            HealthStatus::Degraded => "⚠️ 降级",
            HealthStatus::Unhealthy => "❌ 不健康",
            HealthStatus::Unknown => "❓ 未知",
        }
    }

    /// 获取状态颜色
    pub fn color(&self) -> &str {
        match self {
            HealthStatus::Healthy => "green",
            HealthStatus::Degraded => "yellow",
            HealthStatus::Unhealthy => "red",
            HealthStatus::Unknown => "gray",
        }
    }
}

impl Default for HealthCheckService {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl HealthCheckService {
    /// 创建新的健康检查服务
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(10),
        }
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 测试单个 Provider
    pub async fn check(&self, name: &str, config: &ConfigSection) -> HealthCheckResult {
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());

        let api_key = config.auth_token.clone().unwrap_or_else(|| {
            debug!("Provider {} 未配置 API Key", name);
            String::new()
        });

        info!("检查 Provider: {} ({})", name, base_url);

        let start = Instant::now();

        // 尝试获取模型列表
        let models_result = self.fetch_models(&base_url, &api_key).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match models_result {
            Ok(models) => {
                let model_available = if let Some(ref model) = config.model {
                    models.iter().any(|m| m == model)
                } else {
                    true
                };

                let status = if model_available {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Degraded
                };

                HealthCheckResult {
                    provider_name: name.to_string(),
                    base_url,
                    status,
                    latency_ms: Some(latency_ms),
                    error: None,
                    model_available,
                    available_models: models,
                }
            }
            Err(e) => {
                warn!("Provider {} 检查失败: {}", name, e);

                HealthCheckResult {
                    provider_name: name.to_string(),
                    base_url,
                    status: HealthStatus::Unhealthy,
                    latency_ms: Some(latency_ms),
                    error: Some(e.to_string()),
                    model_available: false,
                    available_models: vec![],
                }
            }
        }
    }

    /// 获取模型列表
    async fn fetch_models(&self, base_url: &str, api_key: &str) -> Result<Vec<String>> {
        let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

        debug!("请求模型列表: {}", url);

        let client = &*HTTP_CLIENT;
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| CcrError::NetworkError(format!("请求失败: {}", e)))?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CcrError::NetworkError("API Key 无效".to_string()));
        }

        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(CcrError::NetworkError("访问被拒绝".to_string()));
        }

        if !status.is_success() {
            return Err(CcrError::NetworkError(format!("HTTP 状态码: {}", status)));
        }

        // 尝试解析响应
        let body = response
            .text()
            .await
            .map_err(|e| CcrError::NetworkError(format!("读取响应失败: {}", e)))?;

        // 尝试解析 OpenAI 格式
        if let Ok(openai_response) = serde_json::from_str::<OpenAIModelsResponse>(&body) {
            return Ok(openai_response.data.into_iter().map(|m| m.id).collect());
        }

        // 尝试解析 Anthropic 格式
        if let Ok(anthropic_response) = serde_json::from_str::<AnthropicModelsResponse>(&body) {
            return Ok(anthropic_response.data.into_iter().map(|m| m.id).collect());
        }

        // 如果无法解析，返回空列表但不报错
        debug!("无法解析模型列表响应: {}", &body[..body.len().min(200)]);
        Ok(vec![])
    }

    /// 简单连通性测试（仅测试是否可达）
    pub async fn ping(&self, base_url: &str) -> Result<Duration> {
        let start = Instant::now();

        let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

        let client = &*HTTP_CLIENT;
        let response = client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| CcrError::NetworkError(format!("连接失败: {}", e)))?;

        // 即使返回 401/403 也说明服务可达
        let _status = response.status();

        Ok(start.elapsed())
    }

    /// 测试 API Key 有效性
    pub async fn verify_api_key(&self, base_url: &str, api_key: &str) -> Result<bool> {
        let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

        let client = &*HTTP_CLIENT;
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| CcrError::NetworkError(format!("请求失败: {}", e)))?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Ok(false);
        }

        Ok(status.is_success())
    }
}

/// OpenAI 模型列表响应
#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
}

/// Anthropic 模型列表响应
#[derive(Debug, Deserialize)]
struct AnthropicModelsResponse {
    data: Vec<AnthropicModel>,
}

#[derive(Debug, Deserialize)]
struct AnthropicModel {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.display(), "✅ 健康");
        assert_eq!(HealthStatus::Unhealthy.display(), "❌ 不健康");
    }
}
