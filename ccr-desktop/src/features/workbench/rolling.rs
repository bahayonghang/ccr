use super::*;
use crate::runtime::platforms::rolling::{Error, Report};

impl WorkbenchView {
    pub(super) fn read_rolling(&mut self, cx: &mut Context<Self>) {
        if self.platform != Platform::Codex || self.rolling_task.is_some() {
            return;
        }
        self.rolling = None;
        self.rolling_request_id += 1;
        let generation = self.generation;
        let result = self.services.codex_rolling_usage(RequestKey {
            environment: EnvironmentId::Local("platform-workbench-rolling".into()),
            platform: Platform::Codex,
            generation,
            request_id: self.rolling_request_id,
        });
        self.rolling_task = Some(cx.spawn(async move |weak, cx| {
            let result = match result {
                Ok(mut subscription) => loop {
                    match subscription.next().await {
                        Some(TaskEvent::Terminal(completion)) => {
                            break match completion.as_ref() {
                                Completion::Success(report) => Ok(report.clone()),
                                Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                    Err(*error)
                                }
                                _ => Err(Error::Unavailable),
                            };
                        }
                        Some(_) => {}
                        None => break Err(Error::Unavailable),
                    }
                },
                Err(_) => Err(Error::Unavailable),
            };
            let _ = weak.update(cx, |this, cx| {
                if this.generation == generation && this.platform == Platform::Codex {
                    this.rolling = Some(result);
                    this.rolling_task = None;
                    cx.notify();
                }
            });
        }));
        cx.notify();
    }

    fn rolling_totals(report: &Report, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut rows = div()
            .id("workbench-rolling-totals")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP));
        for (en, zh, stats) in [
            ("5 hours", "5 小时", &report.usage.five_hour),
            ("7 days", "7 天", &report.usage.seven_day),
            ("All history", "全部历史", &report.usage.all_time),
        ] {
            let value = format!(
                "{}: {} {} · {} {} · {} {}{}",
                text(cx, en, zh),
                stats.total_input_tokens,
                text(cx, "input tokens", "输入 token"),
                stats.total_output_tokens,
                text(cx, "output tokens", "输出 token"),
                stats.total_requests,
                text(cx, "usage records", "用量记录"),
                if stats.details.partial {
                    text(cx, " (partial)", "（不完整）")
                } else {
                    ""
                }
            );
            rows = rows.child(
                div()
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_size(px(theme::BODY_SIZE))
                    .child(value),
            );
        }
        rows.into_any_element()
    }

    pub(super) fn rolling_usage(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let busy = self.rolling_task.is_some();
        let mut section = div().id("workbench-rolling").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(div().text_size(px(theme::SECTION_TITLE_SIZE)).font_weight(gpui_kit::FontWeight::SEMIBOLD).child(text(cx,"Codex local rolling usage","Codex 本地滚动用量")))
            .child(div().text_size(px(theme::META_SIZE)).text_color(cx.theme().muted_foreground).child(text(cx,"All local Codex sessions; usage records are not account quota. Reading refreshes the derived local cache.","统计本地全部 Codex 会话；用量记录不代表账户配额。读取会刷新本地派生缓存。")))
            .child(semantics::button(Button::new("workbench-rolling-read")
                .label(if busy {text(cx,"Reading sessions…","正在读取会话…")} else {text(cx,"Read rolling usage","读取滚动用量")})
                .ghost().on_click(cx.listener(|this,_,_,cx| this.read_rolling(cx))),busy,busy,cx));
        match &self.rolling {
            Some(Ok(report)) => {
                section = section
                    .child(div().text_size(px(theme::META_SIZE)).child(format!(
                        "{} {}",
                        text(cx, "As of (UTC)", "统计截止（UTC）"),
                        report.as_of.format("%Y-%m-%d %H:%M:%S")
                    )))
                    .child(Self::rolling_totals(report, cx));
                if report.usage.all_time.total_requests == 0 {
                    section = section.child(text(
                        cx,
                        "No eligible usage records found in the sessions directory.",
                        "会话目录中没有可统计的用量记录。",
                    ));
                }
                let d = &report.diagnostics;
                let diagnostic = format!(
                    "{}: {} {} · {} {} · {} {} · {} {} · {} {} · {} {} · {} {} · {} {}",
                    text(cx, "Scan", "扫描"),
                    d.files,
                    text(cx, "files", "文件"),
                    d.unreadable_files,
                    text(cx, "unreadable", "不可读"),
                    d.invalid_json,
                    text(cx, "invalid JSON", "无效 JSON"),
                    d.missing_timestamps,
                    text(cx, "missing timestamp", "缺少时间戳"),
                    d.future_records,
                    text(cx, "future records", "未来记录"),
                    d.ambiguous_records,
                    text(cx, "ambiguous", "歧义记录"),
                    d.inconsistent_records,
                    text(cx, "inconsistent", "不一致记录"),
                    d.duplicate_records,
                    text(cx, "duplicates removed", "去重记录")
                );
                section = section.child(
                    div()
                        .id("workbench-rolling-diagnostics")
                        .test_support()
                        .text_size(px(theme::META_SIZE))
                        .child(diagnostic),
                );
                if !d.is_complete() {
                    section = section.child(text(
                        cx,
                        "Partial scan; totals may be incomplete.",
                        "扫描不完整，合计可能有遗漏。",
                    ));
                }
                if d.cache_warning {
                    section = section.child(text(cx,"Derived cache was unavailable or could not be updated; the scan result is shown above.","派生缓存不可用或更新失败；上方显示本次扫描结果。"));
                }
            }
            Some(Err(error)) => {
                section = section.child(
                    div()
                        .id("workbench-rolling-error")
                        .test_support()
                        .role(gpui_kit::Role::Status)
                        .aria_label(service_error(cx, error.message()))
                        .child(service_error(cx, error.message())),
                )
            }
            None if !busy => section = section.child(text(cx, "Not read yet.", "尚未读取。")),
            None => {}
        }
        section.into_any_element()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::runtime::ConfigSource;
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{AppContext, TestAppContext, WindowOptions};
    use std::time::{Duration, Instant};

    #[gpui_kit::test]
    fn codex_rolling_action_distinguishes_missing_and_empty_sessions(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|cx| {
                    let mut view = WorkbenchView::new(services, cx);
                    view.open(Platform::Codex, cx);
                    view
                })
            })
            .unwrap()
        });
        let codex = root.path().join("desktop-platform-fixture/home/.codex");
        assert!(!codex.join(".ccr-codex-usage-cache.json").exists());
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.try_find("workbench-rolling-totals").is_none());
            window.click("workbench-rolling-read", cx);
        })
        .unwrap();
        let wait = |cx: &mut TestAppContext| {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                cx.run_until_parked();
                if cx.update(|cx| view.read(cx).rolling.is_some()) {
                    break;
                }
                assert!(Instant::now() < deadline, "rolling read timed out");
                cx.executor().advance_clock(Duration::from_millis(10));
                std::thread::yield_now();
            }
        };
        wait(cx);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(
                window
                    .find("workbench-rolling-error")
                    .label()
                    .unwrap()
                    .contains("missing")
            );
            assert!(window.try_find("workbench-rolling-totals").is_none());
        })
        .unwrap();
        std::fs::create_dir_all(codex.join("sessions")).unwrap();
        cx.update_window(handle, |_, window, cx| {
            window.click("workbench-rolling-read", cx)
        })
        .unwrap();
        wait(cx);
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.try_find("workbench-rolling-totals").is_some());
            assert!(window.try_find("workbench-rolling-error").is_none());
            view.update(cx, |view, cx| view.open(Platform::Claude, cx));
            window.render_frame(cx);
            assert!(window.try_find("workbench-rolling-read").is_none());
            window.remove_window();
        })
        .unwrap();
        assert!(codex.join(".ccr-codex-usage-cache.json").is_file());
        owner.shutdown();
    }
}
