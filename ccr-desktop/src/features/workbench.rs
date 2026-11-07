//! Platform home reads captured domain owners; opening it never writes or logs in.
use crate::{
    locales::{service_error, text},
    runtime::{
        Services,
        insights::{InsightsQuery, InsightsRequests, InsightsSnapshot},
        platforms::{Command, CredentialState, ResultValue, Surface},
    },
    shell::semantics,
    theme,
};
use ccr_app::{
    config::ConfigRow,
    runtime::{Completion, EnvironmentId, Platform, RequestKey, TaskEvent},
};
use ccr_usage::{QueryFilter, SourceKind};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Task, Window, div, prelude::FluentBuilder, px,
};
use std::sync::Arc;

mod claude;
mod claude_trend;
mod grok;
mod rolling;

#[derive(Clone, Copy)]
pub enum Link {
    Profiles,
    Auth,
    Settings,
    Mcp,
    Files,
    Agents,
    Plugins,
    Commands,
    Usage,
    Observer,
    Sessions,
}

pub fn name(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => "Claude Code",
        Platform::Codex => "Codex",
        Platform::Grok => "Grok",
        Platform::Gemini => "Antigravity",
        Platform::OpenCode => "OpenCode",
        _ => "Platform",
    }
}

pub fn source(platform: Platform) -> Option<SourceKind> {
    SourceKind::parse_id(match platform {
        Platform::Claude => "claude",
        Platform::Codex => "codex",
        Platform::Grok => "grok",
        Platform::Gemini => "antigravity",
        Platform::OpenCode => "opencode",
        _ => return None,
    })
}

pub struct WorkbenchView {
    services: Services,
    requests: InsightsRequests,
    platform: Platform,
    generation: u64,
    task: Option<Task<()>>,
    rolling_task: Option<Task<()>>,
    rolling: Option<
        Result<
            crate::runtime::platforms::rolling::Report,
            crate::runtime::platforms::rolling::Error,
        >,
    >,
    rolling_request_id: u64,
    grok: Option<Result<crate::runtime::profiles::overview::GrokOverview, &'static str>>,
    profiles: Option<Result<Arc<Vec<ConfigRow>>, &'static str>>,
    credential: Option<Result<CredentialState, &'static str>>,
    facts: Vec<(String, String)>,
    usage_until: chrono::NaiveDate,
    usage: Option<Result<Arc<InsightsSnapshot>, &'static str>>,
    observer: Option<crate::runtime::insights::observer::Report>,
    observer_error: Option<&'static str>,
    pending: Option<Link>,
    focus: FocusHandle,
}

