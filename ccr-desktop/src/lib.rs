pub mod app;
pub mod features;
pub mod locales;
pub mod probe;
pub mod runtime;
pub mod shell;
pub mod theme;

use ccr_app::runtime::{Completion, TaskEvent};
use gpui_kit::{AppContext, QuitMode, TitlebarOptions, WindowOptions, px, size};

fn window_options(
    preferences: Option<runtime::preferences::DesktopPreferences>,
    cx: &gpui_kit::App,
) -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("CCR Desktop (GPUI)".into()),
            ..Default::default()
        }),
        window_bounds: Some(shell::window_state::restore(
            preferences.and_then(|value| value.window),
            cx,
        )),
        window_min_size: Some(size(px(theme::MIN_WIDTH), px(theme::MIN_HEIGHT))),
        app_id: Some("io.github.ccr.gpui".into()),
        ..Default::default()
    }
}

/// Load on the bounded worker before creating the one existing GPUI root. An
/// invalid/unreadable file still opens the app, with the preference error visible.
pub(crate) async fn open_app_window(
    services: runtime::Services,
    cx: &mut gpui_kit::AsyncApp,
) -> Result<(gpui_kit::AnyWindowHandle, gpui_kit::Entity<shell::Shell>), String> {
    let requests = runtime::preferences::PreferencesRequests::new(&services);
    let mut loaded = Err(runtime::preferences::PreferencesError::Storage);
    if let Ok(mut subscription) = requests.load() {
        while let Some(event) = subscription.next().await {
            if let TaskEvent::Terminal(result) = event {
                loaded = match result.as_ref() {
                    Completion::Success(value) => Ok(value.clone()),
                    Completion::Failed(error) => Err(*error),
                    _ => Err(runtime::preferences::PreferencesError::Storage),
                };
                break;
            }
        }
    }
    cx.update(|cx| {
        let mut options = window_options(loaded.clone().ok(), cx);
        if let Some(title) = services.benchmark_title()
            && let Some(titlebar) = &mut options.titlebar
        {
            titlebar.title = Some(title.into());
        }
        gpui_kit::open_window(options, cx, move |window, cx| {
            cx.new(|cx| shell::Shell::new_with_preferences(services, loaded, window, cx))
        })
        .map_err(|error| error.to_string())
    })
}

pub fn run() {
    if let Some(code) = runtime::jobs::fixture_child_mode(std::env::args_os().skip(1)) {
        std::process::exit(code);
    }
    let mode = match runtime::LaunchMode::parse(std::env::args_os().skip(1)) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let owner = match ccr_app::runtime::RuntimeOwner::new() {
        Ok(owner) => owner,
        Err(_) => {
            eprintln!("Unable to start the application runtime");
            std::process::exit(1);
        }
    };
    let executor = owner.handle();
    let failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let window_failed = failed.clone();
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            theme::init(cx);
            cx.set_app_identity("io.github.ccr.gpui", "CCR Desktop (GPUI)");
            cx.set_quit_mode(QuitMode::LastWindowClosed);
            let shutdown = executor.clone();
            cx.on_app_quit(move |_| {
                let shutdown = shutdown.clone();
                async move {
                    shutdown.shutdown().await;
                }
            })
            .detach();
            let services = match mode {
                runtime::LaunchMode::Benchmark(root) => {
                    cx.spawn(async move |cx| {
                        let result = match runtime::benchmark::load_services(executor, root).await {
                            Ok(services) => open_app_window(services, cx)
                                .await
                                .map(|_| ())
                                .map_err(|_| "Benchmark window cannot open"),
                            Err(error) => Err(error),
                        };
                        if let Err(error) = result {
                            window_failed.store(true, std::sync::atomic::Ordering::Relaxed);
                            eprintln!("Unable to open CCR benchmark: {error}");
                            cx.update(|cx| cx.quit());
                        } else {
                            cx.update(|cx| cx.activate(true));
                        }
                    })
                    .detach();
                    return;
                }
                runtime::LaunchMode::Probe => {
                    let options = window_options(None, cx);
                    if let Err(error) = gpui_kit::open_window(options, cx, |window, cx| {
                        cx.new(|cx| probe::Probe::new(window, cx))
                    }) {
                        window_failed.store(true, std::sync::atomic::Ordering::Relaxed);
                        eprintln!("Unable to open CCR Desktop (GPUI): {error}");
                        cx.quit();
                    }
                    cx.activate(true);
                    return;
                }
                runtime::LaunchMode::App(source) => {
                    let mut services = runtime::Services::new(executor, source);
                    if !services.is_fixture()
                        && let Some(paths) = runtime::insights::local_insights_paths()
                    {
                        services = services.with_insights_paths(paths);
                    }
                    services
                }
                runtime::LaunchMode::AppWithInsights { source, paths } => {
                    runtime::Services::new(executor, source).with_insights_paths(paths)
                }
            };
            cx.spawn(async move |cx| {
                if let Err(error) = open_app_window(services, cx).await {
                    window_failed.store(true, std::sync::atomic::Ordering::Relaxed);
                    eprintln!("Unable to open CCR Desktop (GPUI): {error}");
                    cx.update(|cx| cx.quit());
                } else {
                    cx.update(|cx| cx.activate(true));
                }
            })
            .detach();
        });
    // GPUI quit observers have a 200ms platform budget. The owner remains alive
    // outside the event loop and joins all started work before process return.
    owner.shutdown();
    if failed.load(std::sync::atomic::Ordering::Relaxed) {
        std::process::exit(1);
    }
}
