#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use russh::{
    MethodKind,
    client::{AuthResult, Prompt},
};
use std::collections::VecDeque;

struct Exchange {
    password_result: AuthResult,
    replies: VecDeque<KeyboardInteractiveAuthResponse>,
    responses: Vec<Vec<String>>,
    starts: usize,
}
impl PasswordExchange for Exchange {
    fn password<'a>(&'a mut self, user: &'a str, secret: &'a str) -> RunFuture<'a, AuthResult> {
        Box::pin(async move {
            assert_eq!(user, "fixture");
            assert_eq!(secret, "synthetic-password");
            Ok(self.password_result.clone())
        })
    }
    fn start<'a>(&'a mut self, _: &'a str) -> RunFuture<'a, KeyboardInteractiveAuthResponse> {
        Box::pin(async move {
            self.starts += 1;
            Ok(self.replies.pop_front().expect("scripted start reply"))
        })
    }
    fn respond(&mut self, values: Vec<String>) -> RunFuture<'_, KeyboardInteractiveAuthResponse> {
        Box::pin(async move {
            self.responses.push(values);
            Ok(self.replies.pop_front().expect("scripted response reply"))
        })
    }
}
fn rejected(methods: &[MethodKind]) -> AuthResult {
    AuthResult::Failure {
        remaining_methods: methods.into(),
        partial_success: false,
    }
}
fn challenge(echo: bool) -> KeyboardInteractiveAuthResponse {
    KeyboardInteractiveAuthResponse::InfoRequest {
        name: "Synthetic login".into(),
        instructions: String::new(),
        prompts: vec![Prompt {
            prompt: "Password:".into(),
            echo,
        }],
    }
}
fn exchange(result: AuthResult, replies: Vec<KeyboardInteractiveAuthResponse>) -> Exchange {
    Exchange {
        password_result: result,
        replies: replies.into(),
        responses: Vec::new(),
        starts: 0,
    }
}
#[tokio::test]
async fn password_success_and_keyboard_info_request_use_the_real_authentication_state_machine() {
    let password = Secret::new("synthetic-password".into());
    let mut direct = exchange(AuthResult::Success, Vec::new());
    password_authenticate(&mut direct, "fixture", &password)
        .await
        .unwrap();
    assert_eq!(direct.starts, 0);
    let empty = KeyboardInteractiveAuthResponse::InfoRequest {
        name: String::new(),
        instructions: String::new(),
        prompts: Vec::new(),
    };
    let mut interactive = exchange(
        rejected(&[MethodKind::KeyboardInteractive]),
        vec![
            empty,
            challenge(false),
            KeyboardInteractiveAuthResponse::Success,
        ],
    );
    password_authenticate(&mut interactive, "fixture", &password)
        .await
        .unwrap();
    assert_eq!(interactive.starts, 1);
    assert_eq!(
        interactive.responses,
        vec![Vec::<String>::new(), vec!["synthetic-password".into()]]
    );
}
#[tokio::test]
async fn rejected_methods_and_unsafe_interactive_prompts_never_receive_a_password() {
    let password = Secret::new("synthetic-password".into());
    let mut unsupported = exchange(
        rejected(&[MethodKind::PublicKey, MethodKind::Password]),
        Vec::new(),
    );
    assert_eq!(
        password_authenticate(&mut unsupported, "fixture", &password).await,
        Err("ssh_auth_failed_key_password")
    );
    assert_eq!(unsupported.starts, 0);
    for response in [
        challenge(true),
        KeyboardInteractiveAuthResponse::InfoRequest {
            name: String::new(),
            instructions: String::new(),
            prompts: vec![
                Prompt {
                    prompt: "Password".into(),
                    echo: false,
                },
                Prompt {
                    prompt: "OTP".into(),
                    echo: false,
                },
            ],
        },
    ] {
        let mut interactive =
            exchange(rejected(&[MethodKind::KeyboardInteractive]), vec![response]);
        assert_eq!(
            password_authenticate(&mut interactive, "fixture", &password).await,
            Err("ssh_auth_interactive_required")
        );
        assert!(interactive.responses.is_empty());
    }
    let mut failure = exchange(
        rejected(&[MethodKind::KeyboardInteractive]),
        vec![KeyboardInteractiveAuthResponse::Failure {
            remaining_methods: [MethodKind::PublicKey].as_slice().into(),
            partial_success: false,
        }],
    );
    assert_eq!(
        password_authenticate(&mut failure, "fixture", &password).await,
        Err("ssh_auth_failed_key")
    );
}
#[test]
fn server_method_combinations_have_distinct_bilingual_messages() {
    for (bits, suffix) in [
        (0, ""),
        (1, "_key"),
        (2, "_password"),
        (3, "_key_password"),
        (4, "_keyboard"),
        (5, "_key_keyboard"),
        (6, "_password_keyboard"),
        (7, "_all"),
    ] {
        let methods = [
            (1, MethodKind::PublicKey),
            (2, MethodKind::Password),
            (4, MethodKind::KeyboardInteractive),
        ]
        .into_iter()
        .filter_map(|(bit, kind)| (bits & bit != 0).then_some(kind))
        .collect::<Vec<_>>();
        let code = failure(rejected(&methods));
        assert_eq!(code, format!("ssh_auth_failed{suffix}"));
        for chinese in [false, true] {
            assert!(authentication_message(code, chinese).is_some());
        }
    }
}
#[tokio::test]
async fn private_key_loader_handles_openssh_and_pkcs8_with_and_without_passphrases() {
    use russh::keys::ssh_key::{
        Cipher, Kdf, LineEnding, PrivateKey,
        private::{Ed25519Keypair, KeypairData},
    };
    let key = PrivateKey::new(
        KeypairData::Ed25519(Ed25519Keypair::from_seed(&[7; 32])),
        "synthetic test key",
    )
    .unwrap();
    let mut formats = vec![(
        key.to_openssh(LineEnding::LF)
            .unwrap()
            .to_string()
            .into_bytes(),
        false,
    )];
    let encrypted = key
        .encrypt_with(
            Cipher::Aes256Ctr,
            Kdf::Bcrypt {
                salt: vec![3; 16],
                rounds: 1,
            },
            42,
            b"synthetic-passphrase",
        )
        .unwrap();
    formats.push((
        encrypted
            .to_openssh(LineEnding::LF)
            .unwrap()
            .to_string()
            .into_bytes(),
        true,
    ));
    let mut plain = Vec::new();
    russh::keys::encode_pkcs8_pem(&key, &mut plain).unwrap();
    formats.push((plain, false));
    let mut encrypted = Vec::new();
    russh::keys::encode_pkcs8_pem_encrypted(&key, b"synthetic-passphrase", 1, &mut encrypted)
        .unwrap();
    formats.push((encrypted, true));
    let root = tempfile::tempdir().unwrap();
    for (index, (content, encrypted)) in formats.into_iter().enumerate() {
        let path = root.path().join(format!("synthetic-{index}.pem"));
        std::fs::write(&path, content).unwrap();
        let path = path.to_string_lossy().into_owned();
        let without = load_private_key(path.clone(), None).await;
        if encrypted {
            assert!(matches!(without, Err("ssh_credentials_missing")));
        } else {
            assert_eq!(
                without.unwrap().public_key().key_data(),
                key.public_key().key_data()
            );
        }
        let loaded = load_private_key(
            path.clone(),
            Some(Secret::new("synthetic-passphrase".into())),
        )
        .await
        .unwrap();
        assert_eq!(loaded.public_key().key_data(), key.public_key().key_data());
        if encrypted {
            assert!(matches!(
                load_private_key(path, Some(Secret::new("wrong".into()))).await,
                Err("ssh_private_key_invalid")
            ));
        }
    }
    assert!(matches!(
        load_private_key(
            root.path().join("missing").to_string_lossy().into_owned(),
            None
        )
        .await,
        Err("ssh_private_key_invalid")
    ));
}
