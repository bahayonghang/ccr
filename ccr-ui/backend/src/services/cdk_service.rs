// CDK 充值服务
// 负责从外部站点获取 CDK 充值码并自动充值到用户账户
// 支持 runawaytime (fuli.hxi.me)、b4u (tw.b4u.qzz.io)、x666 (up.x666.me)

use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::services::checkin_service::DEFAULT_USER_AGENT;

// ═══════════════════════════════════════════════════════════
// 类型定义
// ═══════════════════════════════════════════════════════════

/// CDK 充值结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkTopupResult {
    /// CDK 类型
    pub cdk_type: String,
    /// 是否整体成功
    pub success: bool,
    /// 汇总消息
    pub message: String,
    /// 获取到的 CDK 码列表
    pub codes_found: Vec<String>,
    /// 成功充值的数量
    pub codes_redeemed: usize,
    /// 失败的充值详情
    pub failed_codes: Vec<CdkRedeemError>,
    /// x666 直接奖励金额 (仅 x666)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_reward: Option<String>,
}

/// CDK 充值失败详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdkRedeemError {
    pub code: String,
    pub error: String,
}

/// CDK 扩展配置 (存储在 account.extra_config JSON 中)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CdkExtraConfig {
    /// fuli.hxi.me cookies (runawaytime)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fuli_cookies: Option<HashMap<String, String>>,
    /// tw.b4u.qzz.io cookies (b4u)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub b4u_cdk_cookies: Option<HashMap<String, String>>,
    /// up.x666.me JWT access_token (x666)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x666_access_token: Option<String>,
}

impl CdkExtraConfig {
    /// 从 JSON 字符串解析
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_default()
    }

    /// 序列化为 JSON 字符串
    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// CDK 服务错误
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum CdkError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("CDK configuration missing: {0}")]
    ConfigMissing(String),
    #[error("CDK fetch failed: {0}")]
    FetchFailed(String),
    #[error("Topup failed: {0}")]
    TopupFailed(String),
    #[error("JSON parse error: {0}")]
    JsonParse(String),
}

pub type Result<T> = std::result::Result<T, CdkError>;

// ═══════════════════════════════════════════════════════════
// CDK 服务
// ═══════════════════════════════════════════════════════════

/// CDK 充值服务
/// 从外部福利站获取 CDK 充值码，并自动充值到 provider 账户
pub struct CdkService {
    client: Client,
}

