pub mod app;
pub mod features;
pub mod locales;
pub mod probe;
pub mod runtime;
pub mod shell;
pub mod theme;

use gpui_kit::{
    AppContext, Bounds, QuitMode, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};

pub fn run() {
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
            cx.set_quit_mode(QuitMode::LastWindowClosed);
            let shutdown = executor.clone();
            cx.on_app_quit(move |_| {
                let shutdown = shutdown.clone();
                async move {
                    shutdown.shutdown().await;
                }
            })
            .detach();
            let options = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("CCR Desktop (GPUI)".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(theme::WINDOW_WIDTH), px(theme::WINDOW_HEIGHT)),
                    cx,
                ))),
                window_min_size: Some(size(px(theme::MIN_WIDTH), px(theme::MIN_HEIGHT))),
                app_id: Some("io.github.ccr.gpui".into()),
                ..Default::default()
            };
            let result = match mode {
                runtime::LaunchMode::Probe => gpui_kit::open_window(options, cx, |window, cx| {
                    cx.new(|cx| probe::Probe::new(window, cx))
                })
                .map(|_| ()),
                runtime::LaunchMode::App(source) => {
                    gpui_kit::open_window(options, cx, move |window, cx| {
                        cx.new(|cx| {
                            shell::Shell::new(runtime::Services::new(executor, source), window, cx)
                        })
                    })
                    .map(|_| ())
                }
            };
            if let Err(error) = result {
                window_failed.store(true, std::sync::atomic::Ordering::Relaxed);
                eprintln!("Unable to open CCR Desktop (GPUI): {error}");
                cx.quit();
            }
            cx.activate(true);
        });
    // GPUI quit observers have a 200ms platform budget. The owner remains alive
    // outside the event loop and joins all started work before process return.
    owner.shutdown();
    if failed.load(std::sync::atomic::Ordering::Relaxed) {
        std::process::exit(1);
    }
}
