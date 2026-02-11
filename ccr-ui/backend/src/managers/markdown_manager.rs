use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Frontmatter for slash commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFrontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "argument-hint")]
    pub argument_hint: Option<String>,
}

/// Frontmatter for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFrontmatter {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<String>, // Comma-separated string
}

/// Parsed markdown file with frontmatter
#[derive(Debug, Clone)]
pub struct MarkdownFile<T> {
    pub frontmatter: T,
    pub content: String,
}

/// Manager for markdown files (commands and agents)
pub struct MarkdownManager {
    directory: PathBuf,
}

impl MarkdownManager {
    pub fn from_directory(directory: PathBuf) -> io::Result<Self> {
        Ok(Self { directory })
    }

    pub fn from_home_subdir(subdir: &str) -> io::Result<Self> {
        // Support both Unix (HOME) and Windows (USERPROFILE)
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "HOME/USERPROFILE environment variable not set",
                )
            })?;
        let directory = Path::new(&home).join(".claude").join(subdir);
        Ok(Self { directory })
    }

    pub fn list_files_top_level(&self) -> io::Result<Vec<String>> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.directory)?;
        let mut files = Vec::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("md")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
            {
                files.push(name.to_string());
            }
        }

        files.sort();
        Ok(files)
    }

    /// List all files with their folder information (recursive)
    /// Returns a vector of tuples: (file_name, folder_path_relative)
    /// folder_path_relative is empty string for root files, or "subfolder" for files in subfolders
    pub fn list_files_with_folders(&self) -> io::Result<Vec<(String, String)>> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        Self::list_files_recursive(&self.directory, "", &mut files)?;
        files.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(files)
    }

    fn list_files_recursive(
        dir: &Path,
        folder_path: &str,
        files: &mut Vec<(String, String)>,
    ) -> io::Result<()> {
        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    files.push((name.to_string(), folder_path.to_string()));
                }
            } else if path.is_dir()
                && let Some(dir_name) = path.file_name().and_then(|s| s.to_str())
            {
                let new_folder_path = if folder_path.is_empty() {
                    dir_name.to_string()
                } else {
                    format!("{}/{}", folder_path, dir_name)
                };
                Self::list_files_recursive(&path, &new_folder_path, files)?;
            }
        }

        Ok(())
    }

    /// List all subdirectories
    #[allow(dead_code)]
    pub fn list_subdirs(&self) -> io::Result<Vec<String>> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.directory)?;
        let mut dirs = Vec::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name().and_then(|s| s.to_str())
            {
                dirs.push(name.to_string());
            }
        }

        dirs.sort();
        Ok(dirs)
    }

    /// Read and parse a markdown file with frontmatter
    /// Supports subfolder paths like "kfc/spec-design"
    pub fn read_file<T>(&self, name: &str) -> io::Result<MarkdownFile<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let path = self.directory.join(format!("{}.md", name));
        let content = fs::read_to_string(&path)?;

        let (frontmatter, body) = parse_frontmatter::<T>(&content)?;

        Ok(MarkdownFile {
            frontmatter,
            content: body,
        })
    }

    pub fn read_file_content(&self, name: &str) -> io::Result<String> {
        let path = self.directory.join(format!("{}.md", name));
        fs::read_to_string(&path)
    }

    /// Write a markdown file with frontmatter
    #[allow(dead_code)]
    pub fn write_file<T>(&self, name: &str, file: &MarkdownFile<T>) -> io::Result<()>
    where
        T: Serialize,
    {
        fs::create_dir_all(&self.directory)?;

        let frontmatter_yaml = serde_yaml::to_string(&file.frontmatter).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize frontmatter: {}", e),
            )
        })?;

        let content = format!("---\n{}---\n\n{}", frontmatter_yaml, file.content);

        let path = self.directory.join(format!("{}.md", name));
        fs::write(&path, content)?;

        Ok(())
    }

    /// Delete a markdown file
    #[allow(dead_code)]
    pub fn delete_file(&self, name: &str) -> io::Result<()> {
        let path = self.directory.join(format!("{}.md", name));
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File '{}' not found", name),
            ));
        }
        fs::remove_file(&path)?;
        Ok(())
    }
}

/// Parse YAML frontmatter from markdown content
fn parse_frontmatter<T>(content: &str) -> io::Result<(T, String)>
where
    T: for<'de> Deserialize<'de>,
{
    let content = content.trim();

    if !content.starts_with("---") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing frontmatter delimiter",
        ));
    }

    let rest = &content[3..];
    if let Some(end_pos) = rest.find("\n---") {
        let frontmatter_str = &rest[..end_pos];
        let body = &rest[end_pos + 4..].trim_start();

        let frontmatter: T = serde_yaml::from_str(frontmatter_str).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse frontmatter: {}", e),
            )
        })?;

        Ok((frontmatter, body.to_string()))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing closing frontmatter delimiter",
        ))
    }
}
