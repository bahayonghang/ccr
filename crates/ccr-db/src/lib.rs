//! # ccr-db
//!
//! CCR Desktop 数据库层 — 提供 SQLite 统一存储和数据模型。
//!
//! 本 crate 从原先的 UI 后端实现中提取，封装所有与 SQLite 数据库相关的功能，
//! 使 Tauri 应用和未来的其他消费者可以共享数据访问逻辑。
//!
//! ## 模块结构
//!
//! - `database` — 连接池、Schema 定义、迁移、Repository (DAO)
//! - `models` — 数据模型 (CheckIn、UI State、Usage、Monitoring)
//! - `services` — 高层业务服务 (用量导入、日志持久化、格式转换)
//! - `core` — 基础设施 (错误类型、命令执行器)

pub mod core;
pub mod database;
pub mod managers;
pub mod models;
pub mod services;

#[cfg(test)]
pub(crate) mod test_support {
    use std::ffi::{OsStr, OsString};
    use std::path::Path;
    use std::sync::{LazyLock, Mutex, MutexGuard};
    use tempfile::TempDir;

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) struct TestOpenCodeEnv {
        temp_dir: TempDir,
        previous_vars: Vec<(&'static str, Option<OsString>)>,
        _env_guard: MutexGuard<'static, ()>,
    }

    impl TestOpenCodeEnv {
        pub(crate) fn new() -> Self {
            let env_guard = ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let temp_dir =
                tempfile::tempdir().expect("test OpenCode env temp dir should be created");

            let mut previous_vars = Vec::new();
            set_env_var(
                &mut previous_vars,
                "CCR_OPENCODE_DIR",
                temp_dir.path().as_os_str(),
            );

            Self {
                temp_dir,
                previous_vars,
                _env_guard: env_guard,
            }
        }

        pub(crate) fn opencode_dir(&self) -> &Path {
            self.temp_dir.path()
        }
    }

    impl Drop for TestOpenCodeEnv {
        fn drop(&mut self) {
            self.restore_env_vars();
        }
    }

    impl TestOpenCodeEnv {
        fn restore_env_vars(&mut self) {
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
        // SAFETY: TestOpenCodeEnv holds the process-wide ccr-db test env lock until Drop.
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn restore_env_var(key: &str, previous: Option<OsString>) {
        // SAFETY: Drop runs while TestOpenCodeEnv still holds the process-wide test env lock.
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::TestOpenCodeEnv;
        use std::ffi::OsString;

        #[test]
        fn test_open_code_env_sets_and_restores_opencode_dir() {
            let mut env = TestOpenCodeEnv::new();
            let previous_opencode_dir = captured_previous(&env, "CCR_OPENCODE_DIR");

            assert_eq!(
                std::env::var_os("CCR_OPENCODE_DIR").as_deref(),
                Some(env.opencode_dir().as_os_str())
            );

            env.restore_env_vars();

            assert_eq!(std::env::var_os("CCR_OPENCODE_DIR"), previous_opencode_dir);
        }

        fn captured_previous(env: &TestOpenCodeEnv, key: &'static str) -> Option<OsString> {
            env.previous_vars
                .iter()
                .find_map(|(captured_key, value)| (*captured_key == key).then(|| value.clone()))
                .flatten()
        }
    }
}
