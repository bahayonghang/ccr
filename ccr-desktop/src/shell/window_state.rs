//! Window geometry uses GPUI's real bounds observer and the existing preference writer.
use super::Shell;
use crate::{runtime::preferences::WindowPlacement, theme};
use ccr_app::runtime::{Completion, TaskEvent};
use gpui_kit::{App, AppContext, Bounds, Context, Window, WindowBounds, point, px, size};
use std::time::Duration;

// A final save queued behind long-running jobs must not prevent the shutdown
// that cancels those jobs. After this deadline request cancellation, then still
// await the runner's terminal (an already-started atomic write may finish).
const CLOSE_SAVE_QUEUE_BUDGET: Duration = Duration::from_secs(2);

pub(super) fn placement(bounds: WindowBounds) -> WindowPlacement {
    let rect = bounds.get_bounds();
    WindowPlacement {
        x: rect.origin.x.into(),
        y: rect.origin.y.into(),
        width: rect.size.width.into(),
        height: rect.size.height.into(),
        // Fullscreen is not a launch preference. Its restore bounds are still useful.
        maximized: matches!(bounds, WindowBounds::Maximized(_)),
    }
}

pub(crate) fn restore(saved: Option<WindowPlacement>, cx: &App) -> WindowBounds {
    if let Some(saved) = saved {
        let bounds = Bounds::new(
            point(px(saved.x), px(saved.y)),
            size(
                px(saved.width.max(theme::MIN_WIDTH)),
                px(saved.height.max(theme::MIN_HEIGHT)),
            ),
        );
        // Keep a usable titlebar on a currently connected display. A disconnected
        // monitor or a fully off-screen placement falls back to the normal centre.
        if cx.displays().iter().any(|display| {
            let display = display.bounds();
            let visible_width = (bounds.right().min(display.right())
                - bounds.left().max(display.left()))
            .max(px(0.));
            visible_width >= px(100.)
                && bounds.top() >= display.top()
                && bounds.top() + px(32.) <= display.bottom()
        }) {
            return if saved.maximized {
                WindowBounds::Maximized(bounds)
            } else {
                WindowBounds::Windowed(bounds)
            };
        }
    }
    WindowBounds::centered(size(px(theme::WINDOW_WIDTH), px(theme::WINDOW_HEIGHT)), cx)
}

impl Shell {
    pub(super) fn window_bounds_changed(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.preferences_loaded || self.preferences_closing {
            return;
        }
        let current = placement(window.window_bounds());
        if self.window_placement == Some(current) {
            return;
        }
        self.window_placement = Some(current);
        let handle = window.window_handle();
        // Replacing the task cancels only the debounce timer. Admitted preference
        // jobs stay app-owned and serialize by generation in PreferencesRequests.
        self.window_state_task = Some(cx.spawn(async move |weak, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(300))
                .await;
            let _ = cx.update_window(handle, |_, window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if !this.preferences_closing {
                        this.persist_preferences(this.preference_snapshot(cx), window, cx);
                    }
                });
            });
        }));
    }

    pub(super) fn finish_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.preferences_closing {
            return;
        }
        self.preferences_closing = true;
        self.window_state_task = None;
        if !self.preferences_loaded {
            // A failed initial load must never overwrite the file with defaults.
            window.remove_window();
            return;
        }
        self.window_placement = Some(placement(window.window_bounds()));
        self.preferences_generation = self.preferences_generation.saturating_add(1);
        let subscription = self
            .preferences_requests
            .save(self.preference_snapshot(cx), self.preferences_generation);
        let handle = window.window_handle();
        match subscription {
            Ok(mut subscription) => {
                // Shutdown cancels queued work. Wait for this final write BEFORE
                // removing the last window and triggering runtime shutdown.
                self.preferences_task = Some(cx.spawn(async move |_, cx| {
                    let deadline = cx.background_executor().timer(CLOSE_SAVE_QUEUE_BUDGET);
                    tokio::pin!(deadline);
                    let mut cancellation_requested = false;
                    loop {
                        let event = tokio::select! {
                            event = subscription.next() => event,
                            _ = &mut deadline, if !cancellation_requested => {
                                // All bounded workers may be running jobs that
                                // themselves need shutdown cancellation. Do not
                                // deadlock exit behind a queued preference save.
                                subscription.request_cancel();
                                cancellation_requested = true;
                                continue;
                            }
                        };
                        let Some(event) = event else { break; };
                        if let TaskEvent::Terminal(result) = event {
                            if !matches!(result.as_ref(), Completion::Success(_)) {
                                eprintln!("Unable to save desktop window state before quitting; existing preferences were preserved");
                            }
                            break;
                        }
                    }
                    let _ = cx.update_window(handle, |_, window, _| window.remove_window());
                }));
            }
            Err(_) => {
                eprintln!("Unable to queue desktop window state before quitting");
                window.remove_window();
            }
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui_kit::test]
    fn restored_bounds_keep_maximized_and_recover_a_disconnected_display(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.update(|cx| {
            let saved = WindowPlacement {
                x: 50.,
                y: 75.,
                width: 1280.,
                height: 800.,
                maximized: true,
            };
            let bounds = restore(Some(saved), cx);
            assert!(matches!(bounds, WindowBounds::Maximized(_)));
            assert_eq!(placement(bounds), saved);
            assert_eq!(
                restore(Some(WindowPlacement { x: 50000., ..saved }), cx),
                restore(None, cx)
            );
        });
    }
}
