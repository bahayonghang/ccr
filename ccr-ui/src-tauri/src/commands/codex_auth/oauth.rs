//! Compatibility adapter; the framework-free controller is shared with native desktop.
#[cfg(test)]
use ccr_cli::application::codex_oauth::exchange_oauth_tokens_at;
pub(super) use ccr_cli::application::codex_oauth::{Backend, Controller, Event, Events};

#[cfg(test)]
use ccr_cli::application::codex_oauth::{
    CommitResult as CodexAuthMutationResponse, TokenResponse as CodexOAuthTokenResponse, Work,
};
#[cfg(test)]
use ccr_codex::services::CodexOAuthPendingState;
#[cfg(test)]
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
#[cfg(test)]
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
#[cfg(test)]
use tokio_util::sync::CancellationToken;

#[cfg(test)]
#[path = "oauth_tests.rs"]
mod tests;
