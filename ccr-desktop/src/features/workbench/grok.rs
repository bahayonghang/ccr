use super::*;
use ccr_cli::platforms::{GrokActivationState, GrokProfileAuthMode};

impl WorkbenchView {
    pub(super) fn grok_overview(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut section = div()
            .id("workbench-grok-overview")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .text_size(px(theme::SECTION_TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Grok configuration state", "Grok 配置状态")),
            );
        let report = match &self.grok {
            Some(Ok(report)) => report,
            Some(Err(error)) => {
                return section
                    .child(
                        div()
                            .id("workbench-grok-error")
                            .role(gpui_kit::Role::Status)
                            .child(service_error(cx, error)),
                    )
                    .into_any_element();
            }
            None => {
                return section
                    .child(text(
                        cx,
                        "Reading Grok activation…",
                        "正在读取 Grok 激活状态…",
                    ))
                    .into_any_element();
            }
        };
        let (status, name, warning) = match &report.activation {
            GrokActivationState::Inactive => (
                text(cx, "Profile mode is off", "配置模式已关闭"),
                None,
                false,
            ),
            GrokActivationState::Active { name } => (
                text(
                    cx,
                    "Profile matches runtime configuration",
                    "配置与运行文件一致",
                ),
                Some(name),
                false,
            ),
            GrokActivationState::Drifted { name } => (
                text(
                    cx,
                    "Runtime configuration has drifted from the recorded profile",
                    "运行文件与已记录配置不一致",
                ),
                Some(name),
                true,
            ),
            GrokActivationState::UnsafeMissingEntryState { name } => (
                text(
                    cx,
                    "Profile restoration baseline is missing",
                    "配置恢复基线缺失",
                ),
                name.as_ref(),
                true,
            ),
        };
        section = section.child(
            div()
                .id("workbench-grok-activation")
                .test_support()
                .role(gpui_kit::Role::Status)
                .aria_label(status)
                .text_color(if warning {
                    cx.theme().warning
                } else {
                    cx.theme().foreground
                })
                .child(status),
        );
        if let Some(name) = name {
            section = section.child(format!(
                "{}: {name}",
                text(cx, "Recorded profile", "已记录配置")
            ));
        }
        if warning {
            section = section.child(text(cx,"Review Profiles before switching or turning profile mode off. This overview does not repair files.","请先在 Profiles 检查，再切换或关闭配置模式。概览不会修复文件。"));
        }
        let mode = match report.auth_mode {
            Some(GrokProfileAuthMode::InlineApiKey) => text(cx, "Inline API key", "内联 API key"),
            Some(GrokProfileAuthMode::EnvKey) => text(cx, "Environment variable", "环境变量"),
            Some(GrokProfileAuthMode::Session) => text(cx, "Grok session", "Grok 会话"),
            None => text(
                cx,
                "No recorded profile credential mode",
                "没有已记录配置的凭据形态",
            ),
        };
        section
            .child(format!(
                "{}: {mode}",
                text(cx, "Profile credential source", "配置凭据来源")
            ))
            .child(
                div()
                    .font_family(cx.theme().mono_font_family.clone())
                    .child(format!(
                        "{}: {} / {} {}",
                        text(cx, "Enabled profiles", "已启用配置"),
                        report.profiles_enabled,
                        report.profiles_total,
                        text(cx, "total", "总计")
                    )),
            )
            .child(if report.config_exists {
                text(cx, "Runtime configuration file exists", "运行配置文件存在")
            } else {
                text(
                    cx,
                    "Runtime configuration file is missing",
                    "运行配置文件不存在",
                )
            })
            .into_any_element()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::runtime::{ConfigSource, profiles};
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{AppContext, TestAppContext, WindowOptions};
    use std::time::{Duration, Instant};

    #[gpui_kit::test]
    fn grok_workbench_reports_missing_baseline_and_clears_on_platform_change(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        };
        let context = profiles::context(&source, ccr_config::Platform::Grok).unwrap();
        context
            .run(|| {
                let mut config = ccr_config::CcsConfig {
                    default_config: "official".into(),
                    current_config: "official".into(),
                    settings: Default::default(),
                    sections: Default::default(),
                };
                config
                    .sections
                    .insert("official".into(), Default::default());
                context.manager().save(&config)
            })
            .unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(owner.handle(), source);
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|cx| {
                    let mut view = WorkbenchView::new(services, cx);
                    view.open(Platform::Grok, cx);
                    view
                })
            })
            .unwrap()
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            cx.run_until_parked();
            if cx.update(|cx| view.read(cx).grok.is_some()) {
                break;
            }
            assert!(Instant::now() < deadline, "Grok overview timeout");
            cx.executor().advance_clock(Duration::from_millis(10));
            std::thread::yield_now();
        }
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(
                window
                    .find("workbench-grok-activation")
                    .label()
                    .unwrap()
                    .contains("baseline")
            );
            view.update(cx, |view, cx| view.open(Platform::Claude, cx));
            window.render_frame(cx);
            assert!(window.try_find("workbench-grok-overview").is_none());
            assert!(view.read(cx).grok.is_none());
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }
}
