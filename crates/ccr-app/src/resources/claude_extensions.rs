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
/// Documented precedence for a same-name group; `Unverified` when no rule is documented.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SameNameRule {
    /// Claude documentation: a skill takes precedence over a same-name legacy command.
    SkillOverCommand,
    /// Claude documentation: only one same-name subagent in one tree is loaded; which one is not documented.
    OneSubagentLoaded,
    /// No documented precedence; client effect is unverified.
    Unverified,
}
/// Another discovered location sharing an entry's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Peer {
    pub source: &'static str,
    pub location: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SameName {
    /// The name the entries were matched on.
    pub name: String,
    pub peers: Vec<Peer>,
    pub rule: SameNameRule,
}
/// One discovered row: display text plus local same-name facts. Never contains file bodies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub text: String,
    pub same_name: Option<SameName>,
}
#[derive(Clone)]
pub struct Source {
    pub label: &'static str,
    pub state: State,
    pub entries: Vec<Entry>,
}
impl Source {
    fn failed(label: &'static str, state: State) -> Self {
        Self {
            label,
            state,
            entries: Vec::new(),
        }
    }
}

pub const SUBAGENTS: &str = "Subagents";
pub const SKILLS: &str = "Skills";
pub const LEGACY_COMMANDS: &str = "Legacy slash commands";
pub const PLUGINS: &str = "Plugins (installation registry)";
pub const LEGACY_ENTRIES: &str = "Legacy CCR entries — Claude does not read these arrays";
pub const PROJECT_SCOPE: &str = "Project scope (no project selected)";
pub const PLUGIN_CONTENTS: &str =
    "Plugin-bundled agents, skills and commands (install paths not followed)";
/// Legacy top-level settings arrays that Claude Code does not read.
pub const LEGACY_KEYS: [&str; 3] = ["agents", "slashCommands", "plugins"];

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

/// 发现阶段的中间行：显示文本、比对用名称（没有名称就不参与同名比对）与来源内位置。
struct Found {
    text: String,
    key: Option<String>,
    location: String,
}

fn markdown(store: &Store, root: &Path, kind: Kind) -> Result<Vec<Found>, State> {
    if metadata(&root.join(kind.directory()))?.is_none() {
        return Err(State::Missing);
    }
    let rows = store.list(kind).map_err(|error| match error {
        claude_files::Error::UnsafePath => State::Unsafe,
        claude_files::Error::TooLarge => State::Limit,
        _ => State::Unreadable,
    })?;
    let mut found = Vec::new();
    for row in rows {
        let path = Path::new(&row.name);
        match kind {
            Kind::Skill => {
                if !row.name.ends_with("/SKILL.md") {
                    continue;
                }
                // Reuse the skills owner's metadata parser, without exposing the body.
                let session = store.open(kind, &row.name).map_err(|_| State::Unreadable)?;
                let (meta, _) =
                    ccr_skills::models::skill::Skill::parse_frontmatter(session.content());
                // 技能按所在目录名比对：文档示例 skills/deploy/SKILL.md 与 commands/deploy.md 同为 /deploy。
                let key = path
                    .parent()
                    .and_then(Path::file_name)
                    .map(|name| name.to_string_lossy().into_owned());
                found.push(Found {
                    text: match meta.version {
                        Some(version) => format!(
                            "{} · {}",
                            row.name,
                            version.chars().take(80).collect::<String>()
                        ),
                        None => row.name.clone(),
                    },
                    key,
                    location: row.name,
                });
            }
            Kind::Agent => {
                // 子代理身份只取 frontmatter 的 name；单个文件读不到时保留该行，只是不参与同名比对。
                let key = store.open(kind, &row.name).ok().and_then(|session| {
                    ccr_skills::services::skills_service::SkillsService::extract_frontmatter_name(
                        session.content(),
                    )
                });
                found.push(Found {
                    text: row.name.clone(),
                    key: key.map(|name| name.chars().take(120).collect()),
                    location: row.name,
                });
            }
            _ => {
                let key = path
                    .file_stem()
                    .map(|name| name.to_string_lossy().into_owned());
                found.push(Found {
                    text: row.name.clone(),
                    key,
                    location: row.name,
                });
            }
        }
    }
    Ok(found)
}

