//! Bounded materialization of existing resource owners' user-scoped files.
//! No HOME scan, link following, installation or process execution is performed.
use crate::environments::{
    config_mirror::{ConfigMirror, validate_materialized_paths},
    io::{EntryKind, EnvironmentIo, Error},
};
use std::{collections::BTreeSet, path::Path};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy)]
pub enum Family {
    Instruction(crate::runtime::Platform),
    Presets(crate::runtime::Platform),
    Claude,
    ClaudeRules,
    ClaudeStyles,
    ClaudeCollections,
    Codex,
    CodexFiles,
    PlatformFile(super::platform_files::Kind, bool),
    Mcp,
    McpProject,
    McpClaude,
    McpCodex,
    McpAntigravity,
    CodexProject,
}
#[derive(Clone)]
struct Tree {
    path: &'static str,
    suffixes: &'static [&'static str],
    depth: usize,
}
fn plan(family: Family) -> (&'static [&'static str], Vec<Tree>) {
    let tree = |path, suffixes, depth| Tree {
        path,
        suffixes,
        depth,
    };
    match family {
        Family::Instruction(platform) => (
            match platform {
                crate::runtime::Platform::Claude => &[".claude/CLAUDE.md"],
                crate::runtime::Platform::Codex => &[".codex/AGENTS.md"],
                crate::runtime::Platform::Gemini => &[".gemini/GEMINI.md"],
                crate::runtime::Platform::OpenCode => &[".config/opencode/AGENTS.md"],
                _ => &[],
            },
            vec![],
        ),
        Family::Presets(platform) => (
            match platform {
                crate::runtime::Platform::Claude => &[".ccr/platforms/claude/prompts.toml"],
                crate::runtime::Platform::Codex => &[".ccr/platforms/codex/prompts.toml"],
                crate::runtime::Platform::Gemini => &[".ccr/platforms/gemini/prompts.toml"],
                _ => &[],
            },
            vec![],
        ),
        Family::ClaudeRules => (&[], vec![tree(".claude/rules", &[".md"], 16)]),
        Family::ClaudeStyles => (&[], vec![tree(".claude/output-styles", &[".md"], 16)]),
        Family::ClaudeCollections => (&[".claude/settings.json"], vec![]),
        Family::CodexFiles => (&[], vec![tree(".codex/agents", &[".toml"], 0)]),
        Family::Claude => (
            &[
                ".claude/settings.json",
                ".claude/plugins/installed_plugins.json",
            ],
            vec![
                tree(".claude/rules", &[".md"], 16),
                tree(".claude/output-styles", &[".md"], 16),
                tree(".claude/agents", &[".md"], 16),
                tree(".claude/commands", &[".md"], 16),
                tree(".claude/skills", &["/SKILL.md"], 16),
            ],
        ),
        Family::Codex => (
            &[
                ".ccr/platforms/codex/agent-sources/sources.json",
                ".ccr/platforms/codex/agent-sources/installs.json",
            ],
            vec![
                tree(".codex/agents", &[".toml"], 0),
                tree(".ccr/platforms/codex/agent-sources/catalogs", &[".json"], 0),
            ],
        ),
        Family::PlatformFile(kind, project) => {
            use super::platform_files::Kind;
            let (path, suffixes, depth): (_, &'static [&'static str], _) = match kind {
                Kind::OpenCodeAgent => (
                    if project {
                        ".opencode/agents"
                    } else {
                        ".config/opencode/agents"
                    },
                    &[".md"],
                    0,
                ),
                Kind::OpenCodeCommand => (
                    if project {
                        ".opencode/commands"
                    } else {
                        ".config/opencode/commands"
                    },
                    &[".md"],
                    0,
                ),
                Kind::AntigravityCommand => (".gemini/commands", &[".toml"], 16),
                Kind::Extension => (
                    ".gemini/antigravity-cli/extensions",
                    &["/gemini-extension.json", "/package.json"],
                    1,
                ),
                Kind::OpenCodePlugin => return (&[], vec![]),
            };
            (&[], vec![tree(path, suffixes, depth)])
        }
        Family::CodexProject => (&[], vec![tree(".codex/agents", &[".toml"], 0)]),
        Family::McpClaude => (&[".claude.json"], vec![]),
        Family::McpCodex => (&[".codex/config.toml"], vec![]),
        Family::McpAntigravity => (&[".gemini/antigravity-cli/mcp_config.json"], vec![]),
        Family::McpProject => (&[".mcp.json"], vec![]),
        Family::Mcp => (
            &[
                ".claude.json",
                ".claude/settings.json",
                ".codex/config.toml",
                ".gemini/antigravity-cli/mcp_config.json",
            ],
            vec![],
        ),
    }
}
/// Contains private source bytes and original session authority, never Debug/Serialize.
pub struct ResourceMirror {
    pub mirror: ConfigMirror,
    trees: Vec<Tree>,
    pub unsupported: Vec<String>,
}
impl ResourceMirror {
    pub async fn capture(
        io: EnvironmentIo,
        family: Family,
        targets: BTreeSet<String>,
        cancel: CancellationToken,
    ) -> Result<Self, Error> {
        let (fixed, trees) = plan(family);
        for target in &targets {
            if !fixed.contains(&target.as_str()) && !trees.iter().any(|tree| allowed(tree, target))
            {
                return Err(Error::InvalidPath);
            }
        }
        let mut paths: BTreeSet<String> = fixed.iter().map(|p| (*p).into()).collect();
        paths.extend(targets.clone());
        let mut unsupported = Vec::new();
        let mut directories = Vec::new();
        let mut visited = 0;
        for tree in &trees {
            let mut pending = vec![(tree.path.to_owned(), 0)];
            while let Some((directory, depth)) = pending.pop() {
                let entries = match io.entries(&directory, cancel.clone()).await {
                    Ok(Some(entries)) => entries,
                    Ok(None) => continue,
                    Err(Error::Ssh("ssh_invalid_remote_path")) => {
                        unsupported.push(directory);
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                directories.push(directory.clone());
                for entry in entries {
                    visited += 1;
                    if visited > 4096 {
                        return Err(Error::TooLarge);
                    }
                    let path = format!("{directory}/{}", entry.name);
                    if validate_materialized_paths([path.as_str()]).is_err()
                        || entry.kind == EntryKind::Unsupported
                    {
                        unsupported.push(path);
                        continue;
                    }
                    match entry.kind {
                        EntryKind::Directory if depth < tree.depth => {
                            pending.push((path, depth + 1))
                        }
                        EntryKind::Directory => {}
                        EntryKind::File if allowed(tree, &path) => {
                            paths.insert(path);
                        }
                        _ => {}
                    }
                }
            }
        }
        let mut aliases = std::collections::BTreeMap::<String, String>::new();
        for path in paths.iter().chain(directories.iter()) {
            let mut prefix = String::new();
            for part in path.split('/') {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(part);
                if let Some(previous) = aliases.insert(prefix.to_ascii_lowercase(), prefix.clone())
                    && previous != prefix
                {
                    unsupported.push(previous);
                    unsupported.push(prefix.clone());
                }
            }
        }
        paths.retain(|path| !targets_overlap(&BTreeSet::from([path.clone()]), &unsupported));
        directories.retain(|path| !targets_overlap(&BTreeSet::from([path.clone()]), &unsupported));
        validate_materialized_paths(paths.iter().map(String::as_str))?;
        if targets_overlap(&targets, &unsupported) {
            return Err(Error::UnrepresentablePath);
        }
        if matches!(family, Family::Claude) {
            for path in fixed {
                if matches!(
                    io.metadata(path, cancel.clone()).await,
                    Err(Error::Ssh("ssh_invalid_remote_path"))
                ) {
                    paths.remove(*path);
                    unsupported.push((*path).into());
                }
            }
        }
        let mirror = ConfigMirror::capture(io, paths, None, cancel).await?;
        for directory in directories {
            std::fs::create_dir_all(mirror.root().join(directory)).map_err(|_| Error::Io)?;
        }
        Ok(Self {
            mirror,
            trees,
            unsupported,
        })
    }
    pub fn root(&self) -> &Path {
        self.mirror.root()
    }
    pub fn label(&self, path: &Path) -> Result<String, Error> {
        let relative = path
            .strip_prefix(self.root())
            .map_err(|_| Error::InvalidPath)?
            .to_str()
            .ok_or(Error::InvalidPath)?
            .replace('\\', "/");
        match self.mirror.io().roots().relative(&relative)? {
            crate::environments::io::EnvironmentPath::Ssh(path) => Ok(path.as_str().to_owned()),
            _ => Err(Error::Unsupported),
        }
    }
    pub async fn publish(&self, cancel: CancellationToken) -> Result<(), Error> {
        let mut outputs = BTreeSet::new();
        for tree in &self.trees {
            let mut pending = vec![(self.root().join(tree.path), 0)];
            while let Some((directory, depth)) = pending.pop() {
                let entries = match std::fs::read_dir(directory) {
                    Ok(entries) => entries,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(_) => return Err(Error::Io),
                };
                for entry in entries {
                    let entry = entry.map_err(|_| Error::Io)?;
                    let kind = entry.file_type().map_err(|_| Error::Io)?;
                    let relative = entry
                        .path()
                        .strip_prefix(self.root())
                        .map_err(|_| Error::InvalidPath)?
                        .to_str()
                        .ok_or(Error::InvalidPath)?
                        .replace('\\', "/");
                    if kind.is_symlink() {
                        return Err(Error::InvalidPath);
                    }
                    if kind.is_dir() && depth < tree.depth {
                        pending.push((entry.path(), depth + 1));
                    } else if kind.is_file() && allowed(tree, &relative) {
                        outputs.insert(relative);
                    }
                    if outputs.len() > 256 {
                        return Err(Error::TooLarge);
                    }
                }
            }
        }
        if targets_overlap(&outputs, &self.unsupported) {
            return Err(Error::UnrepresentablePath);
        }
        self.mirror.publish_outputs(outputs, cancel).await
    }
}
fn allowed(tree: &Tree, path: &str) -> bool {
    path.strip_prefix(tree.path)
        .and_then(|p| p.strip_prefix('/'))
        .is_some_and(|p| {
            p.split('/').count() <= tree.depth + 1
                && tree.suffixes.iter().any(|suffix| path.ends_with(suffix))
        })
}
fn targets_overlap(paths: &BTreeSet<String>, unsupported: &[String]) -> bool {
    paths.iter().any(|p| {
        unsupported.iter().any(|u| {
            p.eq_ignore_ascii_case(u)
                || p.to_ascii_lowercase()
                    .starts_with(&format!("{}/", u.to_ascii_lowercase()))
        })
    })
}
