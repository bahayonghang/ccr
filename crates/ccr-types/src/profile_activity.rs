//! Recorded CCR activations, not model request times or lifetime usage counts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileActivityAvailability {
    Ready,
    #[default]
    MissingDatabase,
    MissingSchema,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileActivity {
    /// Newest first, at most three; valid future instants are retained verbatim.
    pub recent_activations: Vec<DateTime<Utc>>,
    /// Eligible records whose timestamp cannot be read as strict RFC3339.
    pub invalid_timestamps: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileActivitySnapshot {
    pub availability: ProfileActivityAvailability,
    /// Exact profile names within the platform requested by the caller.
    pub profiles: HashMap<String, ProfileActivity>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_fields_do_not_claim_history_is_ready() -> Result<(), serde_json::Error> {
        let snapshot: ProfileActivitySnapshot = serde_json::from_str("{}")?;
        assert_eq!(snapshot, ProfileActivitySnapshot::default());
        assert_eq!(
            snapshot.availability,
            ProfileActivityAvailability::MissingDatabase
        );
        Ok(())
    }

    #[test]
    fn activation_instants_and_partial_data_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let activity = ProfileActivity {
            recent_activations: vec![
                DateTime::parse_from_rfc3339("2026-10-12T12:00:00-07:00")?.with_timezone(&Utc),
            ],
            invalid_timestamps: 2,
        };
        let snapshot = ProfileActivitySnapshot {
            availability: ProfileActivityAvailability::Ready,
            profiles: HashMap::from([("work".to_owned(), activity)]),
        };
        let json = serde_json::to_string(&snapshot)?;
        assert!(json.contains("2026-10-12T19:00:00Z"));
        assert_eq!(
            serde_json::from_str::<ProfileActivitySnapshot>(&json)?,
            snapshot
        );
        Ok(())
    }
}
