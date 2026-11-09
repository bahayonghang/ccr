/// Whether a profile edit rewrites the platform runtime.
///
/// Public update entry points used by Tauri and the desktop app keep
/// [`ApplyPolicy::IfRenamed`]. The shared edit use case uses
/// [`ApplyPolicy::IfCurrent`] or [`ApplyPolicy::Never`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyPolicy {
    /// Re-apply when the edited profile is the current one, including a same-name edit.
    IfCurrent,
    /// Change stored profile data only.
    Never,
    /// Re-apply only when renaming the profile that is current.
    IfRenamed,
}
