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
    /// Cycle to the previous tab
    PrevTab,
    /// Move selection cursor up
    SelectPrev,
    /// Move selection cursor down
    SelectNext,
    /// Jump selection to a specific index (e.g. mouse click)
    SelectAt(usize),
    /// Go to previous page
    PrevPage,
    /// Go to next page
    NextPage,
    /// Scroll the selected profile details upward
    ScrollDetailsUp,
    /// Scroll the selected profile details downward
    ScrollDetailsDown,
    /// Apply the currently selected profile (stay in TUI)
    ApplySelected,
    /// Toggle visibility of unset profile detail fields (collapsed summary vs full list)
    ToggleDetailsExpanded,
    /// Exit profile mode and clear CCR login leftovers
    ProfileOff,
    /// Refresh data from disk
    Reload,
}
