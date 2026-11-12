//! Read-only Claude metadata on captured roots; never installs, executes or follows plugin paths.
use super::claude_files::{self, Kind, Store};
use std::{fs, io::Read, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Discovered,
    Empty,
    Missing,
    Unreadable,
    Invalid,
    Unsafe,
    Limit,
    NotConnected,
}
#[derive(Clone)]
pub struct Source {
    pub label: &'static str,
    pub state: State,
    pub names: Vec<String>,
}

fn metadata(path: &Path) -> Result<Option<fs::Metadata>, State> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(State::Unsafe);
                    }
                }
                if meta.file_type().is_symlink() {
                    return Err(State::Unsafe);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(State::Unreadable),
        }
    }
    match fs::metadata(path) {
        Ok(meta) => Ok(Some(meta)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(State::Unreadable),
    }
}
fn json(path: &Path) -> Result<Option<serde_json::Value>, State> {
    let Some(meta) = metadata(path)? else {
        return Ok(None);
    };
    if !meta.is_file() {
        return Err(State::Unreadable);
    }
    if meta.len() > 1024 * 1024 {
        return Err(State::Limit);
    }
    let file = fs::File::open(path).map_err(|_| State::Unreadable)?;
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| State::Unreadable)?;
    if bytes.len() > 1024 * 1024 {
        return Err(State::Limit);
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| State::Invalid)
}
/// Uses the existing bounded Markdown resource owner; independent failures keep other sources visible.
pub fn discover(root: &Path, settings: &Path, backups: &Path, locks: &Path) -> Vec<Source> {
    let store = Store::new(root.into(), backups.into(), locks.into());
    let mut sources = Vec::new();
    for (label, kind) in [
        ("Subagents", Kind::Agent),
        ("Skills", Kind::Skill),
        ("Legacy slash commands", Kind::SlashCommand),
    ] {
        let result = metadata(&root.join(kind.directory())).and_then(|meta| {
            if meta.is_none() {
                return Err(State::Missing);
            }
            store
                .list(kind)
                .map(|rows| {
                    rows.into_iter()
                        .filter(|row| kind != Kind::Skill || row.name.ends_with("/SKILL.md"))
                        .map(|row| {
                            if kind == Kind::Skill {
                                // Reuse the skills owner's metadata parser, without exposing the body.
                                let session =
                                    store.open(kind, &row.name).map_err(|_| State::Unreadable)?;
                                let (meta, _) = ccr_skills::models::skill::Skill::parse_frontmatter(
                                    session.content(),
                                );
                                Ok(match meta.version {
                                    Some(version) => format!(
                                        "{} · {}",
                                        row.name,
                                        version.chars().take(80).collect::<String>()
                                    ),
                                    None => row.name,
                                })
                            } else {
                                Ok(row.name)
                            }
                        })
                        .collect::<Result<Vec<_>, State>>()
                })
                .map_err(|error| match error {
                    claude_files::Error::UnsafePath => State::Unsafe,
                    claude_files::Error::TooLarge => State::Limit,
                    _ => State::Unreadable,
                })?
        });
        sources.push(match result {
            Ok(names) => Source {
                label,
                state: if names.is_empty() {
                    State::Empty
                } else {
                    State::Discovered
                },
                names,
            },
            Err(state) => Source {
                label,
                state,
                names: Vec::new(),
            },
        });
    }
    let plugins = json(&root.join("plugins/installed_plugins.json")).and_then(|value| {
        let Some(value) = value else {
            return Err(State::Missing);
        };
        let entries = value
            .get("plugins")
            .and_then(serde_json::Value::as_object)
            .ok_or(State::Invalid)?;
        if entries.len() > 1000 {
            return Err(State::Limit);
        }
        if entries
            .values()
            .any(|entry| !entry.is_array() && !entry.is_object())
        {
            return Err(State::Invalid);
        }
        Ok(entries.keys().cloned().collect::<Vec<_>>())
    });
    sources.push(match plugins {
        Ok(names) => Source {
            label: "Plugins (installation registry)",
            state: if names.is_empty() {
                State::Empty
            } else {
                State::Discovered
            },
            names,
        },
        Err(state) => Source {
            label: "Plugins (installation registry)",
            state,
            names: Vec::new(),
        },
    });
    let legacy = json(settings).and_then(|value| {
        let Some(value) = value else {
            return Err(State::Missing);
        };
        Ok(["agents", "slashCommands", "plugins"]
            .into_iter()
            .map(
                |key| match value.get(key).and_then(serde_json::Value::as_array) {
                    // 数组存在才报条数；缺失写「—」，不把「没有这个数组」当成 0 条。
                    Some(items) => format!("{key}: {}", items.len()),
                    None => format!("{key}: —"),
                },
            )
            .collect())
    });
    sources.push(match legacy {
        Ok(names) => Source {
            label: "Legacy CCR entries — Claude does not read these arrays",
            state: State::Discovered,
            names,
        },
        Err(state) => Source {
            label: "Legacy CCR entries — Claude does not read these arrays",
            state,
            names: Vec::new(),
        },
    });
    sources.push(Source {
        label: "Project scope (no project selected)",
        state: State::NotConnected,
        names: Vec::new(),
    });
    sources
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn discovery_is_read_only_partial_and_keeps_legacy_arrays() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("claude");
        fs::create_dir_all(config.join("agents")).unwrap();
        fs::create_dir_all(config.join("plugins")).unwrap();
        fs::write(
            config.join("agents/reviewer.md"),
            "---\nname: reviewer\n---\nprivate body",
        )
        .unwrap();
        fs::write(config.join("plugins/installed_plugins.json"), "broken").unwrap();
        let settings = config.join("settings.json");
        let original = b"{\"agents\":[{\"name\":\"old\",\"token\":\"sentinel\"}],\"plugins\":[]}";
        fs::write(&settings, original).unwrap();
        let sources = discover(
            &config,
            &settings,
            &root.path().join("backups"),
            &root.path().join("locks"),
        );
        assert_eq!(sources[0].state, State::Discovered);
        // 旧数组只有 agents/plugins 存在：存在的报条数，缺失的写 —，不能显示 0。
        let legacy = sources
            .iter()
            .find(|source| source.label.starts_with("Legacy CCR entries"))
            .unwrap();
        assert_eq!(
            legacy.names,
            vec!["agents: 1", "slashCommands: —", "plugins: 0"]
        );
        assert_eq!(sources[1].state, State::Missing);
        assert_eq!(sources[3].state, State::Invalid);
        assert_eq!(sources[5].state, State::NotConnected);
        assert_eq!(fs::read(settings).unwrap(), original);
        assert!(!root.path().join("backups").exists());
        assert!(!root.path().join("locks").exists());
        assert!(
            sources
                .iter()
                .all(|source| source.names.iter().all(|name| !name.contains("sentinel")))
        );
    }

    #[test]
    fn skills_use_existing_metadata_and_registry_limits_do_not_hide_other_sources() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("override-claude");
        fs::create_dir_all(config.join("skills/demo")).unwrap();
        fs::create_dir_all(config.join("commands")).unwrap();
        fs::create_dir_all(config.join("plugins")).unwrap();
        fs::write(
            config.join("skills/demo/SKILL.md"),
            "---\nversion: 2.3.4\n---\nprivate-body-sentinel",
        )
        .unwrap();
        fs::write(
            config.join("plugins/installed_plugins.json"),
            vec![b' '; 1024 * 1024 + 1],
        )
        .unwrap();
        fs::write(
            config.join("plugins/do-not-execute.ps1"),
            "throw 'never run'",
        )
        .unwrap();
        let sources = discover(
            &config,
            &config.join("settings.json"),
            &root.path().join("backup"),
            &root.path().join("locks"),
        );
        assert_eq!(sources[0].state, State::Missing);
        assert_eq!(sources[1].names, ["demo/SKILL.md · 2.3.4"]);
        assert_eq!(sources[2].state, State::Empty);
        assert_eq!(sources[3].state, State::Limit);
        assert!(sources.iter().all(|source| {
            source
                .names
                .iter()
                .all(|name| !name.contains("private-body-sentinel"))
        }));
        assert!(!root.path().join("backup").exists());
        assert!(!root.path().join("locks").exists());
    }
}
