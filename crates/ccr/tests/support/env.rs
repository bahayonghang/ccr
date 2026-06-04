use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct CcrIntegrationTestEnv {
    _guard: MutexGuard<'static, ()>,
    temp_dir: TempDir,
    previous_vars: Vec<(&'static str, Option<OsString>)>,
}

impl CcrIntegrationTestEnv {
    pub(crate) fn new() -> Self {
        let guard = TEST_ENV_LOCK
            .lock()
            .expect("CCR integration test env lock should not be poisoned");
        let temp_dir = TempDir::new().expect("CCR integration test temp dir should be created");
        let lock_dir = temp_dir.path().join(".locks");

        std::fs::create_dir_all(temp_dir.path()).expect("CCR integration temp dir should exist");
        std::fs::create_dir_all(&lock_dir).expect("CCR integration lock dir should be created");

        let mut previous_vars = Vec::new();
        set_env_var(&mut previous_vars, "CCR_ROOT", temp_dir.path().as_os_str());
        set_env_var(&mut previous_vars, "CCR_LOCK_DIR", lock_dir.as_os_str());

        Self {
            _guard: guard,
            temp_dir,
            previous_vars,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

impl Drop for CcrIntegrationTestEnv {
    fn drop(&mut self) {
        for (key, previous) in self.previous_vars.drain(..).rev() {
            restore_env_var(key, previous);
        }
    }
}

fn set_env_var(
    previous_vars: &mut Vec<(&'static str, Option<OsString>)>,
    key: &'static str,
    value: &OsStr,
) {
    previous_vars.push((key, std::env::var_os(key)));
    // SAFETY: CcrIntegrationTestEnv holds the process-wide integration test env lock until Drop.
    unsafe {
        std::env::set_var(key, value);
    }
}

fn restore_env_var(key: &str, previous: Option<OsString>) {
    // SAFETY: Drop runs while CcrIntegrationTestEnv still holds the integration test env lock.
    unsafe {
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
