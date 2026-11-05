//! Process-local reconnect and disconnect decisions.
//!
//! The decision does not scan keys, open a socket, or read an identity file.
//! A host counts as trusted only after this process records a confirmation.

use std::collections::BTreeSet;

pub const RECONNECT_UNTRUSTED: &str = "ssh_host_key_untrusted";
pub const DISCONNECT_NOTHING: &str = "ssh_not_connected";

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
        DisconnectReport, ProcessHostTrust, ReconnectGate, allow_reconnect, allow_reconnect_target,
        disconnect_error, plan_disconnect, reconnect_gate,
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