impl CdkService {
    /// 创建新的 CDK 服务实例
    pub fn new(proxy_url: Option<String>) -> Self {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .user_agent(DEFAULT_USER_AGENT)
            .no_proxy();

        if let Some(url) = proxy_url.as_deref()
            && let Ok(proxy) = Proxy::all(url)
        {
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder
            .build()
            .expect("Failed to create HTTP client");
        Self { client }
    }

    // ═══════════════════════════════════════════════════════════
    // 主入口
    // ═══════════════════════════════════════════════════════════

    /// 根据 CDK 类型执行获取 + 充值流程
    pub async fn fetch_and_topup(
        &self,
        cdk_type: &str,
        extra_config: &CdkExtraConfig,
        topup_url: Option<&str>,
        topup_cookies: &HashMap<String, String>,
        topup_api_user: &str,
    ) -> CdkTopupResult {
        match cdk_type {
            "runawaytime" => {
                self.process_runawaytime(extra_config, topup_url, topup_cookies, topup_api_user)
                    .await
            }
            "b4u" => {
                self.process_b4u(extra_config, topup_url, topup_cookies, topup_api_user)
                    .await
            }
            "x666" => self.process_x666(extra_config).await,
            _ => CdkTopupResult {
                cdk_type: cdk_type.to_string(),
                success: false,
                message: format!("Unknown CDK type: {}", cdk_type),
                codes_found: vec![],
                codes_redeemed: 0,
                failed_codes: vec![],
                direct_reward: None,
            },
        }
    }

    // ═══════════════════════════════════════════════════════════
    // RunAnytime (fuli.hxi.me)
    // ═══════════════════════════════════════════════════════════

    /// RunAnytime CDK 流程: 签到 + 大转盘 → CDK 码 → topup
    async fn process_runawaytime(
        &self,
        extra_config: &CdkExtraConfig,
        topup_url: Option<&str>,
        topup_cookies: &HashMap<String, String>,
        topup_api_user: &str,
    ) -> CdkTopupResult {
        let fuli_cookies = match &extra_config.fuli_cookies {
            Some(c) if !c.is_empty() => c.clone(),
            _ => {
                return CdkTopupResult {
                    cdk_type: "runawaytime".to_string(),
                    success: false,
                    message: "Missing fuli.hxi.me cookies in extra_config".to_string(),
                    codes_found: vec![],
                    codes_redeemed: 0,
                    failed_codes: vec![],
                    direct_reward: None,
                };
            }
        };

        let topup_url = match topup_url {
            Some(url) => url.to_string(),
            None => {
                return CdkTopupResult {
                    cdk_type: "runawaytime".to_string(),
                    success: false,
                    message: "Missing topup URL".to_string(),
                    codes_found: vec![],
                    codes_redeemed: 0,
                    failed_codes: vec![],
                    direct_reward: None,
                };
            }
        };

        let mut all_codes = Vec::new();

        // Step 1: 签到获取 CDK
        match self.runawaytime_checkin(&fuli_cookies).await {
            Ok(Some(code)) => {
                info!("[RunAnytime] Checkin CDK: {}", code);
                all_codes.push(code);
            }
            Ok(None) => debug!("[RunAnytime] Checkin: already checked in or no code"),
            Err(e) => warn!("[RunAnytime] Checkin failed: {}", e),
        }

        // Step 2: 大转盘获取 CDK
        match self.runawaytime_wheel_spin(&fuli_cookies).await {
            Ok(codes) => {
                info!("[RunAnytime] Wheel CDKs: {:?}", codes);
                all_codes.extend(codes);
            }
            Err(e) => warn!("[RunAnytime] Wheel spin failed: {}", e),
        }

        if all_codes.is_empty() {
            return CdkTopupResult {
                cdk_type: "runawaytime".to_string(),
                success: true,
                message: "No CDK codes obtained (already collected today)".to_string(),
                codes_found: vec![],
                codes_redeemed: 0,
                failed_codes: vec![],
                direct_reward: None,
            };
        }

        // Step 3: 逐个充值
        let codes_found = all_codes.clone();
        let (redeemed, failed) = self
            .topup_codes(&topup_url, &all_codes, topup_cookies, topup_api_user)
            .await;

        CdkTopupResult {
            cdk_type: "runawaytime".to_string(),
            success: !codes_found.is_empty(),
            message: format!(
                "Found {} codes, redeemed {}, failed {}",
                codes_found.len(),
                redeemed,
                failed.len()
            ),
            codes_found,
            codes_redeemed: redeemed,
            failed_codes: failed,
            direct_reward: None,
        }
    }

    /// fuli.hxi.me 签到
    async fn runawaytime_checkin(
        &self,
        cookies: &HashMap<String, String>,
    ) -> Result<Option<String>> {
        let cookie_str = cookie_header_string(cookies);
        let base = "https://fuli.hxi.me";

        // 检查签到状态
        let (status, body) = self
            .get_json(&format!("{}/api/checkin/status", base), &cookie_str)
            .await?;

        if !status.is_success() {
            return Err(CdkError::FetchFailed(format!(
                "Checkin status check failed: {}",
                status
            )));
        }

        // 如果已签到则跳过
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body)
            && json.get("checked_in").and_then(|v| v.as_bool()) == Some(true)
        {
            debug!("[RunAnytime] Already checked in today");
            return Ok(None);
        }

        // 执行签到
        let (status, body) = self
            .post_json(&format!("{}/api/checkin", base), &cookie_str, None)
            .await?;

        if !status.is_success() {
            return Err(CdkError::FetchFailed(format!(
                "Checkin failed: {} - {}",
                status, body
            )));
        }

        // 提取 CDK 码
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(code) = json.get("code").and_then(|v| v.as_str())
                && !code.is_empty()
            {
                return Ok(Some(code.to_string()));
            }
            // 也尝试 data.code
            if let Some(code) = json
                .get("data")
                .and_then(|d| d.get("code"))
                .and_then(|v| v.as_str())
                && !code.is_empty()
            {
                return Ok(Some(code.to_string()));
            }
        }