fn plugins(root: &Path) -> Result<Vec<Found>, State> {
    let Some(value) = json(&root.join("plugins/installed_plugins.json"))? else {
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
    // 登记键是 name@marketplace；同一插件名在多个 marketplace 登记时按 @ 之前的名称比对。
    Ok(entries
        .keys()
        .map(|id| Found {
            text: id.clone(),
            key: Some(
                id.split_once('@')
                    .map_or(id.as_str(), |(name, _)| name)
                    .to_owned(),
            ),
            location: id.clone(),
        })
        .collect())
}

/// 同名比对的分组：子代理之间、技能与旧命令之间（两者同为斜杠命令）、插件之间。
fn group(label: &str) -> Option<u8> {
    match label {
        SUBAGENTS => Some(0),
        SKILLS | LEGACY_COMMANDS => Some(1),
        PLUGINS => Some(2),
        _ => None,
    }
}

/// 只在研究票（#155）写明规则时给出优先级，其余一律「未验证」。
fn rule(label: &str, peers: &[Peer]) -> SameNameRule {
    match label {
        SUBAGENTS => SameNameRule::OneSubagentLoaded,
        SKILLS if peers.iter().all(|peer| peer.source == LEGACY_COMMANDS) => {
            SameNameRule::SkillOverCommand
        }
        LEGACY_COMMANDS if peers.iter().any(|peer| peer.source == SKILLS) => {
            SameNameRule::SkillOverCommand
        }
        _ => SameNameRule::Unverified,
    }
}

fn same_name(
    index: &std::collections::BTreeMap<(u8, String), Vec<Peer>>,
    label: &'static str,
    row: &Found,
) -> Option<SameName> {
    let key = row.key.as_ref()?;
    let peers = index
        .get(&(group(label)?, key.clone()))?
        .iter()
        .filter(|peer| peer.source != label || peer.location != row.location)
        .cloned()
        .collect::<Vec<_>>();
    if peers.is_empty() {
        return None;
    }
    Some(SameName {
        name: key.clone(),
        rule: rule(label, &peers),
        peers,
    })
}

fn link(found: Vec<(&'static str, Result<Vec<Found>, State>)>) -> Vec<Source> {
    let mut index: std::collections::BTreeMap<(u8, String), Vec<Peer>> = Default::default();
    for (label, rows) in &found {
        let (Some(group), Ok(rows)) = (group(label), rows) else {
            continue;
        };
        for row in rows {
            if let Some(key) = &row.key {
                index.entry((group, key.clone())).or_default().push(Peer {
                    source: label,
                    location: row.location.clone(),
                });
            }
        }
    }
    found
        .into_iter()
        .map(|(label, rows)| match rows {
            Ok(rows) => Source {
                label,
                state: if rows.is_empty() {
                    State::Empty
                } else {
                    State::Discovered
                },
                entries: rows
                    .iter()
                    .map(|row| Entry {
                        text: row.text.clone(),
                        same_name: same_name(&index, label, row),
                    })
                    .collect(),
            },
            Err(state) => Source::failed(label, state),
        })
        .collect()
}

/// Uses the existing bounded Markdown resource owner; independent failures keep other sources visible.
/// Same-name facts are local only: they never claim which entry the client actually loads.
pub fn discover(root: &Path, settings: &Path, backups: &Path, locks: &Path) -> Vec<Source> {
    let store = Store::new(root.into(), backups.into(), locks.into());
    let mut sources = link(vec![
        (SUBAGENTS, markdown(&store, root, Kind::Agent)),
        (SKILLS, markdown(&store, root, Kind::Skill)),
        (LEGACY_COMMANDS, markdown(&store, root, Kind::SlashCommand)),
        (PLUGINS, plugins(root)),
    ]);
    let legacy = json(settings).and_then(|value| {
        let Some(value) = value else {
            return Err(State::Missing);
        };
        Ok(LEGACY_KEYS
            .into_iter()
            .map(|key| Entry {
                text: match value.get(key).and_then(serde_json::Value::as_array) {
                    // 数组存在才报条数；缺失写「—」，不把「没有这个数组」当成 0 条。
                    Some(items) => format!("{key}: {}", items.len()),
                    None => format!("{key}: —"),
                },
                same_name: None,
            })
            .collect())
    });
    sources.push(match legacy {
        Ok(entries) => Source {
            label: LEGACY_ENTRIES,
            state: State::Discovered,
            entries,
        },
        Err(state) => Source::failed(LEGACY_ENTRIES, state),
    });
    sources.push(Source::failed(PROJECT_SCOPE, State::NotConnected));
    // 插件自带的 agents/skills/commands 要跟随 installPath 才能发现，本清单按合同不跟随，列为未接入。
    sources.push(Source::failed(PLUGIN_CONTENTS, State::NotConnected));
    sources
}

/// 凭据形态的值统一掩码；`mask_all` 用于插件 config 这类未知结构：叶子全部掩码，键名保留。
fn redact(value: &serde_json::Value, mask_all: bool) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, child)| {
                    let sensitive = ccr_core::core::is_sensitive_log_key(key);
                    (key.clone(), redact(child, mask_all || sensitive))
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.iter().map(|item| redact(item, mask_all)).collect())
        }
        Value::String(text) if mask_all => Value::String(ccr_core::mask_sensitive(text)),
        Value::String(text) => Value::String(ccr_core::core::redact_log_text(text)),
        Value::Null => Value::Null,
        other if mask_all => Value::String(ccr_core::mask_sensitive(&other.to_string())),
        other => other.clone(),
    }
}