impl WorkbenchView {
    pub fn new(services: Services, cx: &mut Context<Self>) -> Self {
        Self {
            requests: InsightsRequests::new(&services),
            usage_until: services.report_clock().date(),
            services,
            platform: Platform::Claude,
            generation: 0,
            task: None,
            rolling_task: None,
            rolling: None,
            rolling_request_id: 0,
            grok: None,
            profiles: None,
            credential: None,
            facts: Vec::new(),
            usage: None,
            observer: None,
            observer_error: None,
            pending: None,
            focus: cx.focus_handle().tab_stop(false),
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.rolling_task = None;
        self.rolling = None;
        self.grok = None;
        self.generation += 1;
        self.pending = None;
        cx.notify();
    }
    pub fn take_pending(&mut self) -> Option<Link> {
        self.pending.take()
    }
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn open(&mut self, platform: Platform, cx: &mut Context<Self>) {
        self.suspend(cx);
        self.platform = platform;
        self.profiles = None;
        self.credential = None;
        self.usage = None;
        self.observer = None;
        self.observer_error = None;
        self.facts.clear();
        let generation = self.generation;
        let key = |id: &str| RequestKey {
            environment: EnvironmentId::Local(format!("platform-workbench-{id}")),
            platform,
            generation,
            request_id: generation,
        };
        let profiles = matches!(
            platform,
            Platform::Claude | Platform::Codex | Platform::Grok
        )
        .then(|| self.services.profile_rows(key("profiles"), String::new()));
        let surface = match platform {
            Platform::Gemini => Surface::LegacySettings,
            Platform::OpenCode => Surface::Settings,
            _ => Surface::Auth,
        };
        let settings = self
            .services
            .platform_request(key("settings"), Command::Read(surface));
        let clock = if platform == Platform::Claude {
            self.requests.report_clock()
        } else {
            ccr_app::usage::report_clock::ReportClock::live()
        };
        let (since, until) = clock.window(30);
        self.usage_until = until;
        let usage = self.requests.load(
            key("usage"),
            InsightsQuery::Usage(QueryFilter {
                source: source(platform),
                since: Some(since),
                until: Some(until),
                timezone: clock.timezone,
                ..Default::default()
            }),
        );
        let observer = (platform == Platform::Claude)
            .then(|| self.requests.observer_read(key("observer"), 30, false));
        let grok = (platform == Platform::Grok).then(|| self.services.grok_overview(key("grok")));
        self.task = Some(cx.spawn(async move |weak, cx| {
            if let Some(request) = grok {
                let result = match request {
                    Ok(mut sub) => loop {
                        match sub.next().await {
                            Some(TaskEvent::Terminal(value)) => break match value.as_ref() {
                                Completion::Success(report) => Ok(report.clone()),
                                Completion::Failed(error) | Completion::CleanupFailed(error) => Err(error.message()),
                                _ => Err("Grok overview read interrupted. Retry. / Grok 概览读取已中断，请重试"),
                            },
                            Some(_) => {},
                            None => break Err("Grok overview read interrupted. Retry. / Grok 概览读取已中断，请重试"),
                        }
                    },
                    Err(_) => Err("Read queue busy. Retry. / 读取队列繁忙，请重试"),
                };
                let _ = weak.update(cx, |this, cx| {
                    if this.generation == generation { this.grok = Some(result); cx.notify(); }
                });
            }
            if let Some(profiles) = profiles {
                let result = match profiles {
                    Ok(mut sub) => loop { match sub.next().await {
                        Some(TaskEvent::Terminal(value)) => break match value.as_ref() {
                            Completion::Success(rows) => Ok(rows.clone()),
                            Completion::Failed(error) | Completion::CleanupFailed(error) => Err(error.message()),
                            _ => Err("Profile read interrupted. Retry. / 配置读取已中断，请重试"),
                        }, Some(_) => {}, None => break Err("Profile read interrupted. Retry. / 配置读取已中断，请重试")
                    } }, Err(_) => Err("Read queue busy. Retry. / 读取队列繁忙，请重试")
                };
                let _ = weak.update(cx, |this, cx| { if this.generation == generation { this.profiles = Some(result); cx.notify(); } });
            }
            let (credential, facts) = match settings {
                Ok(mut sub) => loop { match sub.next().await {
                    Some(TaskEvent::Terminal(value)) => break match value.as_ref() {
                        Completion::Success(ResultValue::Loaded(session)) => (Ok(session.credential), session.fields.iter().map(|field| (field.key.to_string(), field.value.clone())).collect()),
                        Completion::Failed(error) | Completion::CleanupFailed(error) => (Err(error.message()), Vec::new()),
                        _ => (Err("Settings read interrupted. Retry. / 设置读取已中断，请重试"), Vec::new()),
                    }, Some(_) => {}, None => break (Err("Settings read interrupted. Retry. / 设置读取已中断，请重试"), Vec::new())
                } }, Err(_) => (Err("Read queue busy. Retry. / 读取队列繁忙，请重试"), Vec::new())
            };
            let _ = weak.update(cx, |this, cx| { if this.generation == generation { this.credential = Some(credential); this.facts = facts; cx.notify(); } });
            let result = match usage {
                Ok(mut sub) => loop { match sub.next().await {
                    Some(TaskEvent::Terminal(value)) => break match value.as_ref() {
                        Completion::Success(value) => Ok(value.clone()),
                        Completion::Failed(error) | Completion::CleanupFailed(error) => Err(error.message()),
                        _ => Err("Usage read interrupted. Retry. / 用量读取已中断，请重试"),
                    }, Some(_) => {}, None => break Err("Usage read interrupted. Retry. / 用量读取已中断，请重试")
                } }, Err(_) => Err("Read queue busy. Retry. / 读取队列繁忙，请重试")
            };
            let _ = weak.update(cx, |this, cx| { if this.generation == generation { this.usage = Some(result); cx.notify(); } });
            if let Some(observer) = observer {
                let result = match observer {
                    Ok(mut sub) => loop { match sub.next().await {
                        Some(TaskEvent::Terminal(value)) => break match value.as_ref() {
                            Completion::Success(report) => Ok(report.clone()),
                            _ => Err("Claude insight and subscription read failed. Retry. / Claude 洞察和订阅读取失败，请重试"),
                        }, Some(_) => {}, None => break Err("Claude insight read interrupted. Retry. / Claude 洞察读取中断，请重试")
                    } }, Err(_) => Err("Read queue busy. Retry. / 读取队列繁忙，请重试")
                };
                let _ = weak.update(cx,|this,cx| { if this.generation==generation { match result { Ok(report)=>this.observer=Some(report),Err(error)=>this.observer_error=Some(error) }; cx.notify(); } });
            }
        }));
        cx.notify();
    }
    fn button(
        &self,
        id: &'static str,
        link: Link,
        en: &'static str,
        zh: &'static str,
        primary: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let button = Button::new(id)
            .label(text(cx, en, zh))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.pending = Some(link);
                cx.notify();
            }));
        semantics::button(
            if primary {
                button.primary()
            } else if self.platform == Platform::Claude {
                button
            } else {
                button.ghost()
            },
            false,
            false,
            cx,
        )
        .into_any_element()
    }
    fn current_profile(&self, cx: &Context<Self>) -> gpui_kit::AnyElement {
        let profile: String = match &self.profiles {
            Some(Ok(rows)) => rows
                .iter()
                .find(|row| row.is_current)
                .map(|row| format!("{}  {}", text(cx, "Current profile", "当前配置"), row.name))
                .unwrap_or_else(|| text(cx, "No active profile", "没有正在使用的配置").into()),
            Some(Err(error)) => service_error(cx, error).to_string(),
            None if matches!(
                self.platform,
                Platform::Claude | Platform::Codex | Platform::Grok
            ) =>
            {
                text(cx, "Reading profiles…", "正在读取配置…").into()
            }
            _ => text(cx, "Local platform settings", "本地平台设置").into(),
        };
        div()
            .id("workbench-current-profile")
            .test_support()
            .role(gpui_kit::Role::Label)
            .aria_label(profile.clone())
            .aria_value(profile.clone())
            .min_w_0()
            .max_w_full()
            .text_size(px(theme::SECTION_TITLE_SIZE))
            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
            .child(profile)
            .into_any_element()
    }
    fn credential_status(&self, cx: &Context<Self>) -> gpui_kit::AnyElement {
        let credential = match &self.credential {
            None => text(
                cx,
                "Reading local credential state…",
                "正在读取本地凭据状态…",
            ),
            Some(Err(error)) => service_error(cx, error),
            Some(Ok(state)) => match state {
                CredentialState::Detected => text(
                    cx,
                    "Local credentials detected; remote authentication is unverified.",
                    "已检测到本地凭据；尚未验证远端认证。",
                ),
                CredentialState::NotDetected => {
                    text(cx, "No local credentials detected.", "未检测到本地凭据。")
                }
                CredentialState::Invalid => text(
                    cx,
                    "Local credential source is invalid.",
                    "本地凭据来源无效。",
                ),
                CredentialState::Unreadable => text(
                    cx,
                    "Cannot read local credential source.",
                    "无法读取本地凭据来源。",
                ),
                CredentialState::PlatformUnsupported | CredentialState::StoreUnsupported => text(
                    cx,
                    "Credential observation is unavailable for this platform.",
                    "此平台不支持凭据观测。",
                ),
            },
        };
        div()
            .id("workbench-credential")
            .test_support()
            .role(gpui_kit::Role::Status)
            .aria_label(credential)
            .aria_value(credential)
            .min_w_0()
            .max_w_full()
            .text_color(cx.theme().muted_foreground)
            .child(credential)
            .into_any_element()
    }
    fn current(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut section = div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .py(px(theme::SECTION_GAP))
            .border_t_1()
            .border_b_1()
            .border_color(cx.theme().border);
        section = section
            .child(self.current_profile(cx))
            .child(self.credential_status(cx));
        for (key, value) in self.facts.iter().take(4) {
            section = section.child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .child(format!("{key}: {value}")),
            );
        }
        section.into_any_element()
    }
    fn usage(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut section = div().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(div().text_size(px(theme::SECTION_TITLE_SIZE)).font_weight(gpui_kit::FontWeight::SEMIBOLD).child(text(cx,"Usage · last 30 days","用量 · 最近 30 天")))
            .child(div().text_size(px(theme::META_SIZE)).text_color(cx.theme().muted_foreground).child(text(cx,"Local calendar days. Costs recorded by llmusage (USD); missing data is not zero.","按本地日历日统计。费用为 llmusage 记录的美元费用；缺失数据不记为零。")));
        match &self.usage {
            Some(Ok(snapshot)) => {
                if let InsightsSnapshot::Usage(usage) = snapshot.as_ref() {
                    section = section.child(
                        div()
                            .id("workbench-usage-totals")
                            .test_support()
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_size(px(theme::TITLE_SIZE))
                            .child(format!(
                                "{} {}    {} tokens    ${:.4}",
                                usage.overview.total_events,
                                text(cx, "requests", "请求"),
                                usage.overview.total.total_tokens,
                                usage.overview.total_cost_usd
                            )),
                    );
                    let maximum = usage
                        .trends
                        .iter()
                        .map(|row| row.request_count)
                        .max()
                        .unwrap_or(0)
                        .max(1) as f32;
                    let mut days = div()
                        .id("workbench-daily")
                        .flex()
                        .flex_col()
                        .gap(px(theme::CONTROL_GAP));
                    for row in usage.trends.iter().rev().take(7).rev() {
                        days = days.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(theme::CONTROL_GAP))
                                .child(
                                    div()
                                        .w(px(theme::DATE_COLUMN_WIDTH))
                                        .text_size(px(theme::META_SIZE))
                                        .child(row.date.clone()),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .h(px(theme::CONTROL_GAP))
                                        .bg(cx.theme().muted)
                                        .child(
                                            div()
                                                .h_full()
                                                .w(gpui_kit::relative(
                                                    row.request_count as f32 / maximum,
                                                ))
                                                .bg(cx.theme().primary),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(theme::DAILY_VALUE_WIDTH))
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_size(px(theme::META_SIZE))
                                        .child(format!(
                                            "{} / ${:.2}",
                                            row.request_count, row.cost_usd
                                        )),
                                ),
                        );
                    }
                    section = section.child(days);
                    if usage.trends.is_empty() {
                        section = section.child(text(
                            cx,
                            "No recorded usage in this period.",
                            "此时间段没有用量记录。",
                        ));
                    }
                }
            }
            Some(Err(error)) => {
                section = section.child(
                    div()
                        .id("workbench-usage-unavailable")
                        .test_support()
                        .role(gpui_kit::Role::Status)
                        .aria_label(service_error(cx, error))
                        .child(service_error(cx, error)),
                )
            }
            None => {
                section = section.child(text(cx, "Reading platform usage…", "正在读取平台用量…"))
            }
        }
        section.into_any_element()
    }
}

