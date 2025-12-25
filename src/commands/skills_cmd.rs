use crate::core::error::Result;
use crate::managers::skills_manager::SkillsManager;
use crate::models::Platform;
use crate::models::skill::SkillRepository;
use clap::{Args, Subcommand};
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use std::str::FromStr;

#[derive(Args, Clone)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub action: SkillsAction,

    /// Specify platform (default: claude)
    #[arg(short, long)]
    pub platform: Option<String>,
}

#[derive(Subcommand, Clone)]
pub enum SkillsAction {
    /// List installed skills
    List,

    /// Install a skill from a remote repository
    ///
    /// If the skill content is not provided, it will try to fetch it from the repository.
    /// Currently, this command expects a skill name that was discovered via 'scan'.
    Install {
        /// Name of the skill to install
        name: String,
    },

    /// Uninstall a skill
    Uninstall {
        /// Name of the skill to uninstall
        name: String,
    },

    /// Manage skill repositories
    Repo {
        #[command(subcommand)]
        action: RepoAction,
    },

    /// Scan a remote repository for available skills
    Scan {
        /// Name of the repository to scan (must be added via 'repo add')
        repo_name: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum RepoAction {
    /// List configured repositories
    List,

    /// Add a new repository
    Add {
        /// Name of the repository
        name: String,

        /// URL of the repository (e.g., https://github.com/owner/repo)
        url: String,

        /// Branch to use (default: main)
        #[arg(short, long)]
        branch: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Remove a repository
    Remove {
        /// Name of the repository
        name: String,
    },
}

pub fn skills_command(args: SkillsArgs) -> Result<()> {
    let platform = if let Some(p) = args.platform {
        Platform::from_str(&p)?
    } else {
        Platform::Claude
    };

    let manager = SkillsManager::new(platform)?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(crate::core::error::CcrError::IoError)?;

    rt.block_on(async {
        match args.action {
            SkillsAction::List => {
                let skills = manager.list_skills()?;
                if skills.is_empty() {
                    println!("No skills installed for {}.", platform);
                    return Ok(());
                }

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("Name").add_attribute(Attribute::Bold),
                        Cell::new("Description").add_attribute(Attribute::Bold),
                        Cell::new("Path").add_attribute(Attribute::Bold),
                    ]);

                for skill in skills {
                    table.add_row(vec![
                        Cell::new(skill.name).fg(Color::Green),
                        Cell::new(skill.description.unwrap_or_default()),
                        Cell::new(skill.path),
                    ]);
                }

                println!("{}", table);
            }
            SkillsAction::Repo { action } => match action {
                RepoAction::List => {
                    let repos = manager.list_repositories()?;
                    if repos.is_empty() {
                        println!("No repositories configured.");
                        return Ok(());
                    }

                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .set_content_arrangement(ContentArrangement::Dynamic)
                        .set_header(vec![
                            Cell::new("Name").add_attribute(Attribute::Bold),
                            Cell::new("URL").add_attribute(Attribute::Bold),
                            Cell::new("Branch").add_attribute(Attribute::Bold),
                            Cell::new("Description").add_attribute(Attribute::Bold),
                        ]);

                    for repo in repos {
                        table.add_row(vec![
                            Cell::new(repo.name).fg(Color::Cyan),
                            Cell::new(repo.url),
                            Cell::new(repo.branch),
                            Cell::new(repo.description.unwrap_or_default()),
                        ]);
                    }
                    println!("{}", table);
                }
                RepoAction::Add {
                    name,
                    url,
                    branch,
                    description,
                } => {
                    let repo = SkillRepository {
                        name: name.clone(),
                        url,
                        branch: branch.unwrap_or_else(|| "main".to_string()),
                        description,
                        skill_count: 0,
                        last_synced: None,
                        is_official: false,
                    };
                    manager.add_repository(repo)?;
                    println!("Repository '{}' added successfully.", name);
                }
                RepoAction::Remove { name } => {
                    manager.remove_repository(&name)?;
                    println!("Repository '{}' removed successfully.", name);
                }
            },
            SkillsAction::Scan { repo_name } => {
                println!("Scanning repository '{}'...", repo_name);
                let skills = manager.fetch_remote_skills(&repo_name).await?;

                if skills.is_empty() {
                    println!("No skills found in repository '{}'.", repo_name);
                    return Ok(());
                }

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("Name").add_attribute(Attribute::Bold),
                        Cell::new("Description").add_attribute(Attribute::Bold),
                        Cell::new("Action").add_attribute(Attribute::Bold),
                    ]);

                for skill in skills {
                    table.add_row(vec![
                        Cell::new(skill.name).fg(Color::Yellow),
                        Cell::new(skill.description.unwrap_or_default()),
                        Cell::new("Use 'install <name>' to install"),
                    ]);
                }
                println!("{}", table);
            }
            SkillsAction::Install { name } => {
                println!(
                    "Searching for skill '{}' in configured repositories...",
                    name
                );
                let repos = manager.list_repositories()?;
                let mut found_skill = None;
                let mut source_repo = None;

                for repo in &repos {
                    if let Ok(skills) = manager.fetch_remote_skills(&repo.name).await
                        && let Some(skill) = skills.into_iter().find(|s| s.name == name)
                    {
                        found_skill = Some(skill);
                        source_repo = Some(repo);
                        break;
                    }
                }

                if let (Some(skill), Some(repo)) = (found_skill, source_repo) {
                    println!(
                        "Found skill '{}' in repository '{}'. Installing...",
                        name, repo.name
                    );

                    let content_url = skill.path.clone();
                    let content = match manager.fetch_skill_content(&content_url).await {
                        Ok(c) => c,
                        Err(e) => {
                            if e.to_string().contains("404") && content_url.ends_with("SKILL.md") {
                                let readme_url = content_url.replace("SKILL.md", "README.md");
                                println!("SKILL.md not found, trying README.md...");
                                manager.fetch_skill_content(&readme_url).await?
                            } else {
                                return Err(e);
                            }
                        }
                    };

                    manager.install_skill(&name, &content)?;
                    println!("Skill '{}' installed successfully!", name);
                } else {
                    println!("Skill '{}' not found in any configured repository.", name);
                }
            }
            SkillsAction::Uninstall { name } => {
                manager.uninstall_skill(&name)?;
                println!("Skill '{}' uninstalled successfully.", name);
            }
        }
        Ok(())
    })
}
