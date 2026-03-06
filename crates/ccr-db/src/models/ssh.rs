//! SSH 主机与指纹数据模型。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// SSH 主机配置（持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshHost {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub identity_file: Option<String>,
    pub remote_home: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_connected_at: Option<DateTime<Utc>>,
}

/// 新增 SSH 主机请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSshHostRequest {
    pub name: String,
    pub host: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    pub username: String,
    pub identity_file: Option<String>,
    pub remote_home: Option<String>,
}

/// 更新 SSH 主机请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSshHostRequest {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub identity_file: Option<String>,
    pub remote_home: Option<String>,
}

/// 已确认的远端主机指纹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKnownHost {
    pub host: String,
    pub port: u16,
    pub key_type: String,
    pub fingerprint: String,
    pub confirmed_at: DateTime<Utc>,
}

const fn default_ssh_port() -> u16 {
    22
}
