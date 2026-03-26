use std::fs;
use std::sync::{Arc, Mutex};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Default)]
pub struct SkillsWatcherState {
    inner: Arc<Mutex<Option<RecommendedWatcher>>>,
}

impl SkillsWatcherState {
    fn replace(&self, watcher: RecommendedWatcher) {
        if let Ok(mut slot) = self.inner.lock() {
            *slot = Some(watcher);
        }
    }
}

pub fn reload(app_handle: &AppHandle) -> Result<(), String> {
    let service = ccr::SkillsService::new().map_err(|error| error.to_string())?;
    let paths = service.watch_paths().map_err(|error| error.to_string())?;

    let emit_handle = app_handle.clone();
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| {
            let Ok(event) = result else {
                return;
            };

            let should_emit = matches!(
                event.kind,
                EventKind::Any
                    | EventKind::Create(_)
                    | EventKind::Modify(_)
                    | EventKind::Remove(_)
            );
            if !should_emit {
                return;
            }

            let payload = serde_json::json!({
                "paths": event
                    .paths
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect::<Vec<_>>(),
            });
            let _ = emit_handle.emit("skills-changed", payload);
        },
        Config::default(),
    )
    .map_err(|error| error.to_string())?;

    for path in paths {
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        if path.exists() {
            watcher
                .watch(&path, RecursiveMode::Recursive)
                .map_err(|error| error.to_string())?;
        }
    }

    let state = app_handle.state::<SkillsWatcherState>();
    state.replace(watcher);
    Ok(())
}
