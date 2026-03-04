//! SSH 密钥认证工具 — 密钥发现、校验与认证方式推断。
//!
//! 说明：
//! - 扫描 `~/.ssh/` 目录中的标准及自定义密钥文件
//! - 通过文件头部标识检测密钥类型（RSA / Ed25519 / ECDSA）
//! - 使用 `ssh-keygen -lf` 获取密钥指纹（异步调用）
//! - 不依赖 russh 库，仅使用系统 OpenSSH 工具

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::platform::ssh::SshHostConfig;

/// SSH 认证方式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum SshAuthMethod {
    /// 密钥文件认证
    KeyFile { path: String, key_type: String },
    /// 密码认证
    Password,
    /// SSH Agent 认证
    Agent,
    /// 无认证信息
    None,
}

/// SSH 密钥文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyInfo {
    /// 密钥文件绝对路径
    pub path: String,
    /// 密钥类型（RSA / Ed25519 / ECDSA / Unknown）
    pub key_type: String,
    /// 是否有密码保护（通过文件头检测 ENCRYPTED 标记）
    pub has_passphrase: bool,
    /// 密钥指纹（通过 ssh-keygen -lf 获取，失败时为 None）
    pub fingerprint: Option<String>,
}

/// 标准密钥文件名（不含扩展名）
const STANDARD_KEY_NAMES: &[&str] = &["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa", "id_ecdsa_sk", "id_ed25519_sk"];

/// 根据私钥文件头部内容检测密钥类型
fn detect_key_type(header: &str) -> &'static str {
    if header.contains("RSA PRIVATE KEY") || header.contains("RSA OPENSSH PRIVATE KEY") {
        "RSA"
    } else if header.contains("OPENSSH PRIVATE KEY") {
        // 需结合文件名或公钥文件进一步区分；默认报告为 OpenSSH 格式
        "Ed25519"
    } else if header.contains("EC PRIVATE KEY") || header.contains("ECDSA PRIVATE KEY") {
        "ECDSA"
    } else {
        "Unknown"
    }
}

/// 根据文件名推断密钥类型（辅助函数）
fn key_type_from_filename(filename: &str) -> Option<&'static str> {
    if filename.contains("rsa") {
        Some("RSA")
    } else if filename.contains("ed25519") {
        Some("Ed25519")
    } else if filename.contains("ecdsa") {
        Some("ECDSA")
    } else if filename.contains("dsa") {
        Some("DSA")
    } else {
        None
    }
}

/// 读取密钥文件头部并判断是否有密码保护
fn parse_key_header(content: &str) -> (String, bool) {
    let key_type = detect_key_type(content);

    // 新式 OpenSSH 格式：根据文件名区分（头部均为 OPENSSH PRIVATE KEY）
    // 旧式 PEM 格式：ENCRYPTED 标记出现在头部
    let has_passphrase = content.contains("Proc-Type: 4,ENCRYPTED")
        || content.contains("DEK-Info:")
        || (content.contains("OPENSSH PRIVATE KEY") && content.contains("bcrypt"));

    (key_type.to_string(), has_passphrase)
}

/// 通过 `ssh-keygen -lf` 获取密钥指纹
async fn get_fingerprint(path: &str) -> Option<String> {
    let output = Command::new("ssh-keygen")
        .arg("-lf")
        .arg(path)
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 输出格式：`2048 SHA256:xxxx user@host (RSA)`
    // 第二个空白分隔的字段即为指纹
    stdout
        .split_whitespace()
        .nth(1)
        .map(|s| s.to_string())
}

