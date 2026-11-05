//! Integration checks use the same exported policy as both desktop consumers.
#![allow(clippy::unwrap_used)]
use ccr_app::resources::system_prompts::{self, PromptLocation};

#[test]
fn fixed_ids_and_legacy_save_feedback_are_shared_by_consumers() {
    let spec = system_prompts::prompt_spec("codex", "codex-agents").unwrap();
    assert_eq!(spec.location, PromptLocation::Codex);
    assert_eq!(
        system_prompts::augment_save_result(
            serde_json::json!({"status":"conflict"}),
            spec,
            &"x".repeat(65 * 1024)
        ),
        serde_json::json!({"status":"conflict"})
    );
    let result = system_prompts::augment_save_result(
        serde_json::json!({"status":"saved", "token":"fixture"}),
        spec,
        &"x".repeat(65 * 1024),
    );
    assert_eq!(
        result,
        serde_json::json!({
            "status":"saved", "token":"fixture", "warning":"size", "limitHint":32 * 1024
        })
    );
    assert_eq!(
        system_prompts::prompt_spec("codex", "../outside").unwrap_err(),
        "平台 codex 不支持系统提示词文件: ../outside"
    );
}
