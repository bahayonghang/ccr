use crate::core::error::Result;
use crate::managers::prompts_manager::PromptsManager;
use crate::models::Platform;
use crate::models::prompt::{PromptPreset, PromptTarget};
use clap::{Args, Subcommand};
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use std::str::FromStr;

#[derive(Args, Clone)]
pub struct PromptsArgs {
    #[command(subcommand)]
    pub action: PromptsAction,

    /// Specify platform (default: claude)
    #[arg(short, long)]
    pub platform: Option<String>,
}

#[derive(Subcommand, Clone)]
pub enum PromptsAction {
    /// List all prompt presets
    List,

    /// Add a new prompt preset
    Add {
        /// Name of the preset
        name: String,

        /// Target file (claude/agents/gemini)
        #[arg(short, long)]
        target: String,

        /// Content of the prompt (use @file to read from file)
        #[arg(short, long)]
        content: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Apply a preset to the target file
    Apply {
        /// Name of the preset to apply
        name: String,
    },

    /// Show content of a preset
    Show {
        /// Name of the preset
        name: String,
    },

    /// Remove a preset
    Remove {
        /// Name of the preset
        name: String,
    },

    /// Show current content of a target file
    Current {
        /// Target file (claude/agents/gemini)
        target: String,
    },
}

pub fn prompts_command(args: PromptsArgs) -> Result<()> {
    let platform = if let Some(p) = args.platform {
        Platform::from_str(&p)?
    } else {
        Platform::Claude
    };

    let manager = PromptsManager::new(platform)?;

    match args.action {
        PromptsAction::List => {
            let presets = manager.list_presets()?;
            if presets.is_empty() {
                println!("No prompt presets configured for {}.", platform);
                return Ok(());
            }

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Name").add_attribute(Attribute::Bold),
                    Cell::new("Target").add_attribute(Attribute::Bold),
                    Cell::new("Description").add_attribute(Attribute::Bold),
                    Cell::new("Tags").add_attribute(Attribute::Bold),
                ]);

            for preset in presets {
                table.add_row(vec![
                    Cell::new(preset.name).fg(Color::Cyan),
                    Cell::new(preset.target_file.to_string()).fg(Color::Yellow),
                    Cell::new(preset.description.unwrap_or_default()),
                    Cell::new(preset.tags.join(", ")),
                ]);
            }

            println!("{}", table);
        }
        PromptsAction::Add {
            name,
            target,
            content,
            description,
        } => {
            let target_enum = PromptTarget::from_str(&target)
                .map_err(crate::core::error::CcrError::ConfigError)?;

            // Handle @file syntax
            let content = if let Some(file_path) = content.strip_prefix('@') {
                std::fs::read_to_string(file_path).map_err(crate::core::error::CcrError::IoError)?
            } else {
                content
            };

            let preset = PromptPreset {
                name: name.clone(),
                description,
                target_file: target_enum,
                content,
                tags: vec![],
            };

            manager.add_preset(preset)?;
            println!("Preset '{}' added successfully.", name);
        }
        PromptsAction::Apply { name } => {
            manager.apply_preset(&name)?;
            println!("Preset '{}' applied successfully.", name);
            println!("Note: A backup of the previous file was created with .backup extension.");
        }
        PromptsAction::Show { name } => {
            let preset = manager.get_preset(&name)?;
            println!("Preset: {}", preset.name);
            if let Some(desc) = preset.description {
                println!("Description: {}", desc);
            }
            println!("Target: {}", preset.target_file);
            println!("\n--- Content ---");
            println!("{}", preset.content);
        }
        PromptsAction::Remove { name } => {
            manager.remove_preset(&name)?;
            println!("Preset '{}' removed successfully.", name);
        }
        PromptsAction::Current { target } => {
            let target_enum = PromptTarget::from_str(&target)
                .map_err(crate::core::error::CcrError::ConfigError)?;

            let content = manager.get_current_content(target_enum)?;
            if content.is_empty() {
                println!("No content found in {}", target);
            } else {
                println!("--- Current content of {} ---", target);
                println!("{}", content);
            }
        }
    }

    Ok(())
}
