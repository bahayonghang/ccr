//! Grok official-session status and local OAuth account snapshots.
//!
//! `current` preserves the existence-only DTO; account operations parse credentials
//! inside the private service module and return only explicit display metadata.

use crate::application::AuthOffResult;
use ccr_core::core::error::Result;

/// Read-only Grok official session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrokAuthCurrent {
    pub logged_in: bool,
}

/// Grok Auth surface: current session presence and auth off.
pub struct GrokAuthService {
    captured: Option<crate::application::platform_roots::PlatformRoots>,
}

impl GrokAuthService {
    pub fn new() -> Self {
        Self { captured: None }
    }

    /// Resolve once in the caller; nested writes must use these same roots.
    pub fn from_roots(roots: crate::application::platform_roots::PlatformRoots) -> Self {
        Self {
            captured: Some(roots),
        }
    }

    /// Report whether `$GROK_HOME/auth.json` exists. Does not parse the token.
    pub fn current(&self) -> Result<GrokAuthCurrent> {
        let path = match &self.captured {
            Some(roots) => roots.grok_runtime.join("auth.json"),
            None => crate::application::auth_off::grok_auth_json_path()?,
        };
        Ok(GrokAuthCurrent {
            logged_in: path.try_exists().map_err(ccr_core::CcrError::IoError)?,
        })
    }

    /// Delete the official Grok session file through the shared write core.
    pub fn off(&self) -> Result<AuthOffResult> {
        self.off_inner(None)
    }
}

mod accounts;
pub use accounts::{
    GrokAuthAccount, GrokAuthMutation, GrokAuthRevision, GrokAuthSnapshot, GrokAuthSource,
};

impl Default for GrokAuthService {
    fn default() -> Self {
        Self::new()
    }
}
