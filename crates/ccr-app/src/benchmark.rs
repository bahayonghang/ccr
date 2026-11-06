//! Pure paired-seed admission shared by desktop clients, before their effect owners.
//! Validation is real startup work; it is never subtracted from process timings.
use ccr_usage::ReportTimezone;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};
#[derive(Clone, Copy, Debug)]
pub enum Endpoint {
    Gpui,
    Tauri,
}
impl Endpoint {
    fn name(self) -> &'static str {
        match self {
            Self::Gpui => "gpui",
            Self::Tauri => "tauri",
        }
    }
    fn other(self) -> Self {
        match self {
            Self::Gpui => Self::Tauri,
            Self::Tauri => Self::Gpui,
        }
    }
}
#[derive(Clone)]
pub struct Seed {
    pub root: PathBuf,
    pub id: String,
    pub scenario: String,
    pub logical_sha256: String,
    pub as_of: DateTime<Utc>,
    pub timezone: ReportTimezone,
    pub profile_count: usize,
    pub session_count: usize,
    pub usage_count: usize,
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
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
// The generator uses sorted JSON keys; GPUI enables preserve_order transitively.
// Canonicalize keys only, preserving original values and opaque source identities.
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

impl Seed {
    pub fn load(requested: &Path, endpoint: Endpoint) -> Result<Self, &'static str> {
        if !requested.is_absolute()
            || requested
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err("Benchmark root must be absolute without traversal");
        }
        unlinked(requested)?;
        let root = requested
            .canonicalize()
            .map_err(|_| "Benchmark root is unavailable")?;
        if !root.is_dir() {
            return Err("Benchmark root must be a directory");
        }
        for name in [
            "home",
            "ccr",
            "ui",
            "llmusage",
            "logs",
            "preferences",
            "locks",
            "crypto",
        ] {
            let path = root.join(name);
            unlinked(&path)?;
            if !path.is_dir() {
                return Err("Benchmark layout directory is missing or invalid");
            }
        }
        let marker = json(&root.join("ccr-tauri-fixture.json"))?;
        let pair = root.parent().ok_or("Benchmark pair root is missing")?;
        let manifest = json(&pair.join("logical-manifest.json"))?;
        if marker["version"].as_u64() != Some(1)
            || manifest["version"].as_u64() != Some(1)
            || manifest["synthetic"].as_bool() != Some(true)
            || manifest["report_timezone"].as_str() != Some("UTC")
        {
            return Err("Benchmark manifest version or scope is unsupported");
        }
        let scenario = manifest["scenario"]
            .as_str()
            .ok_or("Benchmark scenario is missing")?;
        let expected = match scenario {
            "S0" => (0, 0, 0),
            "S1" => (100, 10_000, 100_000),
            "S2" => (10_000, 10_000, 100_000),
            _ => return Err("Benchmark scenario is unsupported"),
        };
        let counts = &manifest["counts"];
        if (
            counts["claude_profiles"].as_u64(),
            counts["codex_sessions"].as_u64(),
            counts["usage_events"].as_u64(),
        ) != (Some(expected.0), Some(expected.1), Some(expected.2))
        {
            return Err("Benchmark scenario counts are invalid");
        }
        let name = endpoint.name();
        let other = endpoint.other().name();
        let end = &manifest["ends"][name];
        let recorded = PathBuf::from(
            end["root"]
                .as_str()
                .ok_or("Benchmark source identity is missing")?,
        );
        if recorded != root {
            return Err("Benchmark source identity differs from the generated root");
        }
        let id = marker["fixture_id"]
            .as_str()
            .ok_or("Benchmark identity is missing")?;
        if id != format!("benchmark-{}", scenario.to_ascii_lowercase()) {
            return Err("Benchmark identity does not match the scenario");
        }
        let logical = manifest["logical_sha256"]
            .as_str()
            .ok_or("Benchmark logical hash is missing")?;
        let computed = logical_hash(&end["logical"])?;
        if logical != computed || end["logical_sha256"].as_str() != Some(logical) {
            return Err("Benchmark logical hash differs");
        }
        if root != pair.join(name) {
            return Err("Benchmark source must be the declared member of its pair");
        }
        let paired_root = pair.join(other);
        unlinked(&paired_root)?;
        if !paired_root.is_dir()
            || Path::new(
                manifest["ends"][other]["root"]
                    .as_str()
                    .ok_or("Benchmark paired root is missing")?,
            ) != paired_root
        {
            return Err("Benchmark paired source identity differs");
        }
        let paired = &manifest["ends"][other];
        if paired["logical_sha256"].as_str() != Some(logical)
            || logical_hash(&paired["logical"])? != logical
        {
            return Err("Benchmark paired logical hash differs");
        }
        let parsed = DateTime::parse_from_rfc3339(
            manifest["as_of_utc"]
                .as_str()
                .ok_or("Benchmark clock is missing")?,
        )
        .map_err(|_| "Benchmark clock is invalid")?;
        if parsed.offset().local_minus_utc() != 0 {
            return Err("Benchmark report clock must be UTC");
        }
        if parsed.timestamp_subsec_nanos() >= 1_000_000_000 {
            return Err("Benchmark report clock cannot contain a leap second");
        }
        let as_of = parsed.with_timezone(&Utc);
        // Validate all descendants before a domain scanner can consume them.
        let mut pending = vec![root.clone()];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(directory).map_err(|_| "Benchmark tree is unreadable")? {
                let entry = entry.map_err(|_| "Benchmark tree is unreadable")?;
                unlinked(&entry.path())?;
                if entry
                    .file_type()
                    .map_err(|_| "Benchmark tree is unreadable")?
                    .is_dir()
                {
                    pending.push(entry.path());
                }
            }
        }
        for (role, relative) in [
            ("catalog", "ccr/platforms/claude/profiles.db"),
            ("usage", "llmusage/llmusage.db"),
            ("archive", "ccr/analytics/usage.db"),
            ("main", "ui/ccr-ui.db"),
        ] {
            let expected = &end["artifact_sha256"][role];
            let path = root.join(relative);
            // Static trial input must not depend on un-hashed recovery data.
            // Inspect without opening SQLite: admission never repairs/checkpoints.
            for suffix in ["-wal", "-journal", "-shm"] {
                let mut sidecar = path.as_os_str().to_os_string();
                sidecar.push(suffix);
                let sidecar = PathBuf::from(sidecar);
                unlinked(&sidecar)?;
                match fs::symlink_metadata(&sidecar) {
                    Ok(metadata)
                        if !metadata.is_file() || (suffix != "-shm" && metadata.len() > 0) =>
                    {
                        return Err("Benchmark database sidecar is not a static seed input");
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(_) => return Err("Benchmark database sidecar is unavailable"),
                }
            }
            if role == "catalog" && scenario == "S0" && expected.is_null() && !path.exists() {
                continue;
            }
            if expected.as_str() != Some(file_hash(&path)?.as_str()) {
                return Err("Benchmark seed input changed; restore the original trial data");
            }
        }
        Ok(Self {
            root,
            id: id.to_string(),
            scenario: scenario.to_string(),
            logical_sha256: logical.to_string(),
            as_of,
            timezone: ReportTimezone::Utc,
            profile_count: expected.0 as usize,
            session_count: expected.1 as usize,
            usage_count: expected.2 as usize,
        })
    }
}

#[cfg(test)]
mod tests;