/// 扫描 `~/.ssh/` 目录，收集所有可读的私钥文件信息。
///
/// 标准密钥名（`id_rsa`、`id_ed25519`、`id_ecdsa` 等）优先返回；
/// 其余非 `.pub` 文件且能被识别为 PEM/OpenSSH 格式的文件也会被纳入。
pub async fn discover_keys() -> Vec<SshKeyInfo> {
    let ssh_dir = match dirs::home_dir() {
        Some(home) => home.join(".ssh"),
        None => return Vec::new(),
    };

    if !ssh_dir.exists() {
        return Vec::new();
    }

    let mut result = Vec::new();

    // 优先处理标准密钥文件
    for name in STANDARD_KEY_NAMES {
        let path = ssh_dir.join(name);
        if !path.exists() {
            continue;
        }
        if let Ok(info) = validate_key_file(path.to_string_lossy().as_ref()).await {
            result.push(info);
        }
    }

    // 扫描目录中其余非 .pub 文件
    let read_dir = match tokio::fs::read_dir(&ssh_dir).await {
        Ok(rd) => rd,
        Err(_) => return result,
    };

    let mut entries = read_dir;
    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(e)) => e,
            Ok(None) => break,
            Err(_) => break,
        };

        let filename = entry.file_name();
        let name = filename.to_string_lossy();

        // 跳过公钥文件、已知认证文件、known_hosts 等
        if name.ends_with(".pub")
            || name == "authorized_keys"
            || name == "known_hosts"
            || name == "known_hosts.old"
            || name == "config"
        {
            continue;
        }

        // 跳过已处理的标准密钥
        if STANDARD_KEY_NAMES.contains(&name.as_ref()) {
            continue;
        }

        let path_str = entry.path().to_string_lossy().to_string();
        if let Ok(info) = validate_key_file(&path_str).await {
            // 只纳入能识别类型的自定义密钥
            if info.key_type != "Unknown" {
                result.push(info);
            }
        }
    }

    result
}

/// 校验指定路径的密钥文件，返回 `SshKeyInfo`。
///
/// 错误场景：
/// - 文件不存在或无读取权限
/// - 文件内容无法识别为私钥格式
pub async fn validate_key_file(path: &str) -> Result<SshKeyInfo, String> {
    // 检查文件是否存在且可读
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("读取密钥文件失败 ({path}): {e}"))?;

    // 必须包含 PRIVATE KEY 标记才视为私钥
    if !content.contains("PRIVATE KEY") {
        return Err(format!("文件不是有效的私钥格式: {path}"));
    }

    let (mut key_type, has_passphrase) = parse_key_header(&content);

    // 如果类型为 Ed25519（OpenSSH 统一格式），尝试通过文件名进一步区分
    if key_type == "Ed25519" || key_type == "Unknown" {
        let filename = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if let Some(from_name) = key_type_from_filename(&filename) {
            key_type = from_name.to_string();
        }
    }

    // 异步获取指纹（失败不影响整体结果）
    let fingerprint = get_fingerprint(path).await;

    Ok(SshKeyInfo {
        path: path.to_string(),
        key_type,
        has_passphrase,
        fingerprint,
    })
}

/// 根据 `SshHostConfig` 推断最佳认证方式。
///
/// 优先级：显式 identity_file > 发现的密钥 > Agent > None
#[allow(dead_code)]
pub fn resolve_auth_method(config: &SshHostConfig) -> SshAuthMethod {
    // 1. 使用配置中显式指定的私钥
    if let Some(identity) = &config.identity_file {
        let identity = identity.trim();
        if !identity.is_empty() {
            // 同步读取文件头部判断类型（此处仅用于信息展示，不阻塞关键路径）
            let key_type = std::fs::read_to_string(identity)
                .ok()
                .and_then(|content| {
                    if content.contains("PRIVATE KEY") {
                        let (kt, _) = parse_key_header(&content);
                        Some(kt)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());

            return SshAuthMethod::KeyFile {
                path: identity.to_string(),
                key_type,
            };
        }
    }

    // 2. 查找 ~/.ssh/ 中的默认密钥（同步扫描）
    if let Some(home) = dirs::home_dir() {
        for name in STANDARD_KEY_NAMES {
            let path = home.join(".ssh").join(name);
            if !path.exists() {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path)
                && content.contains("PRIVATE KEY")
            {
                let (key_type, _) = parse_key_header(&content);
                return SshAuthMethod::KeyFile {
                    path: path.to_string_lossy().to_string(),
                    key_type,
                };
            }
        }
    }

    // 3. 假定 SSH Agent 可用
    SshAuthMethod::Agent
}
