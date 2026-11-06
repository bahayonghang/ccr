//! Per-environment cache, edit draft, and job label.
//!
//! Switching the active environment hides the previous slot. A local-only read
//! of another environment returns a static reason and does not run the read.

use std::collections::BTreeMap;

use crate::runtime::{EnvironmentId, RequestKey, ResponseGate};

pub const LOCAL_ONLY: &str = "local_only_environment";

#[derive(Clone, PartialEq, Eq)]
pub struct SurfaceSlot<T> {
    pub cache: Option<T>,
    pub draft: Option<String>,
    pub job: Option<String>,
}

impl<T> Default for SurfaceSlot<T> {
    fn default() -> Self {
        Self {
            cache: None,
            draft: None,
            job: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SurfaceKey {
    Local(String),
    Wsl(String),
    Ssh(String),
}

fn surface_key(environment: &EnvironmentId) -> SurfaceKey {
    match environment {
        EnvironmentId::Local(name) => SurfaceKey::Local(name.clone()),
        EnvironmentId::Wsl(name) => SurfaceKey::Wsl(name.clone()),
        EnvironmentId::Ssh(name) => SurfaceKey::Ssh(name.clone()),
    }
}

pub struct EnvironmentSurface<T> {
    active: EnvironmentId,
    generation: u64,
    slots: BTreeMap<SurfaceKey, SurfaceSlot<T>>,
}

impl<T> EnvironmentSurface<T> {
    pub fn new(active: EnvironmentId) -> Self {
        Self {
            active,
            generation: 0,
            slots: BTreeMap::new(),
        }
    }

    pub fn active(&self) -> &EnvironmentId {
        &self.active
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn set_generation(&mut self, generation: u64) {
        self.generation = generation;
    }

    /// Hide the previous slot. Its cache, draft, and job stay stored.
    pub fn switch(&mut self, environment: EnvironmentId) {
        self.active = environment;
        self.generation = self.generation.saturating_add(1);
    }

    pub fn note_cache(&mut self, value: T) {
        self.slot_mut().cache = Some(value);
    }

    pub fn note_draft(&mut self, value: String) {
        self.slot_mut().draft = Some(value);
    }

    pub fn note_job(&mut self, value: String) {
        self.slot_mut().job = Some(value);
    }

    pub fn accepts(&self, gate: &ResponseGate, key: &RequestKey) -> bool {
        self.active == key.environment && self.generation == key.generation && gate.accepts(key)
    }

    fn slot_mut(&mut self) -> &mut SurfaceSlot<T> {
        let key = surface_key(&self.active);
        self.slots.entry(key).or_default()
    }
}

impl<T: Clone> EnvironmentSurface<T> {
    pub fn cache(&self) -> Option<T> {
        self.visible().cache
    }

    pub fn visible(&self) -> SurfaceSlot<T> {
        self.slots
            .get(&surface_key(&self.active))
            .cloned()
            .unwrap_or_default()
    }
}

pub fn admit_local_only(environment: &EnvironmentId) -> Result<(), &'static str> {
    match environment {
        EnvironmentId::Local(_) => Ok(()),
        EnvironmentId::Wsl(_) | EnvironmentId::Ssh(_) => Err(LOCAL_ONLY),
    }
}

/// Read `owner`'s source only when `caller` is that same local environment.
/// Any other caller returns [`LOCAL_ONLY`] and does not run `read`.
pub fn read_local_source<T>(
    caller: &EnvironmentId,
    owner: &EnvironmentId,
    read: impl FnOnce() -> T,
) -> Result<T, &'static str> {
    if let (EnvironmentId::Local(caller_name), EnvironmentId::Local(owner_name)) = (caller, owner)
        && caller_name == owner_name
    {
        return Ok(read());
    }
    Err(LOCAL_ONLY)
}

#[cfg(test)]
mod tests {
    use super::{EnvironmentSurface, LOCAL_ONLY, admit_local_only, read_local_source};
    use crate::runtime::{EnvironmentId, Platform, RequestKey, ResponseGate};

    fn key(environment: EnvironmentId, generation: u64) -> RequestKey {
        RequestKey {
            environment,
            platform: Platform::Claude,
            generation,
            request_id: generation,
        }
    }

    #[test]
    fn switch_keeps_each_environment_cache_draft_and_job() {
        let local = EnvironmentId::Local("local".into());
        let remote = EnvironmentId::Ssh("fixture-ssh".into());
        let mut surface = EnvironmentSurface::new(local.clone());
        surface.note_cache("local-cache".to_string());
        surface.note_draft("local-draft".to_string());
        surface.note_job("local-job".to_string());
        let mut gate = ResponseGate::default();
        let local_key = key(local.clone(), surface.generation());
        gate.replace(local_key.clone());
        assert!(surface.accepts(&gate, &local_key));

        surface.switch(remote.clone());
        assert!(!surface.accepts(&gate, &local_key));
        let hidden = surface.visible();
        assert!(hidden.cache.is_none());
        assert!(hidden.draft.is_none());
        assert!(hidden.job.is_none());
        surface.note_cache("ssh-cache".to_string());
        surface.note_draft("ssh-draft".to_string());
        surface.note_job("ssh-job".to_string());
        let remote_visible = surface.visible();
        assert_eq!(remote_visible.cache.as_deref(), Some("ssh-cache"));
        assert_eq!(remote_visible.draft.as_deref(), Some("ssh-draft"));
        assert_eq!(remote_visible.job.as_deref(), Some("ssh-job"));
        assert_ne!(remote_visible.cache.as_deref(), Some("local-cache"));

        surface.switch(local.clone());
        let restored = surface.visible();
        assert_eq!(restored.cache.as_deref(), Some("local-cache"));
        assert_eq!(restored.draft.as_deref(), Some("local-draft"));
        assert_eq!(restored.job.as_deref(), Some("local-job"));
        let rendered = format!("{:?}{:?}{:?}", restored.cache, restored.draft, restored.job);
        assert!(!rendered.contains("ssh-cache"));
        assert!(!rendered.contains("ssh-draft"));
        assert!(!rendered.contains("ssh-job"));
        assert!(!rendered.contains("fixture-ssh"));
    }

    #[test]
    fn non_local_read_returns_a_reason_and_does_not_open_the_source() {
        let local = EnvironmentId::Local("local".into());
        let remote = EnvironmentId::Ssh("fixture-ssh".into());
        let wsl = EnvironmentId::Wsl("fixture-wsl".into());
        let other = EnvironmentId::Local("other".into());
        assert_eq!(admit_local_only(&local), Ok(()));
        assert_eq!(admit_local_only(&remote), Err(LOCAL_ONLY));
        assert_eq!(admit_local_only(&wsl), Err(LOCAL_ONLY));

        let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
        let file = dir.path().join("source.txt");
        std::fs::write(&file, b"local-secret").unwrap_or_else(|error| panic!("{error}"));
        let calls = std::cell::Cell::new(0u32);
        let read_file = || {
            calls.set(calls.get() + 1);
            std::fs::read_to_string(&file).unwrap_or_else(|error| panic!("{error}"))
        };

        let refused = read_local_source(&remote, &local, read_file);
        assert_eq!(refused, Err(LOCAL_ONLY));
        let refused = read_local_source(&wsl, &local, read_file);
        assert_eq!(refused, Err(LOCAL_ONLY));
        let refused = read_local_source(&other, &local, read_file);
        assert_eq!(refused, Err(LOCAL_ONLY));
        assert_eq!(calls.get(), 0);
        let Err(reason) = read_local_source(&remote, &local, read_file) else {
            panic!("remote read must be refused");
        };
        assert_eq!(reason, LOCAL_ONLY);
        assert!(!reason.contains("local-secret"));
        assert!(!reason.contains("fixture-ssh"));
        assert!(!reason.contains("ssh-keyscan"));
        assert!(!reason.contains(" / "));
        assert_eq!(calls.get(), 0);
        let body = std::fs::read_to_string(&file).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(body, "local-secret");

        let loaded =
            read_local_source(&local, &local, read_file).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(calls.get(), 1);
        assert_eq!(loaded, "local-secret");
    }
}
