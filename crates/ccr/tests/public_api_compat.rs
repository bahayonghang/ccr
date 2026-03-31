use std::mem::size_of;

#[test]
fn legacy_public_paths_remain_available() {
    let _ = ccr::models::OpenAiAuthMethod::Api;

    assert!(size_of::<ccr::models::prompt::PromptPreset>() > 0);
    assert!(size_of::<ccr::models::mcp_preset::McpServerSpec>() > 0);
    assert!(size_of::<ccr::models::skills::SkillOperationResponse>() > 0);
    assert!(size_of::<ccr::services::codex_session_service::CodexSessionInventory>() > 0);
    assert!(size_of::<ccr::sessions::SessionFilter>() > 0);
    assert!(size_of::<ccr::sessions::SessionIndexer>() > 0);
    assert!(size_of::<ccr::sessions::SessionSummary>() > 0);
}
