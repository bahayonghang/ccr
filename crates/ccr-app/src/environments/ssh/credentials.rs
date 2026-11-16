//! Host-scoped secrets. No secret implements serialization or unmasked formatting.
use super::Error;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Secret(zeroize::Zeroizing<String>);
impl Secret {
    pub fn new(value: String) -> Self {
        Self(zeroize::Zeroizing::new(value))
    }
    pub(super) fn expose(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}
impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AuthMethod {
    #[default]
    Agent,
    PrivateKey,
    Password,
}
impl AuthMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::PrivateKey => "private_key",
            Self::Password => "password",
        }
    }
    pub fn parse(value: &str) -> Result<Self, Error> {
        match value {
            "agent" => Ok(Self::Agent),
            "private_key" => Ok(Self::PrivateKey),
            "password" => Ok(Self::Password),
            _ => Err("ssh_invalid_target"),
        }
    }
}
#[derive(Clone, Default)]
pub(super) struct Credentials(pub Option<Arc<keyring_core::CredentialStore>>);
impl Credentials {
    async fn run<T: Send + 'static>(
        &self,
        id: &str,
        work: impl FnOnce(keyring_core::Entry) -> Result<T, Error> + Send + 'static,
    ) -> Result<T, Error> {
        let store = self.0.clone();
        let account = format!("{id}:ssh-secret");
        tokio::task::spawn_blocking(move || {
            let entry = match store {
                Some(store) => store.build("CCR SSH", &account, None),
                None => keyring::Entry::new("CCR SSH", &account).map(|entry| entry.inner),
            }
            .map_err(|_| "ssh_credential_store_unavailable")?;
            work(entry)
        })
        .await
        .map_err(|_| "ssh_credential_store_unavailable")?
    }
    pub async fn get(&self, id: &str) -> Result<Option<Secret>, Error> {
        self.run(id, |entry| match entry.get_password() {
            Ok(value) => Ok(Some(Secret::new(value))),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(_) => Err("ssh_credential_store_unavailable"),
        })
        .await
    }
    pub async fn set(&self, id: &str, value: Option<Secret>) -> Result<(), Error> {
        self.run(id, move |entry| match value {
            Some(value) => entry
                .set_password(value.expose())
                .map_err(|_| "ssh_credential_store_unavailable"),
            None => match entry.delete_credential() {
                Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
                Err(_) => Err("ssh_credential_store_unavailable"),
            },
        })
        .await
    }
}
#[derive(Clone)]
pub(super) struct Authentication {
    pub method: AuthMethod,
    pub secret: Option<Secret>,
}
