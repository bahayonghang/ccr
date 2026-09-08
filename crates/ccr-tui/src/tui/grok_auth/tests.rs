#![allow(clippy::unwrap_used)]
use super::*;
use ccr_cli::services::grok_auth_service::{GrokAuthAccount, GrokAuthSource};
use crossterm::event::KeyModifiers;
use ratatui::{Terminal, backend::TestBackend};
use std::sync::atomic::{AtomicUsize, Ordering};

struct FakeBackend(AtomicUsize);

impl Backend for FakeBackend {
    fn read(&self) -> std::result::Result<Loaded, String> {
        Ok(Loaded {
            snapshot: GrokAuthSnapshot::default(),
            activation: None,
        })
    }

    fn execute(&self, _: &Request) -> std::result::Result<Vec<String>, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    }
}

fn fixture() -> GrokAuthApp {
    let mut app = GrokAuthApp::empty(
        AsyncTaskExecutor::Disabled,
        Arc::new(FakeBackend(AtomicUsize::new(0))),
    );
    app.initialized = true;
    app.snapshot.accounts = vec![
        GrokAuthAccount {
            name: "personal".into(),
            local_match: true,
            ..Default::default()
        },
        GrokAuthAccount {
            name: "团队工作账号".into(),
            expired: true,
            ..Default::default()
        },
    ];
    app.snapshot.sources = vec![GrokAuthSource {
        scope: "official-scope".into(),
        ..Default::default()
    }];
    app
}

fn press(app: &mut GrokAuthApp, code: KeyCode) -> bool {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
        .unwrap()
}

#[test]
fn grok_selection_is_distinct_from_local_match_and_survives_reordering() {
    let mut app = fixture();
    press(&mut app, KeyCode::Down);
    assert_eq!(app.selected_name(), Some("团队工作账号"));
    assert!(app.snapshot.accounts[0].local_match);
    let mut snapshot = app.snapshot.clone();
    snapshot.accounts.reverse();
    app.consume(Response {
        action: None,
        outcome: Ok(Vec::new()),
        loaded: Ok(Loaded {
            snapshot,
            activation: None,
        }),
    });
    assert_eq!(app.selected_name(), Some("团队工作账号"));
    assert_eq!(app.selected, 0);
}

#[test]
fn grok_confirm_defaults_to_cancel_and_disabled_executor_never_becomes_busy() {
    let mut app = fixture();
    for action in [
        CompletedAction::Switch,
        CompletedAction::Delete,
        CompletedAction::GrokLogout,
        CompletedAction::Save,
    ] {
        app.modal = Some(Modal::Confirm(app.request(action, "personal".into())));
        press(&mut app, KeyCode::Enter);
        assert!(app.modal.is_none());
        assert!(!app.is_busy());
        assert!(app.last_action.is_none());
    }
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('y'));
    assert!(!app.is_busy());
    assert_eq!(app.notice, Some("executor"));
}

#[test]
fn grok_save_requires_explicit_overwrite_and_navigation_cancels_input() {
    let mut app = fixture();
    press(&mut app, KeyCode::Char('s'));
    for c in "personal".chars() {
        press(&mut app, KeyCode::Char(c));
    }
    press(&mut app, KeyCode::Enter);
    assert!(matches!(
        app.modal,
        Some(Modal::Confirm(Request { replace: true, .. }))
    ));
    app.on_deactivated();
    assert!(app.modal.is_none());
    assert!(app.last_action.is_none());
}

#[test]
fn grok_success_survives_refresh_error_and_busy_rejects_quit() {
    let mut app = fixture();
    let request = app.request(CompletedAction::Save, "personal".into());
    app.consume(Response {
        action: Some(request),
        outcome: Ok(Vec::new()),
        loaded: Err("fixture read failure".into()),
    });
    assert!(app.stale);
    assert_eq!(app.snapshot.accounts.len(), 2);
    assert!(app.last_action.as_ref().unwrap().2);
    let (_tx, rx) = mpsc::channel();
    app.pending = Some(rx);
    assert!(!press(&mut app, KeyCode::Char('q')));
    assert!(!press(&mut app, KeyCode::Enter));
    assert!(app.modal.is_none());
}

#[test]
fn grok_disconnected_task_recovers_without_mutation_replay() {
    let mut app = fixture();
    app.last_action = Some((CompletedAction::Save, "older".into(), true, None));
    app.warnings.push("older warning".into());
    let (tx, rx) = mpsc::channel();
    app.pending = Some(rx);
    drop(tx);
    assert!(app.on_tick());
    assert!(!app.is_busy());
    assert_eq!(app.notice, Some("unknown"));
    assert!(app.last_action.is_none());
    assert!(app.operation_warnings().is_empty());
}