        Ok(None)
    }

    /// fuli.hxi.me 大转盘
    async fn runawaytime_wheel_spin(
        &self,
        cookies: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        let cookie_str = cookie_header_string(cookies);
        let base = "https://fuli.hxi.me";
        let mut codes = Vec::new();

        // 检查转盘剩余次数
        let (_, body) = self
            .get_json(&format!("{}/api/wheel/status", base), &cookie_str)
            .await?;

        let remaining = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            json.get("remaining")
                .or_else(|| json.get("data").and_then(|d| d.get("remaining")))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
        } else {
            0
        };

        debug!("[RunAnytime] Wheel spins remaining: {}", remaining);

        // 循环执行转盘
        for i in 0..remaining {
            debug!("[RunAnytime] Wheel spin {}/{}", i + 1, remaining);

            match self
                .post_json(&format!("{}/api/wheel", base), &cookie_str, None)
                .await
            {
                Ok((status, body)) => {
                    if status.is_success()
                        && let Ok(json) = serde_json::from_str::<serde_json::Value>(&body)
                    {
                        // 尝试多种字段名
                        let code = json
                            .get("code")
                            .or_else(|| json.get("data").and_then(|d| d.get("code")))
                            .or_else(|| json.get("prize").and_then(|d| d.get("code")))
                            .and_then(|v| v.as_str());

                        if let Some(code) = code
                            && !code.is_empty()
                            && code != "谢谢参与"
                        {
                            codes.push(code.to_string());
                        }
                    }
                }
                Err(e) => {
                    warn!("[RunAnytime] Wheel spin {} failed: {}", i + 1, e);
                }
            }

            // 小延迟避免请求过快
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Ok(codes)
    }

    // ═══════════════════════════════════════════════════════════
    // B4U (tw.b4u.qzz.io)
    // ═══════════════════════════════════════════════════════════

    /// B4U CDK 流程: luckydraw 抽奖 → CDK 码 → topup
    async fn process_b4u(
        &self,
        extra_config: &CdkExtraConfig,
        topup_url: Option<&str>,
        topup_cookies: &HashMap<String, String>,
        topup_api_user: &str,
    ) -> CdkTopupResult {
        let cdk_cookies = match &extra_config.b4u_cdk_cookies {
            Some(c) if !c.is_empty() => c.clone(),
            _ => {
                return CdkTopupResult {
                    cdk_type: "b4u".to_string(),
                    success: false,
                    message: "Missing tw.b4u.qzz.io cookies in extra_config".to_string(),
                    codes_found: vec![],
                    codes_redeemed: 0,
                    failed_codes: vec![],
                    direct_reward: None,
                };
            }
        };

        let topup_url = match topup_url {
            Some(url) => url.to_string(),
            None => {
                return CdkTopupResult {
                    cdk_type: "b4u".to_string(),
                    success: false,
                    message: "Missing topup URL".to_string(),
                    codes_found: vec![],
                    codes_redeemed: 0,
                    failed_codes: vec![],
                    direct_reward: None,
                };
            }
        };

        // 执行抽奖
        let all_codes = match self.b4u_luckydraw(&cdk_cookies).await {
            Ok(codes) => codes,
            Err(e) => {
                return CdkTopupResult {
                    cdk_type: "b4u".to_string(),
                    success: false,
                    message: format!("Luckydraw failed: {}", e),
                    codes_found: vec![],
                    codes_redeemed: 0,
                    failed_codes: vec![],
                    direct_reward: None,
                };
            }
        };

        if all_codes.is_empty() {
            return CdkTopupResult {
                cdk_type: "b4u".to_string(),
                success: true,
                message: "No CDK codes obtained (already drawn today)".to_string(),
                codes_found: vec![],
                codes_redeemed: 0,
                failed_codes: vec![],
                direct_reward: None,
            };
        }

        // 逐个充值
        let codes_found = all_codes.clone();
        let (redeemed, failed) = self
            .topup_codes(&topup_url, &all_codes, topup_cookies, topup_api_user)
            .await;

        CdkTopupResult {
            cdk_type: "b4u".to_string(),
            success: !codes_found.is_empty(),
            message: format!(
                "Found {} codes, redeemed {}, failed {}",
                codes_found.len(),
                redeemed,
                failed.len()
            ),
            codes_found,
            codes_redeemed: redeemed,
            failed_codes: failed,
            direct_reward: None,
        }
    }

    /// B4U luckydraw 抽奖 (Next.js Server Actions 格式)
    async fn b4u_luckydraw(&self, cookies: &HashMap<String, String>) -> Result<Vec<String>> {
        let cookie_str = cookie_header_string(cookies);
        let base = "https://tw.b4u.qzz.io";
        let mut codes = Vec::new();

        // Next.js Server Actions 使用特殊的 next-action header
        // 这些 action ID 可能会随部署变化，需要从页面获取
        // 先尝试检查剩余次数
        let remaining = match self.b4u_check_remaining(&cookie_str, base).await {
            Ok(n) => n,
            Err(e) => {
                warn!("[B4U] Failed to check remaining draws: {}", e);
                1 // 默认尝试 1 次
            }
        };

        debug!("[B4U] Remaining draws: {}", remaining);

        for i in 0..remaining {
            debug!("[B4U] Drawing {}/{}", i + 1, remaining);

            match self.b4u_draw_once(&cookie_str, base).await {
                Ok(Some(code)) => {
                    info!("[B4U] Got CDK: {}", code);
                    codes.push(code);
                }
                Ok(None) => debug!("[B4U] Draw {}: no code (thank you prize)", i + 1),
                Err(e) => warn!("[B4U] Draw {} failed: {}", i + 1, e),
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Ok(codes)
    }

    /// 检查 B4U 剩余抽奖次数
    async fn b4u_check_remaining(&self, cookie_str: &str, base: &str) -> Result<u64> {
        // POST to /luckydraw with Next.js Server Action
        let response = self
            .client
            .post(format!("{}/luckydraw", base))
            .header("Cookie", cookie_str)
            .header("Content-Type", "text/plain;charset=UTF-8")
            .header("Accept", "text/x-component")
            .header("Next-Action", "check-remaining") // Action ID for checking
            .body("[]")
            .send()
            .await?;

        let body = response.text().await?;

        // Next.js Server Actions 返回格式: 0:value\n1:value\n...
        // 尝试解析剩余次数
        for line in body.lines() {
            if let Some(json_str) = line.strip_prefix("1:")
                && let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str)
                && let Some(remaining) = json
                    .get("remaining")
                    .or_else(|| json.get("draws_remaining"))
                    .and_then(|v| v.as_u64())
            {
                return Ok(remaining);
            }
        }

        // 如果无法解析，默认 1 次
        Ok(1)
    }

    /// 执行一次 B4U 抽奖
    async fn b4u_draw_once(&self, cookie_str: &str, base: &str) -> Result<Option<String>> {
        let response = self
            .client
            .post(format!("{}/luckydraw", base))
            .header("Cookie", cookie_str)
            .header("Content-Type", "text/plain;charset=UTF-8")
            .header("Accept", "text/x-component")
            .header("Next-Action", "draw") // Action ID for drawing
            .body(r#"[{"excludeThankYou":false}]"#)
            .send()
            .await?;

        let body = response.text().await?;

        // 解析 Next.js Server Actions 响应
        for line in body.lines() {
            // 尝试解析每一行的 JSON
            let json_str = if let Some(s) = line.strip_prefix("1:") {
                s
            } else if let Some(s) = line.strip_prefix("0:") {
                s
            } else {
                continue;
            };

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                // 查找 CDK 码
                let code = json
                    .get("code")
                    .or_else(|| json.get("cdk"))
                    .or_else(|| json.get("prize").and_then(|p| p.get("code")))
                    .or_else(|| json.get("data").and_then(|d| d.get("code")))
                    .and_then(|v| v.as_str());

                if let Some(code) = code
                    && !code.is_empty()
                    && !code.contains("谢谢")
                    && !code.contains("thank")
                {
                    return Ok(Some(code.to_string()));
                }
            }
        }

        Ok(None)
    }

    // ═══════════════════════════════════════════════════════════
    // x666 (up.x666.me)
    // ═══════════════════════════════════════════════════════════

    /// x666 流程: 抽奖 → 奖励直接到账 (无需 topup)
    async fn process_x666(&self, extra_config: &CdkExtraConfig) -> CdkTopupResult {
        let access_token = match &extra_config.x666_access_token {
            Some(t) if !t.is_empty() => t.clone(),
            _ => {
                return CdkTopupResult {
                    cdk_type: "x666".to_string(),
                    success: false,
                    message: "Missing x666 access_token in extra_config".to_string(),
                    codes_found: vec![],
                    codes_redeemed: 0,
                    failed_codes: vec![],
                    direct_reward: None,
                };
            }
        };

        let base = "https://up.x666.me";

        // Step 1: 检查是否已抽奖
        let check_result = self
            .client
            .get(format!("{}/api/checkin/status", base))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await;

        match check_result {
            Ok(response) => {
                let body = response.text().await.unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    let already_done = json
                        .get("checked_in")
                        .or_else(|| json.get("has_spun"))
                        .or_else(|| json.get("data").and_then(|d| d.get("has_spun")))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if already_done {
                        return CdkTopupResult {
                            cdk_type: "x666".to_string(),
                            success: true,
                            message: "Already spun today".to_string(),
                            codes_found: vec![],
                            codes_redeemed: 0,
                            failed_codes: vec![],
                            direct_reward: None,
                        };
                    }
                }
            }
            Err(e) => {
                warn!("[x666] Status check failed (continuing): {}", e);
            }
        }

        // Step 2: 执行抽奖
        let spin_result = self
            .client
            .post(format!("{}/api/checkin/spin", base))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .send()
            .await;

        match spin_result {
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();

                if !status.is_success() {
                    return CdkTopupResult {
                        cdk_type: "x666".to_string(),
                        success: false,
                        message: format!("Spin failed: {} - {}", status, body),
                        codes_found: vec![],
                        codes_redeemed: 0,
                        failed_codes: vec![],
                        direct_reward: None,
                    };
                }

                // 解析奖励
                let reward = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    let reward_str = json
                        .get("reward")
                        .or_else(|| json.get("data").and_then(|d| d.get("reward")))
                        .or_else(|| json.get("prize"))
                        .map(|v| {
                            if let Some(s) = v.as_str() {
                                s.to_string()
                            } else {
                                v.to_string()
                            }
                        });

                    let msg = json
                        .get("message")
                        .or_else(|| json.get("msg"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Spin completed");

                    info!("[x666] Spin result: {} (reward: {:?})", msg, reward_str);
                    reward_str
                } else {
                    Some(body.clone())
                };

                CdkTopupResult {
                    cdk_type: "x666".to_string(),
                    success: true,
                    message: "Spin completed, reward credited to account".to_string(),
                    codes_found: vec![],
                    codes_redeemed: 0,
                    failed_codes: vec![],
                    direct_reward: reward,
                }
            }
            Err(e) => CdkTopupResult {
                cdk_type: "x666".to_string(),
                success: false,
                message: format!("Spin request failed: {}", e),
                codes_found: vec![],
                codes_redeemed: 0,
                failed_codes: vec![],
                direct_reward: None,
            },
        }
    }

    // ═══════════════════════════════════════════════════════════
    // 通用 Topup 充值
    // ═══════════════════════════════════════════════════════════

    /// 将 CDK 码逐个充值到目标 provider
    async fn topup_codes(
        &self,
        topup_url: &str,
        codes: &[String],
        cookies: &HashMap<String, String>,
        api_user: &str,
    ) -> (usize, Vec<CdkRedeemError>) {
        let mut redeemed = 0;
        let mut failed = Vec::new();

        for code in codes {
            match self.topup_single(topup_url, code, cookies, api_user).await {
                Ok(true) => {
                    info!("[Topup] Code {} redeemed successfully", code);
                    redeemed += 1;
                }
                Ok(false) => {
                    warn!("[Topup] Code {} already used or invalid", code);
                    failed.push(CdkRedeemError {
                        code: code.clone(),
                        error: "Code already used or invalid".to_string(),
                    });
                }
                Err(e) => {
                    warn!("[Topup] Code {} failed: {}", code, e);
                    failed.push(CdkRedeemError {
                        code: code.clone(),
                        error: e.to_string(),
                    });
                }
            }

            // 小延迟避免请求过快
            tokio::time::sleep(Duration::from_millis(300)).await;
        }

        (redeemed, failed)
    }

    /// 充值单个 CDK 码: POST /api/user/topup
    async fn topup_single(
        &self,
        topup_url: &str,
        code: &str,
        cookies: &HashMap<String, String>,
        api_user: &str,
    ) -> Result<bool> {
        let cookie_str = cookie_header_string(cookies);

        let body = serde_json::json!({
            "key": code
        });

        let mut request = self
            .client
            .post(topup_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-Requested-With", "XMLHttpRequest");

        if !cookie_str.is_empty() {
            request = request.header("Cookie", &cookie_str);
        }

        if !api_user.is_empty() {
            request = request.header("new-api-user", api_user);
        }

        let response = request.json(&body).send().await?;

        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();

        debug!("[Topup] Response: {} - {}", status, response_body);

        if !status.is_success() {
            return Err(CdkError::TopupFailed(format!(
                "HTTP {}: {}",
                status, response_body
            )));
        }

        // 解析响应
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_body) {
            let success = json
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                || json.get("code").and_then(|v| v.as_i64()) == Some(0)
                || json.get("code").and_then(|v| v.as_i64()) == Some(200)
                || json
                    .get("status")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

            if !success {
                let msg = json
                    .get("message")
                    .or_else(|| json.get("msg"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Err(CdkError::TopupFailed(msg.to_string()));
            }

            return Ok(true);
        }

        // 非 JSON 响应，按 HTTP 状态码判断
        Ok(status.is_success())
    }

    // ═══════════════════════════════════════════════════════════
    // HTTP 辅助方法
    // ═══════════════════════════════════════════════════════════

    /// GET 请求并返回 JSON
    async fn get_json(&self, url: &str, cookie_str: &str) -> Result<(reqwest::StatusCode, String)> {
        let mut request = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .header("X-Requested-With", "XMLHttpRequest");

        if !cookie_str.is_empty() {
            request = request.header("Cookie", cookie_str);
        }

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        Ok((status, body))
    }

    /// POST 请求并返回 JSON
    async fn post_json(
        &self,
        url: &str,
        cookie_str: &str,
        body: Option<&str>,
    ) -> Result<(reqwest::StatusCode, String)> {
        let mut request = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-Requested-With", "XMLHttpRequest");

        if !cookie_str.is_empty() {
            request = request.header("Cookie", cookie_str);
        }

        if let Some(body) = body {
            request = request.body(body.to_string());
        }

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        Ok((status, body))
    }
}

// ═══════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════

/// Cookie HashMap → header string
fn cookie_header_string(cookies: &HashMap<String, String>) -> String {
    cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_cdk_extra_config_parse() {
        let json = r#"{"fuli_cookies":{"session":"abc123"},"x666_access_token":"jwt_token"}"#;
        let config = CdkExtraConfig::from_json(json);

        assert!(config.fuli_cookies.is_some());
        assert_eq!(
            config.fuli_cookies.unwrap().get("session").unwrap(),
            "abc123"
        );
        assert_eq!(config.x666_access_token.unwrap(), "jwt_token");
        assert!(config.b4u_cdk_cookies.is_none());
    }

    #[test]
    fn test_cdk_extra_config_empty() {
        let config = CdkExtraConfig::from_json("{}");
        assert!(config.fuli_cookies.is_none());
        assert!(config.b4u_cdk_cookies.is_none());
        assert!(config.x666_access_token.is_none());
    }

    #[test]
    fn test_cdk_extra_config_invalid() {
        let config = CdkExtraConfig::from_json("not valid json");
        assert!(config.fuli_cookies.is_none());
    }

    #[test]
    fn test_cdk_extra_config_roundtrip() {
        let config = CdkExtraConfig {
            x666_access_token: Some("test_token".to_string()),
            ..Default::default()
        };

        let json = config.to_json();
        let parsed = CdkExtraConfig::from_json(&json);

        assert_eq!(parsed.x666_access_token.unwrap(), "test_token");
    }

    #[test]
    fn test_cookie_header_string() {
        let mut cookies = HashMap::new();
        cookies.insert("session".to_string(), "abc".to_string());

        let header = cookie_header_string(&cookies);
        assert_eq!(header, "session=abc");
    }

    #[test]
    fn test_cookie_header_string_empty() {
        let cookies = HashMap::new();
        let header = cookie_header_string(&cookies);
        assert!(header.is_empty());
    }
}