impl Render for WorkbenchView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let profiles = matches!(
            self.platform,
            Platform::Claude | Platform::Codex | Platform::Grok
        );
        let mut actions = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        if profiles {
            actions = actions
                .child(self.button(
                    "workbench-auth",
                    Link::Auth,
                    if self.platform == Platform::Claude {
                        "Open authentication"
                    } else {
                        "Authentication"
                    },
                    if self.platform == Platform::Claude {
                        "打开认证"
                    } else {
                        "认证"
                    },
                    true,
                    cx,
                ))
                .child(self.button(
                    "workbench-profiles",
                    Link::Profiles,
                    "Manage Profiles",
                    "管理 Profiles",
                    false,
                    cx,
                ));
        } else {
            actions = actions.child(self.button(
                "workbench-settings",
                Link::Settings,
                "Open settings",
                "打开设置",
                true,
                cx,
            ));
        }
        let page = div()
            .id("platform-workbench")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .items_start()
                    .gap(px(theme::SECTION_GAP))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(theme::CONTROL_GAP))
                            .child(
                                div()
                                    .id("workbench-title")
                                    .test_support()
                                    .aria_label(name(self.platform))
                                    .role(gpui_kit::Role::Heading)
                                    .aria_level(1)
                                    .text_size(px(theme::TITLE_SIZE))
                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                    .child(name(self.platform)),
                            )
                            .when(self.platform != Platform::Claude, |heading| {
                                heading.child(text(
                                    cx,
                                    "Local platform workbench",
                                    "本地平台工作台",
                                ))
                            }),
                    )
                    .child(actions),
            );
        if self.platform == Platform::Claude {
            return page
                .child(self.claude_status(cx))
                .child(self.claude_insights(cx))
                .child(self.claude_trend(cx));
        }
        let mut management = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for (id, link, en, zh) in [
            ("workbench-mcp", Link::Mcp, "MCP", "MCP"),
            (
                "workbench-files",
                Link::Files,
                "Instruction files",
                "指令文件",
            ),
            (
                "workbench-usage",
                Link::Usage,
                "Full usage report",
                "完整用量报告",
            ),
        ] {
            if (self.platform == Platform::Grok && matches!(link, Link::Files | Link::Mcp))
                || (self.platform == Platform::OpenCode && matches!(link, Link::Mcp))
            {
                continue;
            }
            management = management.child(self.button(id, link, en, zh, false, cx));
        }
        if self.platform == Platform::Codex {
            management = management.child(self.button(
                "workbench-sessions",
                Link::Sessions,
                "Sessions",
                "会话",
                false,
                cx,
            ));
        }
        let rolling = (self.platform == Platform::Codex).then(|| self.rolling_usage(cx));
        let grok = (self.platform == Platform::Grok).then(|| self.grok_overview(cx));
        page.child(self.current(cx))
            .children(grok)
            .children(rolling)
            .child(self.usage(cx))
            .child(
                div()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .pt(px(theme::SECTION_GAP))
                    .child(management),
            )
            .child(semantics::button(
                Button::new("workbench-refresh")
                    .label(text(cx, "Refresh status", "刷新状态"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| this.open(this.platform, cx))),
                false,
                false,
                cx,
            ))
    }
}