#[test]
fn grok_frames_keep_actions_and_local_evidence_across_sizes_and_languages() {
    use crate::tui::i18n::{active_language, set_language};
    use ccr_cli::managers::TuiLanguage;
    let original = active_language();
    for language in [TuiLanguage::English, TuiLanguage::SimplifiedChinese] {
        set_language(language);
        for (width, height) in [(140, 40), (100, 30), (80, 24), (40, 12)] {
            for state in 0..6 {
                let mut app = fixture();
                match state {
                    1 => app.snapshot.accounts.clear(),
                    2 => {
                        app.stale = true;
                        app.error = Some("fixture read failure".into());
                    }
                    3 => {
                        app.modal = Some(Modal::Confirm(
                            app.request(CompletedAction::GrokLogout, String::new()),
                        ))
                    }
                    4 => app.save(),
                    5 => {
                        app.modal = Some(Modal::Source {
                            selected: 0,
                            revision: Default::default(),
                        })
                    }
                    _ => {}
                }
                let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                terminal
                    .draw(|frame| {
                        let mode = crate::tui::theme::viewport_mode(width, height);
                        let footer_height = crate::tui::theme::footer_height(mode);
                        super::super::ui::draw_embedded(
                            frame,
                            &mut app,
                            Rect::new(0, 3, width, height - 3 - footer_height),
                            Rect::new(0, height - footer_height, width, footer_height),
                            mode,
                        );
                    })
                    .unwrap();
                let text: String = terminal
                    .backend()
                    .buffer()
                    .content
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect();
                if state < 3 {
                    assert!(text.contains("q "));
                }
                if state == 3 {
                    assert!(text.contains("Enter/n/Esc"));
                    assert!(text.contains(crate::tui_text!("CCR saved accounts", "CCR 保存账号")));
                    assert!(text.contains(crate::tui_text!("Unsaved", "未保存")));
                }
                if state == 0 {
                    assert!(text.contains("personal"));
                }
            }
        }
    }
    set_language(original);
}

#[test]
fn grok_submits_once_and_collects_background_result() {
    use std::sync::Mutex;
    use std::time::Duration;
    struct BlockingBackend {
        calls: AtomicUsize,
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl Backend for BlockingBackend {
        fn read(&self) -> std::result::Result<Loaded, String> {
            Ok(Loaded {
                snapshot: Default::default(),
                activation: None,
            })
        }
        fn execute(&self, _: &Request) -> std::result::Result<Vec<String>, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.entered.send(()).unwrap();
            self.release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
            Ok(vec!["fixture durability warning".into()])
        }
    }
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let backend = Arc::new(BlockingBackend {
        calls: AtomicUsize::new(0),
        entered: entered_tx,
        release: Mutex::new(release_rx),
    });
    let mut app = GrokAuthApp::empty(AsyncTaskExecutor::from_current_or_test(), backend.clone());
    let request = app.request(CompletedAction::Save, "saved".into());
    app.submit(Some(request.clone()));
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    app.submit(Some(request));
    assert!(!press(&mut app, KeyCode::Char('q')));
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
    release_tx.send(()).unwrap();
    let response = app
        .pending
        .take()
        .unwrap()
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    app.consume(response);
    assert!(app.last_action.as_ref().unwrap().2);
    assert_eq!(app.operation_warnings(), &["fixture durability warning"]);
    assert!(!app.is_busy());
}

#[test]
fn grok_compact_switch_and_stale_success_remain_explicit() {
    let original = crate::tui::i18n::active_language();
    crate::tui::i18n::set_language(ccr_cli::managers::TuiLanguage::English);
    let mut app = fixture();
    app.modal = Some(Modal::Confirm(
        app.request(CompletedAction::Switch, "personal".into()),
    ));
    let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
    let render = |terminal: &mut Terminal<TestBackend>, app: &mut GrokAuthApp| {
        terminal
            .draw(|f| {
                super::super::ui::draw_embedded(
                    f,
                    app,
                    Rect::new(0, 3, 40, 7),
                    Rect::new(0, 10, 40, 2),
                    crate::tui::theme::ViewportMode::Compact,
                )
            })
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|c| c.symbol())
            .collect::<String>()
    };
    let text = render(&mut terminal, &mut app);
    assert!(text.contains("Stop Grok BEFORE confirming"));
    assert!(text.contains("NEW sessions; auth unverified"));
    assert!(text.contains("Default: cancel"));
    app.modal = None;
    app.stale = true;
    app.last_action = Some((CompletedAction::Save, "personal".into(), true, None));
    let text = render(&mut terminal, &mut app);
    assert!(text.contains("Action succeeded; refresh failed"));
    assert!(text.contains("Stale; refresh before retry"));
    app.last_action = Some((
        CompletedAction::Save,
        "personal".into(),
        false,
        Some("fixture failure".into()),
    ));
    let text = render(&mut terminal, &mut app);
    assert!(text.contains("Action failed; refresh failed"));
    crate::tui::i18n::set_language(original);
}

#[test]
fn grok_compact_source_last_row_and_maximum_alias_cursor_stay_visible() {
    let mut app = fixture();
    app.snapshot.sources = (0..4)
        .map(|i| GrokAuthSource {
            scope: format!("source-{i}"),
            ..Default::default()
        })
        .collect();
    app.modal = Some(Modal::Source {
        selected: 3,
        revision: Default::default(),
    });
    let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
    terminal
        .draw(|f| {
            super::super::ui::draw_embedded(
                f,
                &mut app,
                Rect::new(0, 3, 40, 7),
                Rect::new(0, 10, 40, 2),
                crate::tui::theme::ViewportMode::Compact,
            )
        })
        .unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(text.contains("› ? source-3"));
    app.modal = Some(Modal::Name {
        value: "a".repeat(32),
        scope: "source-3".into(),
        revision: Default::default(),
    });
    terminal
        .draw(|f| {
            super::super::ui::draw_embedded(
                f,
                &mut app,
                Rect::new(0, 3, 40, 7),
                Rect::new(0, 10, 40, 2),
                crate::tui::theme::ViewportMode::Compact,
            )
        })
        .unwrap();
    let text: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(text.contains(&format!("{}▏", "a".repeat(32))));
}
