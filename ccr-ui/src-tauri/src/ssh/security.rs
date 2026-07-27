//! SSH input validation, trust state, and OpenSSH process construction.

use std::collections::HashMap;
use std::ffi::OsString;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use ccr_core::core::{
    VersionedWriteOutcome, WriteOptions, content_version_token, write_guarded_versioned,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::process::{ProcessDescriptor, ProcessGateway};

pub const HOST_KEY_CHALLENGE_TTL: Duration = Duration::from_secs(120);
const MAX_REMOTE_PATH_LEN: usize = 1024;
const KNOWN_HOSTS_CAS_ATTEMPTS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTarget {
    host: String,
    port: u16,
    user: Option<String>,
    identity_file: Option<PathBuf>,
}

impl SshTarget {
    pub fn new(
        host: &str,
        port: u16,
        user: Option<&str>,
        identity_file: Option<&str>,
    ) -> Result<Self, String> {
        let host = validate_host(host)?;
        if port == 0 {
            return Err("ssh_invalid_port: port must be greater than zero".to_string());
        }

        let user = user.map(validate_user).transpose()?;
        let identity_file = identity_file.map(validate_identity_file).transpose()?;

        Ok(Self {
            host,
            port,
            user,
            identity_file,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub fn destination(&self) -> String {
        match &self.user {
            Some(user) => format!("{user}@{}", self.host),
            None => self.host.clone(),
        }
    }

    pub fn ssh_command(
        &self,
        known_hosts_path: &Path,
        connect_timeout_secs: u64,
    ) -> Result<tokio::process::Command, String> {
        let mut command = ProcessGateway::command(&ProcessDescriptor::openssh())?;
        configure_common_options(&mut command, known_hosts_path, connect_timeout_secs);
        command.arg("-p").arg(self.port.to_string());
        if let Some(identity_file) = &self.identity_file {
            command.arg("-i").arg(identity_file);
        }
        command.arg(self.destination());
        Ok(command)
    }

    pub fn sftp_command(&self, known_hosts_path: &Path) -> Result<tokio::process::Command, String> {
        let mut command = ProcessGateway::command(&ProcessDescriptor::sftp())?;
        configure_common_options(&mut command, known_hosts_path, 10);
        command.arg("-P").arg(self.port.to_string());
        if let Some(identity_file) = &self.identity_file {
            command.arg("-i").arg(identity_file);
        }
        command.arg("-b").arg("-").arg(self.destination());
        Ok(command)
    }
}

fn configure_common_options(
    command: &mut tokio::process::Command,
    known_hosts_path: &Path,
    connect_timeout_secs: u64,
) {
    let mut known_hosts_option = OsString::from("UserKnownHostsFile=");
    known_hosts_option.push(known_hosts_path.as_os_str());

    command
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg(format!("ConnectTimeout={connect_timeout_secs}"))
        .arg("-o")
        .arg("StrictHostKeyChecking=yes")
        .arg("-o")
        .arg("GlobalKnownHostsFile=none")
        .arg("-o")
        .arg(known_hosts_option);
}

fn validate_host(value: &str) -> Result<String, String> {
    if value.is_empty()
        || value != value.trim()
        || value.starts_with('-')
        || value.starts_with('[')
        || value.ends_with(']')
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err("ssh_invalid_host: invalid host boundary".to_string());
    }

    if value.parse::<IpAddr>().is_ok() {
        return Ok(value.to_ascii_lowercase());
    }

    if value.len() > 253 || value.ends_with('.') {
        return Err("ssh_invalid_host: invalid DNS name".to_string());
    }

    for label in value.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
        {
            return Err("ssh_invalid_host: invalid DNS name".to_string());
        }
    }

    Ok(value.to_ascii_lowercase())
}

fn validate_user(value: &str) -> Result<String, String> {
    if value.is_empty()
        || value != value.trim()
        || value.starts_with('-')
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
    {
        return Err("ssh_invalid_user: invalid user boundary".to_string());
    }
    Ok(value.to_string())
}

fn validate_identity_file(value: &str) -> Result<PathBuf, String> {
    if value.is_empty()
        || value != value.trim()
        || value.starts_with('-')
        || value.chars().any(char::is_control)
    {
        return Err("ssh_invalid_identity_file: invalid identity path boundary".to_string());
    }
    Ok(PathBuf::from(value))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePosixPath(String);

impl RemotePosixPath {
    pub fn root(value: Option<&str>) -> Result<Self, String> {
        let value = value.unwrap_or("~");
        if value.len() > MAX_REMOTE_PATH_LEN
            || value != value.trim()
            || value.chars().any(char::is_control)
            || value.contains('\\')
        {
            return Err("ssh_invalid_remote_path: invalid remote path boundary".to_string());
        }

        if matches!(value, "~" | "/") {
            return Ok(Self(value.to_string()));
        }
        if !value.starts_with('/') || (value.len() > 1 && value.ends_with('/')) {
            return Err(
                "ssh_invalid_remote_path: expected ~ or an absolute POSIX path".to_string(),
            );
        }

        validate_posix_segments(value.trim_start_matches('/'))?;
        Ok(Self(value.to_string()))
    }

    pub fn join_relative(&self, value: &str) -> Result<Self, String> {
        if value.is_empty()
            || value.len() > MAX_REMOTE_PATH_LEN
            || value.starts_with('/')
            || value.ends_with('/')
            || value.contains('\\')
            || value.chars().any(char::is_control)
        {
            return Err("ssh_invalid_remote_path: invalid relative path".to_string());
        }
        validate_posix_segments(value)?;

        let joined = if self.0 == "/" {
            format!("/{value}")
        } else {
            format!("{}/{value}", self.0)
        };
        if joined.len() > MAX_REMOTE_PATH_LEN {
            return Err("ssh_invalid_remote_path: remote path is too long".to_string());
        }
        Ok(Self(joined))
    }

    pub fn with_suffix(&self, suffix: &str) -> Result<Self, String> {
        if suffix.is_empty()
            || !suffix.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
            })
        {
            return Err("ssh_invalid_remote_path: invalid path suffix".to_string());
        }
        let path = format!("{}{suffix}", self.0);
        if path.len() > MAX_REMOTE_PATH_LEN {
            return Err("ssh_invalid_remote_path: remote path is too long".to_string());
        }
        Ok(Self(path))
    }

    pub fn parent_directories(&self) -> Vec<String> {
        let Some((parent, _)) = self.0.rsplit_once('/') else {
            return Vec::new();
        };
        if parent.is_empty() || parent == "~" {
            return Vec::new();
        }

        let mut directories = Vec::new();
        if let Some(relative) = parent.strip_prefix("~/") {
            let mut current = "~".to_string();
            for segment in relative.split('/') {
                current.push('/');
                current.push_str(segment);
                directories.push(current.clone());
            }
            return directories;
        }

        let mut current = String::new();
        for segment in parent.trim_start_matches('/').split('/') {
            if segment.is_empty() {
                continue;
            }
            current.push('/');
            current.push_str(segment);
            directories.push(current.clone());
        }
        directories
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_posix_segments(value: &str) -> Result<(), String> {
    for segment in value.split('/') {
        if segment.is_empty()
            || matches!(segment, "." | "..")
            || !segment.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
            })
        {
            return Err("ssh_invalid_remote_path: invalid POSIX path segment".to_string());
        }
    }
    Ok(())
}

pub fn posix_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub fn sftp_quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub fn sftp_read_batch(remote_path: &RemotePosixPath, local_path: &Path) -> String {
    let local = local_path.to_string_lossy().replace('\\', "/");
    format!(
        "get {} {}\n",
        sftp_quote(remote_path.as_str()),
        sftp_quote(&local)
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SftpWritePlan {
    pub upload_batch: String,
    pub rename_batch: String,
    pub cleanup_batch: String,
    pub remote_temp_path: RemotePosixPath,
}

pub fn sftp_write_plan(
    remote_path: &RemotePosixPath,
    local_path: &Path,
    nonce: Uuid,
) -> Result<SftpWritePlan, String> {
    let remote_temp_path = remote_path.with_suffix(&format!(".ccr-tmp-{nonce}"))?;
    let local = local_path.to_string_lossy().replace('\\', "/");
    let mut upload_batch = String::new();
    for directory in remote_path.parent_directories() {
        upload_batch.push_str(&format!("-mkdir {}\n", sftp_quote(&directory)));
    }
    upload_batch.push_str(&format!(
        "put {} {}\n",
        sftp_quote(&local),
        sftp_quote(remote_temp_path.as_str()),
    ));
    let rename_batch = format!(
        "rename {} {}\n",
        sftp_quote(remote_temp_path.as_str()),
        sftp_quote(remote_path.as_str()),
    );
    let cleanup_batch = format!("-rm {}\n", sftp_quote(remote_temp_path.as_str()));

    Ok(SftpWritePlan {
        upload_batch,
        rename_batch,
        cleanup_batch,
        remote_temp_path,
    })
}

pub async fn run_openssh_command(
    command: tokio::process::Command,
    descriptor: &ProcessDescriptor,
    stdin: Option<&[u8]>,
) -> Result<Output, String> {
    let output =
        ProcessGateway::execute_command(command, descriptor, stdin.map(<[u8]>::to_vec)).await?;
    if output.timed_out {
        return Err("ssh_process_timeout: OpenSSH command timed out".to_string());
    }
    if output.stdout_truncated || output.stderr_truncated {
        return Err("ssh_process_output_limit: OpenSSH output exceeded the limit".to_string());
    }
    Ok(Output {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

pub fn app_known_hosts_path() -> Result<PathBuf, String> {
    let root = match std::env::var_os("CCR_ROOT") {
        Some(root) => PathBuf::from(root),
        None => dirs::home_dir()
            .ok_or_else(|| "ssh_known_hosts_path: home directory is unavailable".to_string())?
            .join(".ccr"),
    };
    Ok(root.join("ssh").join("known_hosts"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostKeyStatus {
    New,
    Matched,
    Mismatch,
}

pub fn classify_host_key(
    stored_fingerprint: Option<&str>,
    observed_fingerprint: &str,
) -> HostKeyStatus {
    match stored_fingerprint {
        None => HostKeyStatus::New,
        Some(stored) if stored == observed_fingerprint => HostKeyStatus::Matched,
        Some(_) => HostKeyStatus::Mismatch,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedHostKey {
    pub key_type: String,
    pub key_data: String,
    pub fingerprint: String,
}

pub fn parse_keyscan_output(output: &str, host: &str, port: u16) -> Result<ScannedHostKey, String> {
    let expected_host = if port == 22 {
        host.to_string()
    } else {
        format!("[{host}]:{port}")
    };
    let bracketed_host = format!("[{host}]:{port}");

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let scanned_host = fields.next().unwrap_or_default();
        let key_type = fields.next().unwrap_or_default();
        let key_data = fields.next().unwrap_or_default();
        if fields.next().is_some()
            || (scanned_host != expected_host && scanned_host != bracketed_host)
            || !matches!(
                key_type,
                "ssh-ed25519"
                    | "ssh-rsa"
                    | "ecdsa-sha2-nistp256"
                    | "ecdsa-sha2-nistp384"
                    | "ecdsa-sha2-nistp521"
            )
        {
            continue;
        }

        let decoded = STANDARD
            .decode(key_data)
            .map_err(|_| "ssh_host_key_invalid: key data is not valid base64".to_string())?;
        if decoded.is_empty() {
            return Err("ssh_host_key_invalid: key data is empty".to_string());
        }
        let fingerprint = format!("SHA256:{}", STANDARD_NO_PAD.encode(Sha256::digest(decoded)));
        return Ok(ScannedHostKey {
            key_type: key_type.to_string(),
            key_data: key_data.to_string(),
            fingerprint,
        });
    }

    Err("ssh_host_key_missing: no valid key for requested host".to_string())
}

#[derive(Debug, Clone)]
pub struct HostKeyChallenge {
    pub host: String,
    pub port: u16,
    pub key_type: String,
    pub key_data: String,
    pub fingerprint: String,
    created_at: Instant,
}

#[derive(Debug, Clone, Copy)]
enum ChallengeTombstone {
    Reused(Instant),
    Expired(Instant),
}

#[derive(Debug, Default)]
struct ChallengeStore {
    live: HashMap<Uuid, HostKeyChallenge>,
    tombstones: HashMap<Uuid, ChallengeTombstone>,
}

#[derive(Debug, Default)]
pub struct SshTrustService {
    store: Mutex<ChallengeStore>,
}

impl SshTrustService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, host: String, port: u16, key: ScannedHostKey) -> String {
        self.register_at(host, port, key, Instant::now())
    }

    fn register_at(&self, host: String, port: u16, key: ScannedHostKey, now: Instant) -> String {
        let id = Uuid::new_v4();
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        prune_challenges(&mut store, now);
        store.live.insert(
            id,
            HostKeyChallenge {
                host,
                port,
                key_type: key.key_type,
                key_data: key.key_data,
                fingerprint: key.fingerprint,
                created_at: now,
            },
        );
        id.to_string()
    }

    pub fn consume(&self, challenge_id: &str) -> Result<HostKeyChallenge, String> {
        self.consume_at(challenge_id, Instant::now())
    }

    fn consume_at(&self, challenge_id: &str, now: Instant) -> Result<HostKeyChallenge, String> {
        let id = Uuid::parse_str(challenge_id)
            .map_err(|_| "ssh_challenge_invalid: malformed challenge id".to_string())?;
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(tombstone) = store.tombstones.get(&id) {
            return Err(match tombstone {
                ChallengeTombstone::Reused(_) => {
                    "ssh_challenge_reused: challenge was already consumed".to_string()
                }
                ChallengeTombstone::Expired(_) => {
                    "ssh_challenge_expired: challenge has expired".to_string()
                }
            });
        }

        let challenge = store
            .live
            .remove(&id)
            .ok_or_else(|| "ssh_challenge_unknown: challenge was not issued".to_string())?;
        if now.duration_since(challenge.created_at) > HOST_KEY_CHALLENGE_TTL {
            store
                .tombstones
                .insert(id, ChallengeTombstone::Expired(now));
            return Err("ssh_challenge_expired: challenge has expired".to_string());
        }
        store.tombstones.insert(id, ChallengeTombstone::Reused(now));
        prune_challenges(&mut store, now);
        Ok(challenge)
    }
}

fn prune_challenges(store: &mut ChallengeStore, now: Instant) {
    let expired: Vec<Uuid> = store
        .live
        .iter()
        .filter_map(|(id, challenge)| {
            (now.duration_since(challenge.created_at) > HOST_KEY_CHALLENGE_TTL).then_some(*id)
        })
        .collect();
    for id in expired {
        store.live.remove(&id);
        store
            .tombstones
            .insert(id, ChallengeTombstone::Expired(now));
    }
    store.tombstones.retain(|_, tombstone| {
        let created_at = match tombstone {
            ChallengeTombstone::Reused(created_at) | ChallengeTombstone::Expired(created_at) => {
                *created_at
            }
        };
        now.duration_since(created_at) <= HOST_KEY_CHALLENGE_TTL
    });
}

pub async fn persist_known_host(
    path: PathBuf,
    host: String,
    port: u16,
    key_type: String,
    key_data: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        persist_known_host_sync(&path, &host, port, &key_type, &key_data)
    })
    .await
    .map_err(|error| format!("ssh_known_hosts_join_failed: {error}"))?
}

fn persist_known_host_sync(
    path: &Path,
    host: &str,
    port: u16,
    key_type: &str,
    key_data: &str,
) -> Result<(), String> {
    let marker = known_host_marker(host, port);
    let replacement = format!("{marker} {key_type} {key_data}");

    for _ in 0..KNOWN_HOSTS_CAS_ATTEMPTS {
        let (current, expected_token) = match std::fs::read(path) {
            Ok(bytes) => {
                let token = content_version_token(&bytes);
                let content = String::from_utf8(bytes)
                    .map_err(|_| "ssh_known_hosts_invalid_utf8".to_string())?;
                (content, token)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (String::new(), String::new())
            }
            Err(error) => return Err(format!("ssh_known_hosts_read_failed: {error}")),
        };

        let mut lines: Vec<&str> = current
            .lines()
            .filter(|line| line.split_whitespace().next() != Some(marker.as_str()))
            .collect();
        lines.push(&replacement);
        let next = format!("{}\n", lines.join("\n"));
        let options = WriteOptions {
            secret: true,
            ..WriteOptions::default()
        };

        match write_guarded_versioned(path, next.as_bytes(), &expected_token, &options)
            .map_err(|error| format!("ssh_known_hosts_write_failed: {error}"))?
        {
            VersionedWriteOutcome::Written => return Ok(()),
            VersionedWriteOutcome::Conflict => continue,
        }
    }

    Err("ssh_known_hosts_conflict: file changed repeatedly".to_string())
}

fn known_host_marker(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_string()
    } else {
        format!("[{host}]:{port}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SshFailureKind {
    HostKeyMismatch,
    HostKeyUntrusted,
    AuthenticationUnsupported,
    Network,
}

impl SshFailureKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::HostKeyMismatch => "ssh_host_key_mismatch",
            Self::HostKeyUntrusted => "ssh_host_key_untrusted",
            Self::AuthenticationUnsupported => "ssh_auth_unsupported",
            Self::Network => "ssh_network_error",
        }
    }
}

pub fn classify_ssh_failure(stderr: &str) -> SshFailureKind {
    let stderr = stderr.to_ascii_lowercase();
    if stderr.contains("remote host identification has changed")
        || stderr.contains("host key for") && stderr.contains("has changed")
        || stderr.contains("offending") && stderr.contains("host key")
    {
        SshFailureKind::HostKeyMismatch
    } else if stderr.contains("host key verification failed")
        || stderr.contains("host key is known")
        || stderr.contains("strict checking")
    {
        SshFailureKind::HostKeyUntrusted
    } else if stderr.contains("permission denied") || stderr.contains("no supported authentication")
    {
        SshFailureKind::AuthenticationUnsupported
    } else {
        SshFailureKind::Network
    }
}

pub fn openssh_error(stderr: &str) -> String {
    let kind = classify_ssh_failure(stderr);
    let detail = stderr.trim();
    if detail.is_empty() {
        kind.code().to_string()
    } else {
        format!("{}: {detail}", kind.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_key() -> ScannedHostKey {
        let key_data = STANDARD.encode(b"test-host-key");
        let output = format!("example.com ssh-ed25519 {key_data}\n");
        parse_keyscan_output(&output, "example.com", 22).expect("sample key should parse")
    }

    #[test]
    fn validates_host_user_identity_and_option_boundaries() {
        for host in [
            "",
            " host",
            "host ",
            "--option",
            "host name",
            "[::1]",
            "bad_host",
        ] {
            assert!(SshTarget::new(host, 22, None, None).is_err(), "{host:?}");
        }
        for user in ["", " user", "--option", "user@host", "line\nbreak"] {
            assert!(
                SshTarget::new("example.com", 22, Some(user), None).is_err(),
                "{user:?}"
            );
        }
        for identity in ["", " key", "--option", "line\nbreak"] {
            assert!(
                SshTarget::new("example.com", 22, None, Some(identity)).is_err(),
                "{identity:?}"
            );
        }

        assert!(
            SshTarget::new(
                "example.com",
                22,
                Some("deploy-user"),
                Some("C:/Keys/id key")
            )
            .is_ok()
        );
        assert!(SshTarget::new("2001:db8::1", 2222, None, None).is_ok());
    }

    #[test]
    fn openssh_commands_always_use_the_app_trust_store() {
        let target = SshTarget::new("example.com", 2222, Some("deploy"), None).unwrap();
        let command = target
            .ssh_command(Path::new("C:/CCR Data/known_hosts"), 10)
            .expect("trusted SSH command");
        let args: Vec<String> = command
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert!(args.contains(&"BatchMode=yes".to_string()));
        assert!(args.contains(&"StrictHostKeyChecking=yes".to_string()));
        assert!(args.contains(&"GlobalKnownHostsFile=none".to_string()));
        assert!(args.contains(&"UserKnownHostsFile=C:/CCR Data/known_hosts".to_string()));
        assert!(!args.iter().any(|arg| arg.contains("accept-new")));
        assert_eq!(args.last().map(String::as_str), Some("deploy@example.com"));
    }

    #[test]
    fn rejects_hostile_remote_home_corpus() {
        for path in [
            "$(id)",
            "`id`",
            "\\$(id)",
            "\\\\$(id)",
            "\";id;\"",
            "' ; id ; '",
            "line\nbreak",
            "--option",
            "/home/$USER",
            "/home/../root",
            "/home//user",
        ] {
            assert!(RemotePosixPath::root(Some(path)).is_err(), "{path:?}");
        }
        assert_eq!(RemotePosixPath::root(Some("~")).unwrap().as_str(), "~");
        assert_eq!(RemotePosixPath::root(Some("/")).unwrap().as_str(), "/");
        assert_eq!(
            RemotePosixPath::root(Some("/home/user")).unwrap().as_str(),
            "/home/user"
        );
    }

    #[test]
    fn remote_path_property_is_deterministic_and_non_interpolating() {
        for byte in 0_u8..=127 {
            let character = char::from(byte);
            let candidate = format!("/safe/a{character}b");
            let first = RemotePosixPath::root(Some(&candidate));
            let second = RemotePosixPath::root(Some(&candidate));
            assert_eq!(first.is_ok(), second.is_ok(), "byte {byte}");
            if let Ok(path) = first {
                assert!(path.as_str().chars().all(|value| {
                    value.is_ascii_alphanumeric() || matches!(value, '/' | '.' | '_' | '-')
                }));
                assert!(!path.as_str().contains(['$', '`', '\\', '\'', '"']));
            }
        }
    }

    #[test]
    fn posix_encoder_handles_single_quotes() {
        assert_eq!(posix_single_quote("a'b"), "'a'\"'\"'b'");
    }

    #[test]
    fn sftp_write_is_staged_and_uses_no_remote_shell() {
        let remote = RemotePosixPath::root(Some("/home/user"))
            .unwrap()
            .join_relative(".claude/settings.json")
            .unwrap();
        let nonce = Uuid::parse_str("00000000-0000-4000-8000-000000000001").unwrap();
        let plan = sftp_write_plan(&remote, Path::new("C:/Temp/config file"), nonce).unwrap();

        assert!(plan.upload_batch.contains("put \"C:/Temp/config file\""));
        assert!(plan.rename_batch.contains("rename "));
        assert!(!plan.upload_batch.contains("rename "));
        assert!(!plan.upload_batch.contains("cat >"));
        assert!(!plan.upload_batch.contains("mkdir -p"));
        assert!(plan.cleanup_batch.starts_with("-rm "));
    }

    #[test]
    fn host_key_state_has_new_match_and_mismatch_outcomes() {
        assert_eq!(classify_host_key(None, "SHA256:new"), HostKeyStatus::New);
        assert_eq!(
            classify_host_key(Some("SHA256:same"), "SHA256:same"),
            HostKeyStatus::Matched
        );
        assert_eq!(
            classify_host_key(Some("SHA256:old"), "SHA256:new"),
            HostKeyStatus::Mismatch
        );
    }

    #[test]
    fn challenge_is_single_use_and_expiry_is_stable() {
        let service = SshTrustService::new();
        let now = Instant::now();
        let challenge_id = service.register_at("example.com".to_string(), 22, sample_key(), now);
        assert!(service.consume_at(&challenge_id, now).is_ok());
        assert!(
            service
                .consume_at(&challenge_id, now)
                .unwrap_err()
                .starts_with("ssh_challenge_reused")
        );

        let expired_id = service.register_at("example.com".to_string(), 22, sample_key(), now);
        let after_ttl = now + HOST_KEY_CHALLENGE_TTL + Duration::from_secs(1);
        assert!(
            service
                .consume_at(&expired_id, after_ttl)
                .unwrap_err()
                .starts_with("ssh_challenge_expired")
        );
        assert!(
            service
                .consume_at(&expired_id, after_ttl)
                .unwrap_err()
                .starts_with("ssh_challenge_expired")
        );

        let pruned_id = service.register_at("example.com".to_string(), 22, sample_key(), now);
        service.register_at("other.example".to_string(), 22, sample_key(), after_ttl);
        assert!(
            service
                .consume_at(&pruned_id, after_ttl)
                .unwrap_err()
                .starts_with("ssh_challenge_expired")
        );
    }

    #[test]
    fn known_hosts_update_replaces_only_the_target() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("ssh/known_hosts");
        persist_known_host_sync(&path, "one.example", 22, "ssh-ed25519", "AAAA").unwrap();
        persist_known_host_sync(&path, "two.example", 2222, "ssh-rsa", "BBBB").unwrap();
        persist_known_host_sync(&path, "one.example", 22, "ssh-ed25519", "CCCC").unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content.matches("one.example ").count(), 1);
        assert!(content.contains("one.example ssh-ed25519 CCCC"));
        assert!(content.contains("[two.example]:2222 ssh-rsa BBBB"));
    }

    #[test]
    fn mismatch_is_a_distinct_blocking_security_failure() {
        assert_eq!(
            classify_ssh_failure("WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!"),
            SshFailureKind::HostKeyMismatch
        );
        assert_eq!(
            classify_ssh_failure("Host key verification failed."),
            SshFailureKind::HostKeyUntrusted
        );
        assert_eq!(
            classify_ssh_failure("Permission denied (publickey)."),
            SshFailureKind::AuthenticationUnsupported
        );
    }
}
