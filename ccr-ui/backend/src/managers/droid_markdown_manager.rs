// Droid Markdown Manager
// Manages markdown files in .factory/agents/ and .factory/commands/ directories

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Frontmatter for Droid agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroidAgentFrontmatter {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<String>, // Comma-separated string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Frontmatter for Droid slash commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroidCommandFrontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "argument-hint")]
    pub argument_hint: Option<String>,
}

/// Parsed markdown file with frontmatter
#[derive(Debug, Clone)]
pub struct DroidMarkdownFile<T> {
    pub frontmatter: T,
    pub content: String,
}

/// Manager for Droid markdown files (agents and commands)
pub struct DroidMarkdownManager {
    directory: PathBuf,
}

impl DroidMarkdownManager {
    /// Create manager for .factory/{subdir} directory
    pub fn from_factory_subdir(subdir: &str) -> io::Result<Self> {
        let current_dir = std::env::current_dir().map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to get current directory: {}", e),
            )
        })?;
        let directory = current_dir.join(".factory").join(subdir);
        Ok(Self { directory })
    }

    /// Create manager with custom path (for testing)
    #[allow(dead_code)]
    pub fn with_path(directory: PathBuf) -> Self {
        Self { directory }
    }

    /// Ensure directory exists
    pub fn ensure_directory(&self) -> io::Result<()> {
        if !self.directory.exists() {
            fs::create_dir_all(&self.directory)?;
        }
        Ok(())
    }

    /// List all files with their folder information (recursive)
    /// Returns a vector of tuples: (file_name, folder_path_relative)
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

    /// Read and parse a markdown file with frontmatter
    pub fn read_file<T>(&self, name: &str) -> io::Result<DroidMarkdownFile<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let path = self.directory.join(format!("{}.md", name));
        let content = fs::read_to_string(&path)?;

        let (frontmatter, body) = parse_frontmatter::<T>(&content)?;

        Ok(DroidMarkdownFile {
            frontmatter,
            content: body,
        })
    }

    /// Write a markdown file with frontmatter
    pub fn write_file<T>(&self, name: &str, file: &DroidMarkdownFile<T>) -> io::Result<()>
    where
        T: Serialize,
    {
        self.ensure_directory()?;

        // Handle subfolder paths
        let path = self.directory.join(format!("{}.md", name));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let frontmatter_yaml = serde_yaml::to_string(&file.frontmatter).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize frontmatter: {}", e),
            )
        })?;

        let content = format!("---\n{}---\n\n{}", frontmatter_yaml, file.content);
        fs::write(&path, content)?;

        Ok(())
    }

    /// Delete a markdown file
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

    /// Check if a file exists
    pub fn file_exists(&self, name: &str) -> bool {
        let path = self.directory.join(format!("{}.md", name));
        path.exists()
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
        let body = rest[end_pos + 4..].trim_start();

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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_agent() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DroidMarkdownManager::with_path(temp_dir.path().to_path_buf());

        let file = DroidMarkdownFile {
            frontmatter: DroidAgentFrontmatter {
                name: "test-agent".to_string(),
                description: Some("Test agent description".to_string()),
                tools: Some("read, write".to_string()),
                model: Some("claude-sonnet-4".to_string()),
            },
            content: "This is the system prompt.".to_string(),
        };

        manager.write_file("test-agent", &file).unwrap();

        let read_file: DroidMarkdownFile<DroidAgentFrontmatter> =
            manager.read_file("test-agent").unwrap();

        assert_eq!(read_file.frontmatter.name, "test-agent");
        assert_eq!(
            read_file.frontmatter.description,
            Some("Test agent description".to_string())
        );
        assert_eq!(read_file.content, "This is the system prompt.");
    }

    #[test]
    fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DroidMarkdownManager::with_path(temp_dir.path().to_path_buf());

        // Create test files
        let file1 = DroidMarkdownFile {
            frontmatter: DroidAgentFrontmatter {
                name: "agent1".to_string(),
                description: None,
                tools: None,
                model: None,
            },
            content: "Content 1".to_string(),
        };
        manager.write_file("agent1", &file1).unwrap();

        let file2 = DroidMarkdownFile {
            frontmatter: DroidAgentFrontmatter {
                name: "agent2".to_string(),
                description: None,
                tools: None,
                model: None,
            },
            content: "Content 2".to_string(),
        };
        manager.write_file("agent2", &file2).unwrap();

        let files = manager.list_files_with_folders().unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DroidMarkdownManager::with_path(temp_dir.path().to_path_buf());

        let file = DroidMarkdownFile {
            frontmatter: DroidAgentFrontmatter {
                name: "to-delete".to_string(),
                description: None,
                tools: None,
                model: None,
            },
            content: "Content".to_string(),
        };
        manager.write_file("to-delete", &file).unwrap();

        assert!(manager.file_exists("to-delete"));
        manager.delete_file("to-delete").unwrap();
        assert!(!manager.file_exists("to-delete"));
    }
}
