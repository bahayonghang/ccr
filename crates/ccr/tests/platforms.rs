use std::ffi::OsString;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

static PLATFORM_ENV_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct PlatformTestEnv {
    _guard: MutexGuard<'static, ()>,
    temp_dir: TempDir,
    previous_ccr_root: Option<OsString>,
    previous_lock_dir: Option<OsString>,
}

impl PlatformTestEnv {
    pub(crate) fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}

impl Drop for PlatformTestEnv {
    fn drop(&mut self) {
        restore_env_var("CCR_ROOT", self.previous_ccr_root.take());
        restore_env_var("CCR_LOCK_DIR", self.previous_lock_dir.take());
    }
}

pub(crate) fn setup_platform_test_env() -> PlatformTestEnv {
    let guard = PLATFORM_ENV_LOCK
        .lock()
        .expect("platform test env lock should not be poisoned");
    let temp_dir = TempDir::new().expect("platform test temp dir should be created");
    let lock_dir = temp_dir.path().join(".locks");
    std::fs::create_dir_all(temp_dir.path()).expect("platform test temp dir should exist");
    std::fs::create_dir_all(&lock_dir).expect("platform test lock dir should be created");

    let previous_ccr_root = std::env::var_os("CCR_ROOT");
    let previous_lock_dir = std::env::var_os("CCR_LOCK_DIR");

    unsafe {
        std::env::set_var("CCR_ROOT", temp_dir.path());
        std::env::set_var("CCR_LOCK_DIR", &lock_dir);
    }

    PlatformTestEnv {
        _guard: guard,
        temp_dir,
        previous_ccr_root,
        previous_lock_dir,
    }
}

fn restore_env_var(key: &str, value: Option<OsString>) {
    unsafe {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

#[path = "platforms/auth_profile_surface.rs"]
mod auth_profile_surface;
#[path = "platforms/general.rs"]
mod general;
#[path = "platforms/integration.rs"]
mod integration;
