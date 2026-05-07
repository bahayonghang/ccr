//! CCR CLI/application domain crate.

pub mod application;
pub mod cli;
pub mod commands;
pub mod managers;
pub mod models;
pub mod platforms;
pub mod services;
pub mod sync;

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{LazyLock, Mutex, MutexGuard};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
