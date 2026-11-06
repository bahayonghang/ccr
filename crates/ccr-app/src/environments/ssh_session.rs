//! Process-local reconnect and disconnect decisions.
//!
//! The decision does not scan keys, open a socket, or read an identity file.
//! A host counts as trusted only after this process records a confirmation.

use std::collections::BTreeSet;

pub const RECONNECT_UNTRUSTED: &str = "ssh_host_key_untrusted";
pub const DISCONNECT_NOTHING: &str = "ssh_not_connected";
pub const AUTH_UNSUPPORTED: &str = "ssh_auth_unsupported";
pub const CANCELLED_BEFORE_CONNECT: &str = "ssh_cancelled";
pub const HOST_KEY_MISMATCH: &str = "ssh_host_key_mismatch";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectKind {
    /// First connection. Cancel, password-only, and supplied key material refuse
    /// before a socket. Absent key material is not treated as acceptance.
    Open,
    /// Reconnect. Also requires process trust. A matching fingerprint is not enough.
    Again,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConnectFacts<'a> {
    pub cancel_requested: bool,
    pub password_only: bool,
    pub stored_fingerprint: Option<&'a str>,
    pub observed_fingerprint: Option<&'a str>,
    pub process_trusted: bool,
}

/// Refuse before any socket, key scan, or identity read.
///
/// `Ok` means this decision found no refusal. It does not open a connection.
pub fn boundary_before_connect(
    kind: ConnectKind,
    facts: ConnectFacts<'_>,
) -> Result<(), &'static str> {
    if facts.cancel_requested {
        return Err(CANCELLED_BEFORE_CONNECT);
    }
    if facts.password_only {
        return Err(AUTH_UNSUPPORTED);
    }
    let supplied = facts.stored_fingerprint.is_some() || facts.observed_fingerprint.is_some();
    if supplied {
        let decision = super::decide_host_key(facts.stored_fingerprint, facts.observed_fingerprint);
        if decision.may_store() || decision.may_trust() || decision.may_accept() {
            return Err(RECONNECT_UNTRUSTED);
        }
        match decision.status() {
            Some(super::HostKeyStatus::Mismatch) => return Err(HOST_KEY_MISMATCH),
            Some(super::HostKeyStatus::Matched)
                if facts.process_trusted && matches!(kind, ConnectKind::Again) => {}
            Some(super::HostKeyStatus::Matched | super::HostKeyStatus::New) | None => {
                return Err(RECONNECT_UNTRUSTED);
            }
        }
    } else if matches!(kind, ConnectKind::Again) && !facts.process_trusted {
        return Err(RECONNECT_UNTRUSTED);
    }
    Ok(())
}

pub fn password_only(password: Option<&str>) -> bool {
    password.is_some_and(|value| !value.trim().is_empty())
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcessHostTrust {
    hosts: BTreeSet<(String, u16)>,
}

fn remembered_host(host: &str) -> Option<String> {
    if host.is_empty()
        || host != host.trim()
        || host
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return None;
    }
    Some(host.to_ascii_lowercase())
}

impl ProcessHostTrust {
    pub fn note_confirmed(&mut self, host: &str, port: u16) {
        if port == 0 {
            return;
        }
        let Some(host) = remembered_host(host) else {
            return;
        };
        self.hosts.insert((host, port));
    }

    fn allows(&self, host: &str, port: u16) -> bool {
        let Some(host) = remembered_host(host) else {
            return false;
        };
        self.hosts.contains(&(host, port))
    }
}

/// Empty trust refuses before a target lookup. A non-empty set still has to
/// match a host and port already loaded in memory, before any socket.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconnectGate {
    Refused(&'static str),
    CheckTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisconnectReport {
    NothingConnected,
    RemoteDisconnected,
}

pub fn reconnect_gate(trust: &ProcessHostTrust) -> ReconnectGate {
    if trust.hosts.is_empty() {
        ReconnectGate::Refused(RECONNECT_UNTRUSTED)
    } else {
        ReconnectGate::CheckTarget
    }
}

pub fn allow_reconnect(
    trust: &ProcessHostTrust,
    host: &str,
    port: u16,
) -> Result<(), &'static str> {
    if trust.allows(host, port) {
        Ok(())
    } else {
        Err(RECONNECT_UNTRUSTED)
    }
}

