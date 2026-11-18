//! Explicit declarations for capabilities that run locally, follow the selected
//! environment, or do not access an environment at all.
use crate::runtime::EnvironmentId;

use super::LOCAL_ONLY;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvironmentPolicy {
    LocalOnly,
    FollowsEnvironment,
    Independent,
}

impl EnvironmentPolicy {
    /// Admission does not establish SSH authority. Environment I/O must still
    /// capture an already connected session before queueing work.
    pub fn admit(self, environment: &EnvironmentId) -> Result<(), &'static str> {
        match (self, environment) {
            (Self::Independent, _)
            | (Self::LocalOnly | Self::FollowsEnvironment, EnvironmentId::Local(_))
            | (Self::FollowsEnvironment, EnvironmentId::Ssh(_)) => Ok(()),
            _ => Err(LOCAL_ONLY),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declarations_do_not_enable_wsl_or_implicitly_connect_ssh() {
        for (environment, local, follows) in [
            (EnvironmentId::Local("local".into()), true, true),
            (EnvironmentId::Ssh("host".into()), false, true),
            (EnvironmentId::Wsl("distro".into()), false, false),
        ] {
            assert_eq!(
                EnvironmentPolicy::LocalOnly.admit(&environment).is_ok(),
                local
            );
            assert_eq!(
                EnvironmentPolicy::FollowsEnvironment
                    .admit(&environment)
                    .is_ok(),
                follows
            );
            assert!(EnvironmentPolicy::Independent.admit(&environment).is_ok());
        }
    }
}
