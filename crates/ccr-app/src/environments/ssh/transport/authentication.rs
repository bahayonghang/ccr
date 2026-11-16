use super::*;
use russh::{client::KeyboardInteractiveAuthResponse, keys::PrivateKeyWithHashAlg};

pub(super) fn failure(result: russh::client::AuthResult) -> Error {
    let russh::client::AuthResult::Failure {
        remaining_methods, ..
    } = result
    else {
        return "ssh_auth_failed";
    };
    // 只展示协议方法白名单，不把服务器任意文本或认证应答带入错误。
    match (
        remaining_methods.contains(&russh::MethodKind::PublicKey),
        remaining_methods.contains(&russh::MethodKind::Password),
        remaining_methods.contains(&russh::MethodKind::KeyboardInteractive),
    ) {
        (true, true, true) => "ssh_auth_failed_all",
        (true, true, false) => "ssh_auth_failed_key_password",
        (true, false, true) => "ssh_auth_failed_key_keyboard",
        (false, true, true) => "ssh_auth_failed_password_keyboard",
        (true, false, false) => "ssh_auth_failed_key",
        (false, true, false) => "ssh_auth_failed_password",
        (false, false, true) => "ssh_auth_failed_keyboard",
        _ => "ssh_auth_failed",
    }
}
pub(super) async fn authenticate(
    connection: &mut LiveConnection,
    host: &SshHost,
    auth: &Authentication,
) -> Result<(), Error> {
    let handle = Arc::get_mut(&mut connection.0).ok_or("ssh_auth_failed")?;
    if auth.method == AuthMethod::PrivateKey {
        let path = host.identity_file.clone().ok_or("ssh_invalid_target")?;
        let key = load_private_key(path, auth.secret.clone()).await?;
        let hash = handle
            .best_supported_rsa_hash()
            .await
            .map_err(|_| "ssh_network_error")?
            .flatten();
        // RSA 不回退 SHA-1；没有服务器扩展时优先 SHA-256。
        let hash = hash.or(Some(russh::keys::HashAlg::Sha256));
        let result = handle
            .authenticate_publickey(
                &host.username,
                PrivateKeyWithHashAlg::new(Arc::new(key), hash),
            )
            .await
            .map_err(|_| "ssh_network_error")?;
        return if result.success() {
            Ok(())
        } else {
            Err(failure(result))
        };
    }
    let password = auth.secret.as_ref().ok_or("ssh_credentials_missing")?;
    password_authenticate(handle, &host.username, password).await
}

async fn load_private_key(
    path: String,
    secret: Option<Secret>,
) -> Result<russh::keys::PrivateKey, Error> {
    tokio::task::spawn_blocking(move || {
        let content = zeroize::Zeroizing::new(
            std::fs::read_to_string(path).map_err(|_| "ssh_private_key_invalid")?,
        );
        match russh::keys::decode_secret_key(&content, None) {
            Ok(key) => Ok(key),
            Err(error) => {
                if let Some(secret) = secret {
                    return russh::keys::decode_secret_key(&content, Some(secret.expose()))
                        .map_err(|_| "ssh_private_key_invalid");
                }
                let encrypted = matches!(error, russh::keys::Error::KeyIsEncrypted)
                    || content.contains("-----BEGIN ENCRYPTED PRIVATE KEY-----")
                    || content.contains("Proc-Type: 4,ENCRYPTED")
                    || content.lines().any(|line| {
                        line.strip_prefix("Encryption: ")
                            .is_some_and(|value| value.trim() != "none")
                    });
                Err(if encrypted {
                    "ssh_credentials_missing"
                } else {
                    "ssh_private_key_invalid"
                })
            }
        }
    })
    .await
    .map_err(|_| "ssh_private_key_invalid")?
}

// 密码协议交互独立成可替换接缝，fixture 运行与真实连接相同的往返状态机。
trait PasswordExchange {
    fn password<'a>(
        &'a mut self,
        user: &'a str,
        secret: &'a str,
    ) -> RunFuture<'a, russh::client::AuthResult>;
    fn start<'a>(&'a mut self, user: &'a str) -> RunFuture<'a, KeyboardInteractiveAuthResponse>;
    fn respond(&mut self, values: Vec<String>) -> RunFuture<'_, KeyboardInteractiveAuthResponse>;
}
impl PasswordExchange for client::Handle<Verifier> {
    fn password<'a>(
        &'a mut self,
        user: &'a str,
        secret: &'a str,
    ) -> RunFuture<'a, russh::client::AuthResult> {
        Box::pin(async move {
            self.authenticate_password(user, secret)
                .await
                .map_err(|_| "ssh_network_error")
        })
    }
    fn start<'a>(&'a mut self, user: &'a str) -> RunFuture<'a, KeyboardInteractiveAuthResponse> {
        Box::pin(async move {
            self.authenticate_keyboard_interactive_start(user, None::<String>)
                .await
                .map_err(|_| "ssh_network_error")
        })
    }
    fn respond(&mut self, values: Vec<String>) -> RunFuture<'_, KeyboardInteractiveAuthResponse> {
        Box::pin(async move {
            self.authenticate_keyboard_interactive_respond(values)
                .await
                .map_err(|_| "ssh_network_error")
        })
    }
}
async fn password_authenticate(
    exchange: &mut impl PasswordExchange,
    user: &str,
    password: &Secret,
) -> Result<(), Error> {
    let result = exchange.password(user, password.expose()).await?;
    if result.success() {
        return Ok(());
    }
    if let russh::client::AuthResult::Failure {
        remaining_methods,
        partial_success,
    } = &result
    {
        if *partial_success {
            return Err("ssh_auth_interactive_required");
        }
        if !remaining_methods.contains(&russh::MethodKind::KeyboardInteractive) {
            return Err(failure(result));
        }
    }
    let mut response = exchange.start(user).await?;
    for _ in 0..8 {
        match response {
            KeyboardInteractiveAuthResponse::Success => return Ok(()),
            KeyboardInteractiveAuthResponse::Failure {
                remaining_methods,
                partial_success,
            } => {
                return Err(failure(russh::client::AuthResult::Failure {
                    remaining_methods,
                    partial_success,
                }));
            }
            KeyboardInteractiveAuthResponse::InfoRequest { prompts, .. } => {
                // 此切片只有密码，不猜测 OTP/MFA；只应答单个不回显挑战，空轮次允许继续。
                if prompts.len() > 1 || prompts.iter().any(|p| p.echo) {
                    return Err("ssh_auth_interactive_required");
                }
                let responses = prompts
                    .iter()
                    .map(|_| password.expose().to_owned())
                    .collect();
                response = exchange.respond(responses).await?;
            }
        }
    }
    Err("ssh_auth_interactive_required")
}

#[cfg(test)]
mod tests;