fn redact_plugins(value: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    let Value::Array(plugins) = value else {
        return redact(value, false);
    };
    Value::Array(
        plugins
            .iter()
            .map(|plugin| match plugin {
                Value::Object(fields) => Value::Object(
                    fields
                        .iter()
                        .map(|(name, child)| {
                            let sensitive =
                                name == "config" || ccr_core::core::is_sensitive_log_key(name);
                            (name.clone(), redact(child, sensitive))
                        })
                        .collect(),
                ),
                other => redact(other, false),
            })
            .collect(),
    )
}

/// Legacy `agents` / `slashCommands` / `plugins` arrays as a JSON document; absent keys are omitted.
/// The default copy masks credential-like values and every `plugins[].config` value; `full`
/// keeps every field verbatim. Read-only: the settings file is never written.
pub fn legacy_export(settings: &Path, full: bool) -> Result<String, State> {
    let Some(value) = json(settings)? else {
        return Err(State::Missing);
    };
    let document = value.as_object().ok_or(State::Invalid)?;
    let mut out = serde_json::Map::new();
    for key in LEGACY_KEYS {
        let Some(item) = document.get(key) else {
            continue;
        };
        let copy = match (full, key) {
            (true, _) => item.clone(),
            (false, "plugins") => redact_plugins(item),
            (false, _) => redact(item, false),
        };
        out.insert(key.into(), copy);
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(out)).map_err(|_| State::Invalid)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    fn texts(source: &Source) -> Vec<&str> {
        source
            .entries
            .iter()
            .map(|entry| entry.text.as_str())
            .collect()
    }
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
            texts(legacy),
            vec!["agents: 1", "slashCommands: —", "plugins: 0"]
        );
        assert_eq!(sources[1].state, State::Missing);
        assert_eq!(sources[3].state, State::Invalid);
        assert_eq!(sources[5].state, State::NotConnected);
        assert_eq!(sources[6].label, PLUGIN_CONTENTS);
        assert_eq!(sources[6].state, State::NotConnected);
        assert_eq!(fs::read(settings).unwrap(), original);
        assert!(!root.path().join("backups").exists());
        assert!(!root.path().join("locks").exists());
        assert!(
            sources
                .iter()
                .all(|source| texts(source).iter().all(|name| !name.contains("sentinel")))
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
        assert_eq!(texts(&sources[1]), ["demo/SKILL.md · 2.3.4"]);
        assert_eq!(sources[2].state, State::Empty);
        assert_eq!(sources[3].state, State::Limit);
        assert!(sources.iter().all(|source| {
            texts(source)
                .iter()
                .all(|name| !name.contains("private-body-sentinel"))
        }));
        assert!(!root.path().join("backup").exists());
        assert!(!root.path().join("locks").exists());
    }

    fn same(source: &Source, text: &str) -> Option<SameName> {
        source
            .entries
            .iter()
            .find(|entry| entry.text == text)
            .unwrap_or_else(|| panic!("{text} listed"))
            .same_name
            .clone()
    }

    #[test]
    fn same_name_entries_stay_listed_per_source_with_documented_rules_only() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("claude");
        for directory in ["agents/team", "skills/deploy", "commands/ops", "plugins"] {
            fs::create_dir_all(config.join(directory)).unwrap();
        }
        for (relative, content) in [
            (
                "agents/reviewer.md",
                "---\nname: reviewer\n---\nagent-body-sentinel",
            ),
            (
                "agents/team/copy.md",
                "---\nname: \"reviewer\"\n---\nagent-body-sentinel",
            ),
            ("agents/deploy.md", "---\nname: deploy\n---\nbody"),
            ("agents/nameless.md", "no frontmatter"),
            (
                "skills/deploy/SKILL.md",
                "---\nname: deploy\n---\nskill-body-sentinel",
            ),
            ("commands/deploy.md", "command-body-sentinel"),
            ("commands/ops/lint.md", "body"),
            ("commands/lint.md", "body"),
        ] {
            fs::write(config.join(relative), content).unwrap();
        }
        fs::write(
            config.join("plugins/installed_plugins.json"),
            r#"{"version":2,"plugins":{"demo@alpha":[{"scope":"user"}],"demo@beta":[{"scope":"user"}],"solo@alpha":[]}}"#,
        )
        .unwrap();
        let settings = config.join("settings.json");
        let original = br#"{"agents":[{"name":"reviewer"}]}"#;
        fs::write(&settings, original).unwrap();
        let sources = discover(
            &config,
            &settings,
            &root.path().join("backups"),
            &root.path().join("locks"),
        );
        // 各来源仍各自完整列出，不按名称合并。
        assert_eq!(texts(&sources[0]).len(), 4);
        assert_eq!(texts(&sources[1]), ["deploy/SKILL.md"]);
        assert_eq!(texts(&sources[2]), ["deploy.md", "lint.md", "ops/lint.md"]);
        // 技能与旧命令同名：文档写明技能优先。
        let skill = same(&sources[1], "deploy/SKILL.md").unwrap();
        assert_eq!(skill.name, "deploy");
        assert_eq!(skill.rule, SameNameRule::SkillOverCommand);
        assert_eq!(
            skill.peers,
            [Peer {
                source: LEGACY_COMMANDS,
                location: "deploy.md".into()
            }]
        );
        let command = same(&sources[2], "deploy.md").unwrap();
        assert_eq!(command.rule, SameNameRule::SkillOverCommand);
        assert_eq!(command.peers[0].source, SKILLS);
        // 两个旧命令同名：没有文档规则，只写未验证。
        assert_eq!(
            same(&sources[2], "lint.md").unwrap().rule,
            SameNameRule::Unverified
        );
        // 子代理按 frontmatter name 比对，与文件名无关；没有 name 的不参与比对。
        let agent = same(&sources[0], "team/copy.md").unwrap();
        assert_eq!(agent.name, "reviewer");
        assert_eq!(agent.rule, SameNameRule::OneSubagentLoaded);
        assert_eq!(agent.peers[0].location, "reviewer.md");
        assert!(same(&sources[0], "nameless.md").is_none());
        // 子代理与技能不在同一命名空间，同名也不互相标注。
        assert!(same(&sources[0], "deploy.md").is_none());
        // 同一插件在两个 marketplace 登记。
        let plugin = same(&sources[3], "demo@alpha").unwrap();
        assert_eq!(plugin.name, "demo");
        assert_eq!(plugin.rule, SameNameRule::Unverified);
        assert_eq!(plugin.peers[0].location, "demo@beta");
        assert!(same(&sources[3], "solo@alpha").is_none());
        // 旧数组里的同名条目不计入有效扩展，也不参与比对。
        assert!(
            sources[4]
                .entries
                .iter()
                .all(|entry| entry.same_name.is_none())
        );
        assert!(
            sources
                .iter()
                .all(|source| { texts(source).iter().all(|text| !text.contains("sentinel")) })
        );
        assert_eq!(fs::read(&settings).unwrap(), original);
        assert!(!root.path().join("backups").exists());
        assert!(!root.path().join("locks").exists());
    }

    #[cfg(any(unix, windows))]
    fn link_directory(link: &Path, target: &Path) {
        #[cfg(unix)]
        std::os::unix::fs::symlink(target, link).unwrap();
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let result = std::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction -Path $env:CCR_FIXTURE_LINK -Value $env:CCR_FIXTURE_TARGET | Out-Null"])
                .env("CCR_FIXTURE_LINK", link)
                .env("CCR_FIXTURE_TARGET", target)
                .creation_flags(0x08000000)
                .output()
                .unwrap();
            assert!(result.status.success(), "fixture junction creation failed");
        }
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn linked_sources_are_refused_without_hiding_other_sources() {
        let root = tempfile::tempdir().unwrap();
        let outside = root.path().join("outside");
        fs::create_dir_all(outside.join("plugins")).unwrap();
        fs::write(outside.join("escaped.md"), "---\nname: escaped\n---\n").unwrap();
        fs::write(
            outside.join("settings.json"),
            br#"{"agents":[{"name":"x","token":"linkAAAASENTINELBBBB"}]}"#,
        )
        .unwrap();
        let config = root.path().join("claude");
        fs::create_dir_all(config.join("skills/demo")).unwrap();
        fs::write(
            config.join("skills/demo/SKILL.md"),
            "---\nname: demo\n---\n",
        )
        .unwrap();
        link_directory(&config.join("agents"), &outside);
        let linked_home = root.path().join("linked-home");
        link_directory(&linked_home, &outside);
        let sources = discover(
            &config,
            &linked_home.join("settings.json"),
            &root.path().join("backups"),
            &root.path().join("locks"),
        );
        assert_eq!(sources[0].state, State::Unsafe);
        assert!(sources[0].entries.is_empty());
        assert_eq!(texts(&sources[1]), ["demo/SKILL.md"]);
        assert_eq!(sources[4].state, State::Unsafe);
        assert_eq!(
            legacy_export(&linked_home.join("settings.json"), true),
            Err(State::Unsafe)
        );
        assert!(!root.path().join("backups").exists());
    }

    #[test]
    fn legacy_export_masks_credentials_by_default_and_full_copy_is_verbatim() {
        let root = tempfile::tempdir().unwrap();
        let settings = root.path().join("settings.json");
        let original = br#"{
  "model": "keep-out-of-export",
  "env": {"ANTHROPIC_AUTH_TOKEN": "envAAAASENTINELBBBB"},
  "agents": [{"name": "old", "model": "m", "token": "agentAAAASENTINELBBBB", "x-unknown": {"keep": true}}],
  "slashCommands": [{"name": "deploy", "description": "d", "command": "curl -H 'Authorization: Bearer sk-ant-SENTINEL-0123456789'"}],
  "plugins": [{"id": "p", "name": "p", "version": "1", "enabled": true,
    "config": {"endpoint": "https://SENTINEL.example", "retries": 3, "nested": {"apiKey": "nestAAAASENTINELBBBB"}},
    "x-unknown": [1, 2]}]
}"#;
        fs::write(&settings, original).unwrap();
        let redacted: serde_json::Value =
            serde_json::from_str(&legacy_export(&settings, false).unwrap()).unwrap();
        let text = redacted.to_string();
        assert!(!text.contains("SENTINEL"), "{text}");
        assert!(redacted.get("model").is_none());
        assert!(redacted.get("env").is_none());
        // 结构与非敏感字段保留，插件 config 的键名保留、值全部掩码。
        assert_eq!(redacted["agents"][0]["name"], "old");
        assert_eq!(redacted["agents"][0]["x-unknown"]["keep"], true);
        assert_eq!(redacted["plugins"][0]["id"], "p");
        assert_eq!(
            redacted["plugins"][0]["x-unknown"],
            serde_json::json!([1, 2])
        );
        let config = redacted["plugins"][0]["config"].as_object().unwrap();
        assert_eq!(
            config.keys().collect::<Vec<_>>(),
            ["endpoint", "nested", "retries"]
        );
        assert_ne!(config["retries"], 3);

        let full: serde_json::Value =
            serde_json::from_str(&legacy_export(&settings, true).unwrap()).unwrap();
        let source: serde_json::Value = serde_json::from_slice(original).unwrap();
        for key in LEGACY_KEYS {
            assert_eq!(full[key], source[key], "{key} is exported verbatim");
        }
        assert_eq!(full.as_object().unwrap().len(), 3);
        assert_eq!(fs::read(&settings).unwrap(), original.as_slice());

        // 缺失的数组不写成空数组；缺文件、非对象和坏 JSON 各自报告。
        fs::write(&settings, br#"{"plugins":[]}"#).unwrap();
        let only: serde_json::Value =
            serde_json::from_str(&legacy_export(&settings, true).unwrap()).unwrap();
        assert_eq!(only, serde_json::json!({"plugins": []}));
        fs::write(&settings, b"[]").unwrap();
        assert_eq!(legacy_export(&settings, false), Err(State::Invalid));
        fs::write(&settings, b"broken").unwrap();
        assert_eq!(legacy_export(&settings, true), Err(State::Invalid));
        assert_eq!(
            legacy_export(&root.path().join("absent.json"), false),
            Err(State::Missing)
        );
        assert!(!root.path().join("absent.json").exists());
    }
}
