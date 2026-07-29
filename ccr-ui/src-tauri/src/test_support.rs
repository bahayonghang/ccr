use std::ffi::{OsStr, OsString};
use std::sync::{LazyLock, Mutex, MutexGuard};

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) struct TestProcessEnv {
    previous_vars: Vec<(&'static str, Option<OsString>)>,
    _guard: MutexGuard<'static, ()>,
}

impl TestProcessEnv {
    pub(crate) fn new() -> Self {
        Self {
            previous_vars: Vec::new(),
            _guard: lock_env(),
        }
    }

    pub(crate) fn set(&mut self, key: &'static str, value: &OsStr) {
        self.previous_vars.push((key, std::env::var_os(key)));
        // SAFETY: the shared desktop test env lock is held until Drop.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    pub(crate) fn remove(&mut self, key: &'static str) {
        self.previous_vars.push((key, std::env::var_os(key)));
        // SAFETY: the shared desktop test env lock is held until Drop.
        unsafe {
            std::env::remove_var(key);
        }
    }
}

impl Drop for TestProcessEnv {
    fn drop(&mut self) {
        for (key, previous) in self.previous_vars.drain(..).rev() {
            // SAFETY: restoration runs before releasing the shared test env lock.
            unsafe {
                match previous {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}
