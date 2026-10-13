//! Read-only activity facade for background profile snapshots.

use ccr_config::models::platform::Platform;
pub use ccr_store::{
    ProfileActivity, ProfileActivityAvailability, ProfileActivityError, ProfileActivitySnapshot,
};

pub fn load_profile_activity(
    platform: Platform,
    names: &[String],
) -> Result<ProfileActivitySnapshot, ProfileActivityError> {
    let path =
        ccr_store::Database::default_path().map_err(|_| ProfileActivityError::PathUnavailable)?;
    ccr_store::read_profile_activity(&path, platform, names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_activity_resolves_existing_path_precedence_without_initializing()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut home = crate::test_support::TestHome::new();
        let custom = home.root().join("missing-activity-data");
        home.set_env("CCR_DATA_DIR", custom.as_os_str());
        assert_eq!(ccr_store::Database::default_path()?, custom.join("data.db"));
        let result = load_profile_activity(Platform::Codex, &["work".to_owned()])?;
        assert_eq!(
            result.availability,
            ProfileActivityAvailability::MissingDatabase
        );
        assert!(!custom.exists());
        assert!(!home.root().join("data.db").exists());
        home.remove_env("CCR_DATA_DIR");
        assert_eq!(
            ccr_store::Database::default_path()?,
            home.root().join("data.db")
        );
        home.remove_env("CCR_ROOT");
        // Windows resolves the OS known folder, not the HOME/USERPROFILE fixture.
        // Resolve only: never open the fallback database from this test.
        assert_eq!(
            ccr_store::Database::default_path()?,
            dirs::home_dir()
                .ok_or("fixture home location unavailable")?
                .join(".ccr/data.db")
        );
        Ok(())
    }

    #[test]
    fn profile_activity_reads_the_existing_writer_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        use ccr_store::{
            HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType,
        };

        let mut home = crate::test_support::TestHome::new();
        let root = home.root().to_owned();
        home.set_env("CCR_DATA_DIR", root.as_os_str());
        let manager = HistoryManager::with_default()?;
        for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
            let entry = HistoryEntry::new(
                OperationType::Switch,
                OperationDetails {
                    from_config: None,
                    to_config: Some("work".into()),
                    backup_path: None,
                    extra: Some(platform.to_string()),
                },
                OperationResult::Success,
            );
            let expected = entry.timestamp.with_timezone(&chrono::Utc);
            manager.add(entry)?;
            let snapshot = load_profile_activity(platform, &["work".to_owned()])?;
            assert_eq!(snapshot.availability, ProfileActivityAvailability::Ready);
            assert_eq!(snapshot.profiles["work"].recent_activations, [expected]);
        }
        Ok(())
    }
}
