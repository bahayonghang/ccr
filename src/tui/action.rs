// TUI Action enum — decouples key-mapping from side-effect execution (Elm-like pattern)

/// Represents all possible user-initiated state transitions in the TUI.
#[derive(Debug, Clone)]
pub enum Action {
    /// No-op — ignore this event
    Noop,
    /// Exit the TUI
    Quit,
    /// Switch to the tab at the given index (wraps on overflow)
    #[allow(dead_code)]
    SwitchTab(usize),
    /// Cycle to the next tab
    NextTab,
    /// Move selection cursor up
    SelectPrev,
    /// Move selection cursor down
    SelectNext,
    /// Go to previous page
    PrevPage,
    /// Go to next page
    NextPage,
    /// Apply the currently selected profile (stay in TUI)
    ApplySelected,
    /// Apply the currently selected profile and quit
    ApplyAndQuit,
    /// Refresh data from disk
    Reload,
}
