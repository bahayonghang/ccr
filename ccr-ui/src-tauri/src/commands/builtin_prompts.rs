//! 内置提示词命令

use ccr_skills::{BuiltinPrompt, PromptVariable};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/builtin_prompts/")]
pub struct PromptVariableDto {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
    pub required: bool,
}

impl From<PromptVariable> for PromptVariableDto {
    fn from(value: PromptVariable) -> Self {
        Self {
            name: value.name,
            description: value.description,
            default: value.default,
            required: value.required,
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/builtin_prompts/")]
pub struct BuiltinPromptDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub variables: Vec<PromptVariableDto>,
}

impl From<BuiltinPrompt> for BuiltinPromptDto {
    fn from(value: BuiltinPrompt) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            content: value.content,
            category: value.category.as_str().to_string(),
            tags: value.tags,
            variables: value
                .variables
                .into_iter()
                .map(PromptVariableDto::from)
                .collect(),
        }
    }
}

#[tauri::command]
pub async fn list_builtin_prompts() -> Result<Vec<BuiltinPromptDto>, String> {
    tokio::task::spawn_blocking(|| {
        ccr_skills::get_builtin_prompts()
            .into_iter()
            .map(BuiltinPromptDto::from)
            .collect()
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))
}

#[tauri::command]
pub async fn get_builtin_prompt(id: String) -> Result<Option<BuiltinPromptDto>, String> {
    tokio::task::spawn_blocking(move || {
        ccr_skills::get_prompt_by_id(&id).map(BuiltinPromptDto::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))
}

#[tauri::command]
pub async fn get_builtin_prompts_by_category(
    category: String,
) -> Result<Vec<BuiltinPromptDto>, String> {
    tokio::task::spawn_blocking(move || {
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

        ccr_skills::get_prompts_by_category(cat)
            .into_iter()
            .map(BuiltinPromptDto::from)
            .collect()
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))
}
