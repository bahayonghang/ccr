use crate::models::skill::Skill;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct SkillsManager {
    base_path: PathBuf,
}

impl SkillsManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").expect("HOME environment variable not set");
        // Default path: ~/.claude/skills
        // TODO: Make this configurable if needed
        let base_path = Path::new(&home).join(".claude").join("skills");
        Self { base_path }
    }

    fn ensure_base_dir(&self) -> io::Result<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)?;
        }
        Ok(())
    }

    pub fn list_skills(&self) -> io::Result<Vec<Skill>> {
        self.ensure_base_dir()?;
        let mut skills = Vec::new();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let skill_file = path.join("SKILL.md");

                if skill_file.exists() {
                    let instruction = fs::read_to_string(&skill_file)?;
                    // Simple description extraction: first line or empty
                    let description = instruction.lines().next().map(|s| s.to_string());

                    skills.push(Skill {
                        name,
                        description,
                        path: path.to_string_lossy().to_string(),
                        instruction,
                    });
                }
            }
        }

        Ok(skills)
    }

    #[allow(dead_code)]
    pub fn get_skill(&self, name: &str) -> io::Result<Skill> {
        let skill_path = self.base_path.join(name);
        let skill_file = skill_path.join("SKILL.md");

        if !skill_file.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Skill not found"));
        }

        let instruction = fs::read_to_string(&skill_file)?;
        let description = instruction.lines().next().map(|s| s.to_string());

        Ok(Skill {
            name: name.to_string(),
            description,
            path: skill_path.to_string_lossy().to_string(),
            instruction,
        })
    }

    pub fn create_skill(&self, name: &str, instruction: &str) -> io::Result<()> {
        self.ensure_base_dir()?;
        let skill_path = self.base_path.join(name);

        if skill_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Skill already exists",
            ));
        }

        fs::create_dir_all(&skill_path)?;
        fs::write(skill_path.join("SKILL.md"), instruction)?;

        Ok(())
    }

    pub fn update_skill(&self, name: &str, instruction: &str) -> io::Result<()> {
        let skill_path = self.base_path.join(name);
        let skill_file = skill_path.join("SKILL.md");

        if !skill_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Skill not found"));
        }

        fs::write(skill_file, instruction)?;

        Ok(())
    }

    pub fn delete_skill(&self, name: &str) -> io::Result<()> {
        let skill_path = self.base_path.join(name);

        if !skill_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Skill not found"));
        }

        fs::remove_dir_all(skill_path)?;

        Ok(())
    }
}
