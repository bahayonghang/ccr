//! Explicit benchmark source identity. Loading is blocking work admitted before
//! window creation; no marker or filename changes the ordinary Fixture mode.
use ccr_cli::application::{
    platform_roots::PlatformRoots, profile_lifecycle::ProfileLifecycleContext,
};
use ccr_config::{Platform, PlatformPaths};
use ccr_db::database::{DbPool, PoolConfig};
use ccr_usage::ReportTimezone;
use chrono::{DateTime, Utc};
#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::io::Read;
use std::{
    fs,
    path::{Component, Path, PathBuf},
    sync::OnceLock,
};

pub struct Layout {
    pub root: PathBuf,
    pub roots: PlatformRoots,
    pub id: String,
    pub scenario: String,
    pub logical_sha256: String,
    pub as_of: DateTime<Utc>,
    pub timezone: ReportTimezone,
    pub profile_count: usize,
    pub session_count: usize,
    pub usage_count: usize,
    main: OnceLock<Result<DbPool, &'static str>>,
}

fn unlinked(path: &Path) -> Result<(), &'static str> {
    for path in path.ancestors() {
        match fs::symlink_metadata(path) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err("Benchmark linked path refused");
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err("Benchmark reparse path refused");
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("Benchmark path is unavailable"),
        }
    }
    Ok(())
}
#[cfg(test)]
fn json(path: &Path) -> Result<Value, &'static str> {
    unlinked(path)?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| "Benchmark manifest is unavailable")?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Benchmark manifest is unreadable")?;
    if bytes.len() > 1024 * 1024 {
        return Err("Benchmark manifest is too large");
    }
    serde_json::from_slice(&bytes).map_err(|_| "Benchmark manifest is invalid")
}
#[cfg(test)]
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
// The generator uses sorted JSON keys; GPUI enables preserve_order transitively.
// Canonicalize keys only, preserving original values and opaque source identities.
#[cfg(test)]
fn logical_hash(value: &Value) -> Result<String, &'static str> {
    fn ordered(value: &Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(key, value)| (key.clone(), ordered(value)))
                    .collect::<std::collections::BTreeMap<_, _>>()
                    .into_iter()
                    .collect(),
            ),
            Value::Array(values) => Value::Array(values.iter().map(ordered).collect()),
            value => value.clone(),
        }
    }
    Ok(hex(&Sha256::digest(
        serde_json::to_vec(&ordered(value)).map_err(|_| "Benchmark logical data is invalid")?,
    )))
}
#[cfg(test)]
fn file_hash(path: &Path) -> Result<String, &'static str> {
    let mut reader = fs::File::open(path).map_err(|_| "Benchmark input is unavailable")?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|_| "Benchmark input is unreadable")?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex(&digest.finalize()))
}

impl Layout {
    pub fn load(requested: &Path) -> Result<Self, &'static str> {
        let seed = ccr_app::benchmark::Seed::load(requested, ccr_app::benchmark::Endpoint::Gpui)?;
        let roots = PlatformRoots::fixture(&seed.root)
            .map_err(|_| "Benchmark platform roots are invalid")?;
        Ok(Self {
            root: seed.root,
            roots,
            id: seed.id,
            scenario: seed.scenario,
            logical_sha256: seed.logical_sha256,
            as_of: seed.as_of,
            timezone: seed.timezone,
            profile_count: seed.profile_count,
            session_count: seed.session_count,
            usage_count: seed.usage_count,
            main: OnceLock::new(),
        })
    }
    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }
    pub fn check_path(&self, path: &Path) -> Result<(), &'static str> {
        if !path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err("Benchmark path is invalid");
        }
        unlinked(path)?;
        let existing = path
            .ancestors()
            .find(|part| part.exists())
            .ok_or("Benchmark parent is missing")?;
        if !existing
            .canonicalize()
            .map_err(|_| "Benchmark parent is unavailable")?
            .starts_with(&self.root)
        {
            return Err("Benchmark path escapes the captured root");
        }
        Ok(())
    }
    pub fn main_pool(&self) -> Result<DbPool, &'static str> {
        self.main
            .get_or_init(|| {
                let path = self.path("ui/ccr-ui.db");
                self.check_path(&path)?;
                if !path.is_file() {
                    return Err("Benchmark main database is missing");
                }
                ccr_db::database::create_pool(
                    &path,
                    Some(PoolConfig {
                        max_size: 8,
                        ..Default::default()
                    }),
                )
                .map_err(|_| "Benchmark main database cannot open")
            })
            .clone()
    }
    pub fn platform_paths(&self, platform: Platform) -> ccr_core::Result<PlatformPaths> {
        let root = self.roots.ccr.clone();
        let platform_dir = root.join("platforms").join(platform.short_name());
        let paths = PlatformPaths {
            registry_file: root.join("config.toml"),
            profiles_file: platform_dir.join("profiles.toml"),
            settings_file: platform_dir.join("settings.json"),
            history_file: root
                .join("history")
                .join(format!("{}.json", platform.short_name())),
            backups_dir: root.join("backups").join(platform.short_name()),
            platform_dir,
            root,
        };
        for path in [
            &paths.registry_file,
            &paths.profiles_file,
            &paths.profiles_db(),
            &paths.backups_dir,
        ] {
            self.check_path(path)
                .map_err(|error| ccr_core::CcrError::ConfigError(error.into()))?;
        }
        Ok(paths)
    }
    pub fn profile_context(&self, platform: Platform) -> ccr_core::Result<ProfileLifecycleContext> {
        let paths = self.platform_paths(platform)?;
        let locks = self.roots.locks.clone();
        let history = self.roots.ccr.join("history.db");
        match platform {
            Platform::Claude => ProfileLifecycleContext::for_claude(
                paths,
                self.roots.claude.clone(),
                locks,
                history,
            )
            .map(|context| context.with_process_environment(false)),
            Platform::Codex => ProfileLifecycleContext::for_codex(
                paths,
                self.roots.codex_runtime.clone(),
                locks,
                history,
            ),
            Platform::Grok => ProfileLifecycleContext::for_grok(
                paths,
                self.roots.grok_runtime.join("config.toml"),
                locks,
                history,
            ),
            _ => Err(ccr_core::CcrError::PlatformNotSupported(
                "benchmark_profiles".into(),
            )),
        }
    }
}

/// Admission and all filesystem/database/log initialization precede window creation.
pub async fn load_services(
    executor: ccr_app::runtime::RuntimeHandle,
    root: PathBuf,
) -> Result<super::Services, &'static str> {
    use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, TaskEvent};
    let key = RequestKey {
        environment: EnvironmentId::Local("benchmark-startup".into()),
        platform: Platform::Claude,
        generation: 0,
        request_id: 0,
    };
    let mut subscription = executor
        .read(key, move |_, _| {
            let layout = Layout::load(&root)?;
            layout.main_pool()?;
            ccr_core::init_logger_at(&layout.path("logs"));
            Ok::<_, &'static str>(std::sync::Arc::new(layout))
        })
        .map_err(|_| "Benchmark startup admission failed")?;
    while let Some(event) = subscription.next().await {
        if let TaskEvent::Terminal(result) = event {
            return match result.as_ref() {
                Completion::Success(layout) => Ok(super::Services::new(
                    executor,
                    super::ConfigSource::Benchmark(layout.clone()),
                )),
                Completion::Failed(error) => Err(*error),
                _ => Err("Benchmark startup did not complete"),
            };
        }
    }
    Err("Benchmark startup did not complete")
}

#[cfg(test)]
mod tests;