/// `loaded` must already be in memory. `None` refuses; the caller does not
/// fetch a host row, scan a key, or open a socket to supply it.
pub fn allow_reconnect_target(
    trust: &ProcessHostTrust,
    loaded: Option<(&str, u16)>,
) -> Result<(), &'static str> {
    let Some((host, port)) = loaded else {
        return Err(RECONNECT_UNTRUSTED);
    };
    allow_reconnect(trust, host, port)
}

pub fn plan_disconnect(was_connected: bool) -> DisconnectReport {
    if was_connected {
        DisconnectReport::RemoteDisconnected
    } else {
        DisconnectReport::NothingConnected
    }
}

pub fn disconnect_error(report: DisconnectReport) -> Option<&'static str> {
    match report {
        DisconnectReport::NothingConnected => Some(DISCONNECT_NOTHING),
        DisconnectReport::RemoteDisconnected => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AUTH_UNSUPPORTED, CANCELLED_BEFORE_CONNECT, ConnectFacts, ConnectKind, DisconnectReport,
        HOST_KEY_MISMATCH, ProcessHostTrust, ReconnectGate, allow_reconnect,
        allow_reconnect_target, boundary_before_connect, disconnect_error, password_only,
        plan_disconnect, reconnect_gate,
    };

    #[test]
    fn empty_trust_refuses_reconnect_before_a_target() {
        let trust = ProcessHostTrust::default();
        assert_eq!(
            reconnect_gate(&trust),
            ReconnectGate::Refused("ssh_host_key_untrusted")
        );
        let mut blank = ProcessHostTrust::default();
        blank.note_confirmed("", 22);
        blank.note_confirmed("  ", 22);
        blank.note_confirmed("example.com", 0);
        assert_eq!(
            reconnect_gate(&blank),
            ReconnectGate::Refused("ssh_host_key_untrusted")
        );
        assert_eq!(
            allow_reconnect_target(&blank, Some(("example.com", 22))),
            Err("ssh_host_key_untrusted")
        );
        let rendered = format!("{trust:?}{blank:?}");
        assert!(!rendered.contains("ssh-keyscan"));
        assert!(!rendered.contains("id_rsa"));
        assert!(!rendered.contains("password"));
        assert!(!rendered.contains("SHA256"));
    }

    #[test]
    fn only_the_confirmed_host_and_port_may_pass_the_gate() {
        let mut trust = ProcessHostTrust::default();
        trust.note_confirmed("Example.COM", 22);
        assert_eq!(reconnect_gate(&trust), ReconnectGate::CheckTarget);
        assert_eq!(allow_reconnect(&trust, "example.com", 22), Ok(()));
        assert_eq!(allow_reconnect(&trust, "EXAMPLE.COM", 22), Ok(()));
        assert_eq!(
            allow_reconnect_target(&trust, None),
            Err("ssh_host_key_untrusted")
        );
        assert_eq!(
            allow_reconnect(&trust, "example.com", 2222),
            Err("ssh_host_key_untrusted")
        );
        assert_eq!(
            allow_reconnect(&trust, "other.example", 22),
            Err("ssh_host_key_untrusted")
        );
        let Err(hostile) = allow_reconnect(&trust, "$(id)", 22) else {
            panic!("untrusted host must be refused");
        };
        assert_eq!(hostile, "ssh_host_key_untrusted");
        assert!(!hostile.contains("$("));
        assert!(!hostile.contains("ssh-keyscan"));
        let rendered = format!("{trust:?}");
        assert!(!rendered.contains("SHA256"));
        assert!(!rendered.contains("id_rsa"));
        assert!(!rendered.contains("password"));
    }

    fn facts<'a>(
        cancel_requested: bool,
        password: Option<&'a str>,
        stored: Option<&'a str>,
        observed: Option<&'a str>,
        process_trusted: bool,
    ) -> ConnectFacts<'a> {
        ConnectFacts {
            cancel_requested,
            password_only: password_only(password),
            stored_fingerprint: stored,
            observed_fingerprint: observed,
            process_trusted,
        }
    }

    #[test]
    fn connect_boundary_refuses_before_a_socket() {
        let password = "secret-password";
        let identity = "C:/Keys/id_rsa";
        let host = "example.com";
        let fingerprint = "SHA256:example.com";
        let Err(cancelled) = boundary_before_connect(
            ConnectKind::Again,
            facts(
                true,
                Some(password),
                Some(fingerprint),
                Some(fingerprint),
                true,
            ),
        ) else {
            panic!("cancel must refuse before connect");
        };
        assert_eq!(cancelled, CANCELLED_BEFORE_CONNECT);
        assert!(!cancelled.contains(password));
        assert!(!cancelled.contains(identity));
        assert!(!cancelled.contains(host));
        assert!(!cancelled.contains("SHA256"));
        assert!(!cancelled.contains("ssh-keyscan"));

        let Err(auth) = boundary_before_connect(
            ConnectKind::Open,
            facts(false, Some(password), None, None, false),
        ) else {
            panic!("password authentication must be refused");
        };
        assert_eq!(auth, AUTH_UNSUPPORTED);
        assert!(!auth.contains(password));
        assert!(!auth.contains(identity));

        let Err(first) = boundary_before_connect(
            ConnectKind::Open,
            facts(false, None, None, Some(fingerprint), false),
        ) else {
            panic!("first trust must not accept a new key");
        };
        assert_eq!(first, "ssh_host_key_untrusted");
        assert!(!first.contains(fingerprint));
        assert!(!first.contains("ssh-keyscan"));

        let Err(changed) = boundary_before_connect(
            ConnectKind::Again,
            facts(false, None, Some("SHA256:old"), Some("SHA256:new"), true),
        ) else {
            panic!("a changed key must not be accepted");
        };
        assert_eq!(changed, HOST_KEY_MISMATCH);
        assert!(!changed.contains("SHA256"));
        assert!(!changed.contains(identity));

        let Err(matched) = boundary_before_connect(
            ConnectKind::Again,
            facts(false, None, Some(fingerprint), Some(fingerprint), false),
        ) else {
            panic!("a matching fingerprint is not process trust");
        };
        assert_eq!(matched, "ssh_host_key_untrusted");
        assert!(!matched.contains(fingerprint));

        let Err(reconnect) =
            boundary_before_connect(ConnectKind::Again, facts(false, None, None, None, false))
        else {
            panic!("reconnect without process trust must refuse");
        };
        assert_eq!(reconnect, "ssh_host_key_untrusted");

        let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
        let hosts = dir.path().join("known_hosts");
        std::fs::write(&hosts, b"original").unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(
            boundary_before_connect(
                ConnectKind::Again,
                facts(false, None, Some(fingerprint), Some(fingerprint), true),
            ),
            Ok(())
        );
        assert_eq!(
            boundary_before_connect(ConnectKind::Open, facts(false, None, None, None, false)),
            Ok(())
        );
        let body = std::fs::read_to_string(&hosts).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(body, "original");
        let rendered = format!("{cancelled}{auth}{first}{changed}{matched}{reconnect}");
        assert!(!rendered.contains(password));
        assert!(!rendered.contains(identity));
        assert!(!rendered.contains("ssh-keyscan"));
        assert!(!rendered.contains(" / "));
    }

    #[test]
    fn never_connected_disconnect_is_not_a_remote_success() {
        let nothing = plan_disconnect(false);
        assert_eq!(nothing, DisconnectReport::NothingConnected);
        assert_eq!(disconnect_error(nothing), Some("ssh_not_connected"));
        let remote = plan_disconnect(true);
        assert_eq!(remote, DisconnectReport::RemoteDisconnected);
        assert_eq!(disconnect_error(remote), None);
        assert_ne!(nothing, remote);
    }
}
