//! Bilingual safe authentication messages, shared by native adapters.
/// Resolve a stable authentication error code without rendering server-controlled text.
pub fn authentication_message(code: &str, chinese: bool) -> Option<&'static str> {
    Some(match (code, chinese) {
        ("ssh_file_too_large", false) => "Remote file or output exceeds the size limit.",
        ("ssh_file_too_large", true) => "远程文件或输出超过大小上限。",
        ("ssh_session_changed", false) => {
            "This document belongs to a previous connection. Reload before saving; your draft is retained."
        }
        ("ssh_session_changed", true) => "文件属于之前的连接。请重新读取后保存；草稿已保留。",
        ("ssh_tested", false) => "Connection test succeeded. No host or credential was saved.",
        ("ssh_tested", true) => "测试连接成功，未保存主机或凭据。",
        ("ssh_invalid_auth_method", false) => "Choose agent, private_key or password.",
        ("ssh_invalid_auth_method", true) => "认证方式必须为 agent、private_key 或 password。",
        ("ssh_credentials_missing", false) => {
            "Credentials missing. Edit the host and enter the password or key passphrase again."
        }
        ("ssh_credentials_missing", true) => "凭据缺失。请编辑主机，重新输入密码或私钥口令。",
        ("ssh_credential_store_unavailable", false) => {
            "System credential store unavailable. No plaintext fallback is used."
        }
        ("ssh_credential_store_unavailable", true) => "系统凭据库不可用，不会回退到明文存储。",
        ("ssh_private_key_invalid", false) => {
            "Cannot load private key. Check the path, format and passphrase."
        }
        ("ssh_private_key_invalid", true) => "无法读取私钥。请检查路径、格式和口令。",
        ("ssh_auth_interactive_required", false) => {
            "Server requires additional interactive authentication. Use an SSH agent instead."
        }
        ("ssh_auth_interactive_required", true) => "服务器要求额外交互认证。请改用 SSH agent。",
        ("ssh_auth_failed", false) => {
            "Authentication failed. Server offered no supported authentication method."
        }
        ("ssh_auth_failed", true) => "认证失败。服务器未提供支持的认证方式。",
        ("ssh_auth_failed_key", false) => "Authentication failed. Server allows: publickey.",
        ("ssh_auth_failed_key", true) => "认证失败。服务器允许：publickey。",
        ("ssh_auth_failed_password", false) => "Authentication failed. Server allows: password.",
        ("ssh_auth_failed_password", true) => "认证失败。服务器允许：password。",
        ("ssh_auth_failed_keyboard", false) => {
            "Authentication failed. Server allows: keyboard-interactive."
        }
        ("ssh_auth_failed_keyboard", true) => "认证失败。服务器允许：keyboard-interactive。",
        ("ssh_auth_failed_key_password", false) => {
            "Authentication failed. Server allows: publickey, password."
        }
        ("ssh_auth_failed_key_password", true) => "认证失败。服务器允许：publickey、password。",
        ("ssh_auth_failed_key_keyboard", false) => {
            "Authentication failed. Server allows: publickey, keyboard-interactive."
        }
        ("ssh_auth_failed_key_keyboard", true) => {
            "认证失败。服务器允许：publickey、keyboard-interactive。"
        }
        ("ssh_auth_failed_password_keyboard", false) => {
            "Authentication failed. Server allows: password, keyboard-interactive."
        }
        ("ssh_auth_failed_password_keyboard", true) => {
            "认证失败。服务器允许：password、keyboard-interactive。"
        }
        ("ssh_auth_failed_all", false) => {
            "Authentication failed. Server allows: publickey, password, keyboard-interactive."
        }
        ("ssh_auth_failed_all", true) => {
            "认证失败。服务器允许：publickey、password、keyboard-interactive。"
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn environment_io_codes_have_safe_bilingual_messages() {
        for code in ["ssh_file_too_large", "ssh_session_changed"] {
            for chinese in [false, true] {
                let message = super::authentication_message(code, chinese).unwrap_or_default();
                assert!(!message.is_empty());
                assert!(!message.contains(code));
            }
        }
    }
}
