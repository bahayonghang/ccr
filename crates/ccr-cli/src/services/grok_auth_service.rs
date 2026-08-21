//! Grok official-session status and auth off.
//!
//! Existence-read of `$GROK_HOME/auth.json` only. Token values are never parsed
//! or returned.

use crate::application::{AuthOffResult, auth_off_for_platform};
use crate::models::Platform;
use ccr_core::core::error::Result;

/// Read-only Grok official session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrokAuthCurrent {
    pub logged_in: bool,
}

/// Grok Auth surface: current session presence and auth off.
pub struct GrokAuthService;

impl GrokAuthService {
    pub fn new() -> Self {
        Self
    }

    /// Report whether `$GROK_HOME/auth.json` exists. Does not parse the token.
    pub fn current(&self) -> Result<GrokAuthCurrent> {
        let path = crate::application::auth_off::grok_auth_json_path()?;
        Ok(GrokAuthCurrent {
            logged_in: path.exists(),
        })
    }

    /// Delete the official Grok session file through the shared write core.
    pub fn off(&self) -> Result<AuthOffResult> {
        auth_off_for_platform(Platform::Grok)
    }
}

impl Default for GrokAuthService {
    fn default() -> Self {
        Self::new()
    }
}
