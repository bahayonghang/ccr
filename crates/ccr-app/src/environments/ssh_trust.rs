//! Host-key comparison shared with the Tauri SSH trust service.
//!
//! The comparison is fingerprint text only. It does not scan keys, open a
//! socket, read a password, or read an identity file.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
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

/// What this desktop shell may do with a host-key comparison.
///
/// The booleans stay false. A match records that the fingerprint strings are
/// equal and still does not write a key, open a socket, or call the host
/// trusted. Blank text is not a fingerprint. Surrounding spaces stay part of
/// the text, so they do not make two different strings equal. A missing
/// observed key is unknown and has no status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostKeyDecision {
    status: Option<HostKeyStatus>,
    may_store: bool,
    may_trust: bool,
    may_accept: bool,
}

impl HostKeyDecision {
    pub fn status(self) -> Option<HostKeyStatus> {
        self.status
    }

    pub fn may_store(self) -> bool {
        self.may_store
    }

    pub fn may_trust(self) -> bool {
        self.may_trust
    }

    pub fn may_accept(self) -> bool {
        self.may_accept
    }
}

pub fn decide_host_key(
    stored_fingerprint: Option<&str>,
    observed_fingerprint: Option<&str>,
) -> HostKeyDecision {
    let Some(observed) = present_fingerprint(observed_fingerprint) else {
        return refused(None);
    };
    let status = classify_host_key(present_fingerprint(stored_fingerprint), observed);
    match status {
        HostKeyStatus::New | HostKeyStatus::Matched | HostKeyStatus::Mismatch => {
            refused(Some(status))
        }
    }
}

/// Blank text is absent. Any other fingerprint is compared exactly.
fn present_fingerprint(value: Option<&str>) -> Option<&str> {
    value.filter(|text| !text.trim().is_empty())
}

fn refused(status: Option<HostKeyStatus>) -> HostKeyDecision {
    HostKeyDecision {
        status,
        may_store: false,
        may_trust: false,
        may_accept: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{HostKeyStatus, classify_host_key, decide_host_key};

    #[test]
    fn comparison_keeps_new_match_and_mismatch() {
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
    fn new_or_unknown_key_is_not_stored_or_trusted() {
        let unknown = decide_host_key(None, None);
        assert!(unknown.status().is_none());
        assert!(!unknown.may_store());
        assert!(!unknown.may_trust());
        assert!(!unknown.may_accept());

        let blank = decide_host_key(Some("SHA256:old"), Some("  "));
        assert!(blank.status().is_none());
        assert!(!blank.may_store());
        assert!(!blank.may_trust());

        let new_key = decide_host_key(None, Some("SHA256:new"));
        assert_eq!(new_key.status(), Some(HostKeyStatus::New));
        assert!(!new_key.may_store());
        assert!(!new_key.may_trust());
        assert!(!new_key.may_accept());
        assert!(!format!("{new_key:?}").contains("SHA256"));
    }

    #[test]
    fn mismatch_is_not_accepted_and_a_match_is_not_trusted() {
        let mismatch = decide_host_key(Some("SHA256:old"), Some("SHA256:new"));
        assert_eq!(mismatch.status(), Some(HostKeyStatus::Mismatch));
        assert!(!mismatch.may_store());
        assert!(!mismatch.may_trust());
        assert!(!mismatch.may_accept());

        let padded = decide_host_key(Some(" SHA256:same "), Some("SHA256:same"));
        assert_eq!(padded.status(), Some(HostKeyStatus::Mismatch));
        assert!(!padded.may_store());
        assert!(!padded.may_trust());
        assert!(!padded.may_accept());

        let matched = decide_host_key(Some("SHA256:same"), Some("SHA256:same"));
        assert_eq!(matched.status(), Some(HostKeyStatus::Matched));
        assert!(!matched.may_store());
        assert!(!matched.may_trust());
        assert!(!matched.may_accept());
        let rendered = format!("{mismatch:?}{padded:?}{matched:?}");
        assert!(!rendered.contains("SHA256"));
        assert!(!rendered.contains("id_rsa"));
        assert!(!rendered.contains("password"));
        assert!(!rendered.contains("ssh-keyscan"));
    }
}
