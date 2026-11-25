//! Explicitly admitted development catalogs, rooted in this binary's checkout.
use super::*;
use std::ffi::OsStr;
use std::io::Read;
use std::path::Component;

const DENIED: &str =
    "Check-in fixture requires paired files in a marked directory under outputs/desktop";
const MARKER: &str = ".ccr-gpui-checkin-fixture";
const IDENTITY: &[u8] = b"ccr-gpui-checkin-fixture-v1";

pub(in crate::runtime) fn admit(
    profiles: &Path,
    registry: &Path,
    delay: Duration,
) -> Result<ConfigSource, &'static str> {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(DENIED)?;
    let outputs = checkout
        .canonicalize()
        .map_err(|_| DENIED)?
        .join("outputs")
        .join("desktop");
    let root = registry.parent().ok_or(DENIED)?;
    if !root.is_absolute()
        || profiles.parent() != Some(root)
        || profiles.file_name() != Some(OsStr::new("profiles.toml"))
        || registry.file_name() != Some(OsStr::new("config.toml"))
        || root.components().any(|c| c == Component::ParentDir)
        || outputs.canonicalize().map_err(|_| DENIED)? != outputs
    {
        return Err(DENIED);
    }
    let canonical = root.canonicalize().map_err(|_| DENIED)?;
    if !canonical.starts_with(&outputs) || canonical == outputs {
        return Err(DENIED);
    }
    for path in root.ancestors() {
        let meta = path.symlink_metadata().map_err(|_| DENIED)?;
        if meta.file_type().is_symlink() {
            return Err(DENIED);
        }
        #[cfg(windows)]
        if std::os::windows::fs::MetadataExt::file_attributes(&meta) & 0x400 != 0 {
            return Err(DENIED);
        }
    }
    let profile_db = canonical.join("profiles.db");
    if profile_db.symlink_metadata().is_ok()
        && (!profile_db.is_file() || profile_db.canonicalize().map_err(|_| DENIED)? != profile_db)
    {
        return Err(DENIED);
    }
    for name in ["profiles.toml", "config.toml", MARKER] {
        let path = canonical.join(name);
        if !path.symlink_metadata().is_ok_and(|m| m.is_file())
            || path.canonicalize().map_err(|_| DENIED)? != path
        {
            return Err(DENIED);
        }
    }
    let mut value = Vec::new();
    std::fs::File::open(canonical.join(MARKER))
        .map_err(|_| DENIED)?
        .take(IDENTITY.len() as u64 + 1)
        .read_to_end(&mut value)
        .map_err(|_| DENIED)?;
    if value != IDENTITY {
        return Err(DENIED);
    }
    Ok(ConfigSource::Fixture {
        profiles: canonical.join("profiles.toml"),
        registry: canonical.join("config.toml"),
        delay,
    })
}

#[cfg(test)]
#[test]
#[allow(clippy::unwrap_used)]
fn launch_requires_the_complete_marked_pair() {
    use crate::runtime::LaunchMode;
    let outputs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../outputs/desktop");
    std::fs::create_dir_all(&outputs).unwrap();
    let outputs = outputs.canonicalize().unwrap();
    let root = tempfile::tempdir_in(&outputs).unwrap();
    let root = root.path();
    let args = |root: &Path| {
        vec![
            "--fixture-checkin-loopback".into(),
            "--fixture-profiles".into(),
            root.join("profiles.toml").into_os_string(),
            "--fixture-registry".into(),
            root.join("config.toml").into_os_string(),
        ]
    };
    assert!(LaunchMode::parse(args(root)).is_err());
    for name in ["config.toml", "profiles.toml"] {
        std::fs::write(root.join(name), b"").unwrap();
    }
    assert!(LaunchMode::parse(args(root)).is_err());
    std::fs::write(root.join(MARKER), b"ccr-gpui-sync-fixture-v1").unwrap();
    assert!(LaunchMode::parse(args(root)).is_err());
    std::fs::write(root.join(MARKER), IDENTITY).unwrap();
    assert!(matches!(
        LaunchMode::parse(args(root)),
        Ok(LaunchMode::CheckinFixture(_))
    ));
    assert!(matches!(
        LaunchMode::parse(args(root).into_iter().skip(1)),
        Ok(LaunchMode::App(_))
    ));
    let outside = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(root).unwrap().flatten() {
        std::fs::copy(entry.path(), outside.path().join(entry.file_name())).unwrap();
    }
    assert!(LaunchMode::parse(args(outside.path())).is_err());
    let mut mismatched = args(root);
    mismatched[2] = outside.path().join("profiles.toml").into_os_string();
    assert!(LaunchMode::parse(mismatched).is_err());
    let mut benchmark = args(root);
    benchmark.extend(["--benchmark-fixture-root".into(), root.as_os_str().into()]);
    assert!(LaunchMode::parse(benchmark).is_err());
    assert!(LaunchMode::parse(args(root).into_iter().take(3)).is_err());
}
