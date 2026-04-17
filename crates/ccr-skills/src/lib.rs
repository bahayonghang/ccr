pub mod managers;
pub mod models;
pub mod services;
pub mod skills_ext;

pub use managers::builtin_prompts::{
    BuiltinPrompt, PromptCategory, PromptVariable, get_builtin_prompts, get_prompt_by_id,
    get_prompts_by_category,
};
pub use managers::mcp_preset_manager::{McpPresetManager, McpSyncManager, get_builtin_presets};
pub use managers::prompts_manager::PromptsManager;
pub use models::mcp_preset::{McpPreset, McpPresetCategory, McpServerSpec};
pub use models::prompt::{PromptPreset, PromptTarget};
pub use models::skill::{Skill, SkillMetadata, SkillRepository};
pub use models::skills::*;
pub use services::skills_service::SkillsService;
