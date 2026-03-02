//! Lock Recovery Utilities
//!
//! Provides safe recovery functions for poisoned mutexes and rwlocks.
//! When a thread panics while holding a lock, the lock becomes "poisoned".
//! These utilities allow safe recovery by logging a warning and extracting the inner lock.

use std::sync::{LockResult, MutexGuard, RwLockReadGuard, RwLockWriteGuard};
use tracing::warn;

/// Recover from a poisoned mutex lock
///
/// If the mutex is poisoned, logs a warning and returns the inner guard.
/// This allows the program to continue instead of panicking.
///
/// # Example
/// ```rust,ignore
/// use std::sync::Mutex;
/// use crate::core::lock_utils::recover_mutex_lock;
///
/// let mutex = Mutex::new(42);
/// let guard = recover_mutex_lock(mutex.lock()).unwrap();
/// ```
#[allow(dead_code)]
pub fn recover_mutex_lock<T>(guard: LockResult<MutexGuard<'_, T>>) -> MutexGuard<'_, T> {
    guard.unwrap_or_else(|poisoned| {
        warn!("Mutex lock was poisoned, recovering from poison");
        poisoned.into_inner()
    })
}

/// Recover from a poisoned RwLock read lock
///
/// If the lock is poisoned, logs a warning and returns the inner guard.
///
/// # Example
/// ```rust,ignore
/// use std::sync::RwLock;
/// use crate::core::lock_utils::recover_rwlock_read;
///
/// let lock = RwLock::new(42);
/// let guard = recover_rwlock_read(lock.read()).unwrap();
/// ```
pub fn recover_rwlock_read<T>(guard: LockResult<RwLockReadGuard<'_, T>>) -> RwLockReadGuard<'_, T> {
    guard.unwrap_or_else(|poisoned| {
        warn!("RwLock read lock was poisoned, recovering from poison");
        poisoned.into_inner()
    })
}

/// Recover from a poisoned RwLock write lock
///
/// If the lock is poisoned, logs a warning and returns the inner guard.
///
/// # Example
/// ```rust,ignore
/// use std::sync::RwLock;
/// use crate::core::lock_utils::recover_rwlock_write;
///
/// let lock = RwLock::new(42);
/// let guard = recover_rwlock_write(lock.write()).unwrap();
/// ```
pub fn recover_rwlock_write<T>(
    guard: LockResult<RwLockWriteGuard<'_, T>>,
) -> RwLockWriteGuard<'_, T> {
    guard.unwrap_or_else(|poisoned| {
        warn!("RwLock write lock was poisoned, recovering from poison");
        poisoned.into_inner()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::{
        sync::{Mutex, RwLock},
        thread,
    };

    #[test]
    fn test_recover_mutex_lock() {
        let mutex = Mutex::new(42);
        let guard = recover_mutex_lock(mutex.lock());
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_recover_mutex_after_poison() {
        use std::sync::Arc;

        let mutex = Arc::new(Mutex::new(42));
        let mutex_ref = Arc::clone(&mutex);

        // Poison the mutex
        let _ = thread::spawn(move || {
            let _guard = mutex_ref.lock().unwrap();
            panic!("Poison the mutex!");
        })
        .join();

        // Recover from poison
        let guard = recover_mutex_lock(mutex.lock());
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_recover_rwlock_read() {
        let lock = RwLock::new(42);
        let guard = recover_rwlock_read(lock.read());
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_recover_rwlock_write() {
        let lock = RwLock::new(42);
        let mut guard = recover_rwlock_write(lock.write());
        *guard = 100;
        assert_eq!(*guard, 100);
    }

    #[test]
    fn test_recover_rwlock_read_after_poison() {
        use std::sync::Arc;

        let lock = Arc::new(RwLock::new(42));
        let lock_ref = Arc::clone(&lock);

        // Poison the lock with a write panic
        let _ = thread::spawn(move || {
            let _guard = lock_ref.write().unwrap();
            panic!("Poison the rwlock!");
        })
        .join();

        // Recover from poison
        let guard = recover_rwlock_read(lock.read());
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_recover_rwlock_write_after_poison() {
        use std::sync::Arc;

        let lock = Arc::new(RwLock::new(42));
        let lock_ref = Arc::clone(&lock);

        // Poison the lock
        let _ = thread::spawn(move || {
            let _guard = lock_ref.write().unwrap();
            panic!("Poison the rwlock!");
        })
        .join();

        // Recover from poison
        let mut guard = recover_rwlock_write(lock.write());
        *guard = 200;
        assert_eq!(*guard, 200);
    }
}
