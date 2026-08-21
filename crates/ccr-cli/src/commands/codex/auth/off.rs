//! `ccr codex auth off`

#![allow(clippy::unused_async)]

use crate::application::auth_off_for_platform;
use crate::commands::claude::auth::off::print_auth_off;
use crate::models::Platform;
use ccr_core::core::error::Result;

pub async fn off_command(json: bool) -> Result<()> {
    let result = auth_off_for_platform(Platform::Codex)?;
    print_auth_off(
        json,
        result.changed,
        result.path,
        result.profile_pointer,
        result.warnings,
    )
}
