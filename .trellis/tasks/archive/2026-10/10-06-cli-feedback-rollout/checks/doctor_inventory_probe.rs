use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

fn inventory(root: &Path, fresh: bool) -> BTreeMap<PathBuf, SystemTime> {
    let mut entries = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = if fresh {
                fs::metadata(&path).unwrap()
            } else {
                entry.metadata().unwrap()
            };
            if metadata.is_dir() {
                pending.push(path.clone());
            }
            entries.insert(path.strip_prefix(root).unwrap().to_path_buf(), metadata.modified().unwrap());
        }
    }
    entries
}

fn main() {
    let root = PathBuf::from(std::env::args_os().nth(1).unwrap());
    assert!(root.is_dir());
    for round in 0..20 {
        let fixture = root.join(format!("fixture-{round}"));
        for path in ["ccr/platforms/claude", "claude", "codex", "grok", "work"] {
            fs::create_dir_all(fixture.join(path)).unwrap();
        }
        fs::write(fixture.join("ccr/config.toml"), "[claude]\nenabled = true\n").unwrap();
        fs::write(fixture.join("ccr/platforms/claude/profiles.toml"), "default_config = ''\ncurrent_config = ''\n").unwrap();
        fs::write(fixture.join("claude/settings.json"), "{}").unwrap();
        let before = inventory(&fixture, false);
        let fresh_before = inventory(&fixture, true);
        std::thread::sleep(Duration::from_millis(10));
        let after = inventory(&fixture, false);
        let fresh_after = inventory(&fixture, true);
        assert_eq!(fresh_before, fresh_after);
        for (path, stamp) in before {
            if after[&path] != stamp {
                println!("round={round} changed={path:?} cached_before={stamp:?} cached_after={:?} fresh_before={:?} fresh_after={:?}", after[&path], fresh_before[&path], fresh_after[&path]);
            }
        }
    }
}
