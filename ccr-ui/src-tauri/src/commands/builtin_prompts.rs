//! 内置提示词命令

use serde_json::Value;

#[tauri::command]
pub async fn list_builtin_prompts() -> Result<Value, String> {
    let prompts = tokio::task::spawn_blocking(|| {
        let prompts = ccr_skills::get_builtin_prompts();
        serde_json::to_value(prompts).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(prompts)
}

#[tauri::command]
pub async fn get_builtin_prompt(id: String) -> Result<Value, String> {
    let prompt = tokio::task::spawn_blocking(move || {
        let result = ccr_skills::get_prompt_by_id(&id);
        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(prompt)
}

#[tauri::command]
pub async fn get_builtin_prompts_by_category(category: String) -> Result<Value, String> {
    let prompts = tokio::task::spawn_blocking(move || {
        use ccr_skills::PromptCategory;

        let cat = match category.as_str() {
            "code_review" => PromptCategory::CodeReview,
            "debugging" => PromptCategory::Debugging,
            "refactoring" => PromptCategory::Refactoring,
            "testing" => PromptCategory::Testing,
            "documentation" => PromptCategory::Documentation,
            "security" => PromptCategory::Security,
            _ => PromptCategory::General,
        };

        let results = ccr_skills::get_prompts_by_category(cat);
        serde_json::to_value(results).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(prompts)
}
