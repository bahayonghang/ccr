//! Native configuration catalog. Domain-backed actions live in the Profiles feature.
mod cards;
mod toolbar;

use crate::{
    features::profiles::ProfileTools,
    locales::{is_chinese, service_error, text},
    runtime::{
        Services,
        profiles::{ProfileCatalog, ProfileError},
    },
    shell::{badge, semantics},
    theme,
};
use ccr_app::{
    config::ConfigRow,
    runtime::{AdmissionError, Completion, Platform, RequestKey, ResponseGate, TaskEvent},
};
use ccr_cli::application::profile_observation::{ProfileObservation, RuntimeState};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Disableable, Selectable, Sizable,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ListAlignment, ListOffset, ListState, ParentElement, Render, StatefulInteractiveElement,
    Styled, Subscription, Task, Window, div, list, px,
};
use std::sync::Arc;

pub struct ConfigurationApp {
    services: Services,
    catalog_environment: ccr_app::runtime::EnvironmentId,
    rows: Arc<Vec<ConfigRow>>,
    all_rows: Arc<Vec<ConfigRow>>,
    observation: Option<ProfileObservation>,
    runtime_file: String,
    provider_filter: Option<String>,
    tag_filter: Option<String>,
    enabled_filter: Option<bool>,
    sort_usage: bool,
    management: bool,
    selected: Option<String>,
    detail_open: bool,
    gate: ResponseGate,
    generation: u64,
    read_task: Option<Task<()>>,
    loading: bool,
    error: Option<&'static str>,
    search: Entity<InputState>,
    search_text: String,
    search_language: Option<bool>,
    _search_subscription: Subscription,
    list_focus: FocusHandle,
    detail_focus: FocusHandle,
    scroll: ListState,
    columns: usize,
    card_focus: std::collections::HashMap<String, (FocusHandle, FocusHandle)>,
    filter_panel: Option<toolbar::FilterPanel>,
    tools: Entity<ProfileTools>,
    _tools_subscription: Subscription,
    tools_version: u64,
    platform: Platform,
    pinned: bool,
}

impl ConfigurationApp {
    pub fn has_dirty_draft(&self, cx: &gpui_kit::App) -> bool {
        self.tools.read(cx).has_dirty_draft(cx)
    }
    pub fn read_generation(&self) -> u64 {
        self.generation
    }
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::build(services, true, window, cx)
    }
    pub(crate) fn new_deferred(
        services: Services,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::build(services, false, window, cx)
    }
    fn build(services: Services, load: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx).placeholder(text(
                cx,
                "Search name, endpoint or model",
                "搜索名称、端点或模型",
            ))
        });
        let subscription = cx.subscribe(&search, |this, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                this.search_text = this.search.read(cx).value().to_lowercase();
                this.refilter();
                this.sync_target(cx);
                cx.notify();
            }
        });
        let tools = cx.new(|cx| {
            let mut tools = ProfileTools::new(services.clone(), window, cx);
            tools.embedded = true;
            tools
        });
        let tools_subscription = cx.observe(&tools, |this, tools, cx| {
            let version = tools.read(cx).refresh_version;
            if this.tools_version != version {
                this.tools_version = version;
                this.refresh(cx);
            }
            cx.notify();
        });
        let mut this = Self {
            catalog_environment: services.environment(),
            services,
            rows: Arc::new(Vec::new()),
            all_rows: Arc::new(Vec::new()),
            observation: None,
            runtime_file: String::new(),
            provider_filter: None,
            tag_filter: None,
            enabled_filter: None,
            sort_usage: false,
            management: false,
            selected: None,
            detail_open: false,
            gate: ResponseGate::default(),
            generation: 0,
            read_task: None,
            loading: false,
            error: None,
            search,
            search_text: String::new(),
            search_language: None,
            _search_subscription: subscription,
            list_focus: cx.focus_handle(),
            detail_focus: cx.focus_handle().tab_stop(false),
            scroll: ListState::new(0, ListAlignment::Top, px(100.)),
            columns: 1,
            card_focus: Default::default(),
            filter_panel: None,
            tools,
            _tools_subscription: tools_subscription,
            tools_version: 0,
            platform: Platform::Claude,
            pinned: false,
        };
        if load {
            this.refresh(cx);
        }
        this
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.showing_history(cx) {
            let focus = self.tools.read(cx).output_focus(cx);
            window.focus(&focus, cx);
            return;
        }
        window.focus(&self.search.read(cx).focus_handle(cx), cx);
    }
    pub fn showing_history(&self, cx: &gpui_kit::App) -> bool {
        self.tools.read(cx).showing_history()
    }
    #[cfg(test)]
    pub fn history_busy(&self, cx: &gpui_kit::App) -> bool {
        self.tools.read(cx).busy()
    }
    pub fn open_history(
        &mut self,
        platform: Platform,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.suspend(cx);
        self.platform = platform;
        self.rows = Arc::new(Vec::new());
        self.all_rows = Arc::new(Vec::new());
        self.selected = None;
        self.detail_open = false;
        self.error = None;
        self.tools
            .update(cx, |tools, cx| tools.open_history(platform, window, cx));
        cx.notify();
    }
    /// Navigation dismisses the read subscription, not the application-owned runner.
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        self.read_task = None;
        self.gate.clear();
        self.loading = false;
        cx.notify();
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.suspend(cx);
        self.management = false;
        self.tools.update(cx, |tools, cx| tools.leave(window, cx));
    }
    /// Shell destinations own the platform choice, so the in-view switcher stays hidden.
    pub fn open_platform(
        &mut self,
        platform: Platform,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pinned = true;
        if self.platform == platform {
            self.refresh(cx);
        } else {
            self.provider_filter = None;
            self.tag_filter = None;
            self.enabled_filter = None;
            self.sort_usage = false;
            self.platform(platform, window, cx);
        }
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.read_task = None;
        self.generation += 1;
        let environment = self.services.environment();
        if self.catalog_environment != environment {
            self.rows = Arc::new(Vec::new());
            self.all_rows = Arc::new(Vec::new());
            self.selected = None;
            self.detail_open = false;
            self.card_focus.clear();
            self.sync_target(cx);
        }
        self.catalog_environment = environment.clone();
        let key = RequestKey {
            environment,
            platform: self.platform,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.loading = true;
        self.observation = None;
        self.runtime_file.clear();
        self.tools.update(cx, |tools, cx| tools.refresh_status(cx));
        self.error = None;
        match self.services.profile_catalog(key.clone(), String::new()) {
            Ok(mut subscription) => {
                self.read_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak
                                .update(cx, |this, cx| this.finish_read(&key, result.as_ref(), cx));
                            break;
                        }
                    }
                }));
            }
            Err(error) => {
                self.loading = false;
                self.error = Some(match error {
                    AdmissionError::Full => "Read queue is busy. Retry. / 读取队列繁忙，请重试",
                    AdmissionError::Stopped => "Application is shutting down / 应用正在退出",
                });
            }
        }
        cx.notify();
    }
    fn finish_read(
        &mut self,
        key: &RequestKey,
        result: &Completion<ProfileCatalog, ProfileError>,
        cx: &mut Context<Self>,
    ) {
        if !self.gate.accepts(key) {
            return;
        }
        self.loading = false;
        match result {
            Completion::Success(catalog) => {
                self.observation = Some(catalog.observation.clone());
                self.runtime_file.clone_from(&catalog.runtime_file);
                self.accept_rows(catalog.rows.clone());
                self.sync_target(cx);
            }
            Completion::Failed(error) | Completion::CleanupFailed(error) => {
                self.all_rows = Arc::new(Vec::new());
                self.rows = Arc::new(Vec::new());
                self.card_focus.clear();
                self.selected = None;
                self.sync_target(cx);
                self.detail_open = false;
                self.error = Some(error.message());
            }
            Completion::Cancelled => self.error = Some("Read cancelled / 已取消读取"),
            Completion::WorkerFailed => {
                self.error = Some("Read worker stopped. Retry. / 读取线程已停止，请重试")
            }
        }
        cx.notify();
    }
    fn accept_rows(&mut self, rows: Arc<Vec<ConfigRow>>) {
        self.all_rows = rows;
        let names = self
            .all_rows
            .iter()
            .map(|r| r.name.as_str())
            .collect::<std::collections::HashSet<_>>();
        self.card_focus
            .retain(|name, _| names.contains(name.as_str()));
        self.refilter();
    }

    fn refilter(&mut self) {
        let mut rows =
            self.all_rows
                .iter()
                .filter(|row| {
                    (row.name.to_lowercase().contains(&self.search_text)
                        || row.base_url.to_lowercase().contains(&self.search_text)
                        || row
                            .model
                            .as_deref()
                            .is_some_and(|model| model.to_lowercase().contains(&self.search_text)))
                        && self
                            .provider_filter
                            .as_ref()
                            .is_none_or(|provider| row.provider.as_ref() == Some(provider))
                        && self.tag_filter.as_ref().is_none_or(|tag| {
                            row.tags.as_ref().is_some_and(|tags| tags.contains(tag))
                        })
                        && self
                            .enabled_filter
                            .is_none_or(|enabled| row.enabled == enabled)
                })
                .cloned()
                .collect::<Vec<_>>();
        if self.sort_usage {
            rows.sort_by(|a, b| {
                b.usage_count
                    .cmp(&a.usage_count)
                    .then_with(|| a.name.cmp(&b.name))
            });
        } else {
            rows.sort_by(|a, b| {
                a.name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then_with(|| a.name.cmp(&b.name))
            });
        }
        let top = self.scroll.logical_scroll_top();
        let anchor = self.rows.get(top.item_ix * self.columns).map(|r| &r.name);
        let new_anchor = anchor.and_then(|name| rows.iter().position(|r| &r.name == name));
        self.rows = Arc::new(rows);
        self.scroll.reset(self.rows.len().div_ceil(self.columns));
        if let Some(index) = new_anchor {
            self.scroll.scroll_to(ListOffset {
                item_ix: index / self.columns,
                ..top
            });
        }
        if !self
            .rows
            .iter()
            .any(|row| Some(&row.name) == self.selected.as_ref())
        {
            self.selected = None;
            self.detail_open = false;
        }
    }
    fn sync_target(&mut self, cx: &mut Context<Self>) {
        let row = self
            .rows
            .iter()
            .find(|row| Some(&row.name) == self.selected.as_ref())
            .cloned();
        self.tools.update(cx, |tools, cx| tools.set_target(row, cx));
    }
    fn platform(&mut self, platform: Platform, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .tools
            .update(cx, |tools, cx| tools.set_platform(platform, window, cx))
        {
            self.platform = platform;
            self.rows = Arc::new(Vec::new());
            self.card_focus.clear();
            self.filter_panel = None;
            self.all_rows = Arc::new(Vec::new());
            self.selected = None;
            self.detail_open = false;
            self.refresh(cx);
        }
    }
    fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.detail_open = false;
        if let Some((focus, _)) = self
            .selected
            .as_ref()
            .and_then(|name| self.card_focus.get(name))
        {
            window.focus(focus, cx);
        } else {
            window.focus(&self.list_focus, cx);
        }
        cx.notify();
    }
    fn runtime_summary(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let source = match &self.catalog_environment {
            ccr_app::runtime::EnvironmentId::Local(_) => text(cx, "Local", "本机"),
            ccr_app::runtime::EnvironmentId::Ssh(_) => text(cx, "SSH host", "SSH 主机"),
            ccr_app::runtime::EnvironmentId::Wsl(_) => "WSL",
        };
        let status = match &self.observation {
            Some(observation) => {
                let label = state_label(observation.state, cx);
                match &observation.name {
                    Some(name) if self.platform == Platform::Claude => {
                        format!("{} {name} · {label}", text(cx, "CCR marker", "CCR 标记"))
                    }
                    Some(name) => format!("{name} · {label}"),
                    None => label.into(),
                }
            }
            None if self.loading => text(cx, "Reading catalog…", "正在读取目录…").into(),
            None => text(cx, "Catalog unavailable", "目录不可用").into(),
        };
        let file = self
            .runtime_file
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or_default();
        let summary = if file.is_empty() {
            format!("{source} · {status}")
        } else {
            format!("{source} · {file} · {status}")
        };
        let source_path = self.runtime_file.clone();
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .id("configuration-runtime-summary")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(summary.clone())
                    .child(summary)
                    .when(!source_path.is_empty(), |view| {
                        view.tooltip(move |window, cx| {
                            gpui_kit::component::tooltip::Tooltip::new(source_path.clone())
                                .build(window, cx)
                        })
                    }),
            )
            .child(semantics::ghost_button(
                Button::new("profile-off")
                    .label(text(cx, "Detach current profile", "解除当前方案"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.tools
                            .update(cx, |tools, cx| tools.detach_current(window, cx))
                    })),
                self.tools.read(cx).expanded() || !self.tools.read(cx).can_detach(),
                false,
                cx,
            ))
    }
}

fn state_label(state: RuntimeState, cx: &gpui_kit::App) -> &'static str {
    match state {
        RuntimeState::NoMarker => text(cx, "No current marker", "无当前标记"),
        RuntimeState::NoMatch => text(cx, "No runtime config match", "运行配置无匹配"),
        RuntimeState::Inactive => text(cx, "Inactive", "未激活"),
        RuntimeState::Matched => text(cx, "Runtime config matches", "运行配置匹配"),
        RuntimeState::Drifted => text(cx, "Runtime config differs", "运行配置偏离"),
        RuntimeState::OverridesCleared => text(cx, "CCR overrides cleared", "CCR 托管覆盖已清除"),
        RuntimeState::OverridesPresent => text(cx, "CCR overrides remain", "仍有 CCR 托管覆盖"),
        RuntimeState::BaselineMissing => text(cx, "Recovery baseline missing", "恢复基线缺失"),
        RuntimeState::Missing => text(cx, "Runtime file missing", "运行文件缺失"),
        RuntimeState::Invalid => text(cx, "Cannot verify · invalid format", "无法核对 · 格式无效"),
        RuntimeState::Unreadable => text(
            cx,
            "Cannot verify · source unreadable",
            "无法核对 · 来源不可读",
        ),
        RuntimeState::Unsupported => text(cx, "Verification unsupported", "该来源不支持核对"),
    }
}

fn auth_label(row: &ConfigRow, cx: &gpui_kit::App) -> &'static str {
    match row.auth_mode.as_deref() {
        Some("subscription") => text(cx, "Claude subscription", "Claude 订阅"),
        Some("api_key" | "inline_api_key" | "openai_api_key") => text(cx, "API key", "API 密钥"),
        Some("openai_chatgpt") => text(cx, "ChatGPT login", "ChatGPT 登录"),
        Some("provider_env_key" | "env_key") => text(cx, "Environment variable", "环境变量"),
        Some("provider_bearer_token") => text(cx, "Provider bearer token", "供应商访问令牌"),
        Some("no_auth") => text(cx, "No authentication", "无需认证"),
        Some("session") => text(cx, "Grok session or environment", "Grok 会话或环境变量"),
        _ => text(cx, "Invalid authentication settings", "认证配置无效"),
    }
}

fn profile_details(row: &ConfigRow, cx: &gpui_kit::App) -> gpui_kit::Div {
    let unknown = text(cx, "Unknown", "未知");
    let mut grid = div().grid().grid_cols(2).gap(px(12.));
    for (label, value) in [
        (
            text(cx, "Endpoint", "端点"),
            if row.base_url.is_empty() {
                unknown.into()
            } else {
                row.base_url.clone()
            },
        ),
        (
            text(cx, "Model", "模型"),
            row.model
                .clone()
                .unwrap_or_else(|| text(cx, "Unspecified", "未指定").into()),
        ),
        (
            text(cx, "Authentication", "认证方式"),
            auth_label(row, cx).into(),
        ),
        (
            text(cx, "Provider", "供应商"),
            row.provider.clone().unwrap_or_else(|| unknown.into()),
        ),
        (
            text(cx, "Tags", "标签"),
            row.tags
                .as_ref()
                .map(|tags| tags.join(", "))
                .unwrap_or_else(|| unknown.into()),
        ),
        (
            text(cx, "Observed uses", "已观测使用次数"),
            row.usage_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| unknown.into()),
        ),
    ] {
        grid = grid.child(
            div()
                .min_w_0()
                .child(
                    div()
                        .text_size(crate::theme::text_size(12.))
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .child(value),
        );
    }
    let credential = if row.auth_token.is_empty() {
        auth_label(row, cx)
    } else {
        "••••"
    };
    grid.child(
        div()
            .id("configuration-credential")
            .test_support()
            .aria_label(format!("{}: {credential}", text(cx, "Credential", "凭据")))
            .child(format!("{}: {credential}", text(cx, "Credential", "凭据"))),
    )
}

impl Render for ConfigurationApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.showing_history(cx) {
            let busy = self.tools.read(cx).busy();
            return div()
                .flex()
                .flex_col()
                .size_full()
                .min_h_0()
                .p(px(theme::CONTENT_PADDING))
                .gap(px(theme::CONTROL_GAP))
                .child(div().id("configuration-history").test_support().child(text(
                    cx,
                    "Application operation records · independent of the execution environment",
                    "应用操作记录 · 不随执行环境切换",
                )))
                .child(
                    div().flex().items_center().child(semantics::button(
                        Button::new("configuration-history-back")
                            .label(if busy {
                                text(cx, "Reading operation records…", "正在读取操作记录…")
                            } else {
                                text(cx, "Back to profiles", "返回配置方案")
                            })
                            .disabled(busy)
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.tools
                                    .update(cx, |tools, cx| tools.close_history(window, cx));
                                let available = match this.services.environment() {
                                    ccr_app::runtime::EnvironmentId::Local(_) => true,
                                    ccr_app::runtime::EnvironmentId::Ssh(id) => {
                                        this.services.ssh_connected(&id)
                                    }
                                    _ => false,
                                };
                                if available {
                                    this.refresh(cx);
                                }
                                cx.notify();
                            })),
                        busy,
                        busy,
                        cx,
                    )),
                )
                .child(self.tools.clone());
        }
        let split = window.viewport_size().width >= px(theme::SPLIT_WIDTH);
        let status = if self.loading {
            text(cx, "Reading catalog", "正在读取配置").into()
        } else if let Some(error) = self.error {
            service_error(cx, error).to_string()
        } else if self.rows.is_empty() {
            if self.all_rows.is_empty() {
                text(cx, "This catalog has no profiles", "这个目录里没有配置").into()
            } else {
                text(cx, "No profiles match this filter", "没有符合筛选的配置").into()
            }
        } else {
            self.count_label(cx)
        };
        let detail = self
            .rows
            .iter()
            .find(|row| Some(&row.name) == self.selected.as_ref())
            .cloned();
        let show_detail = self.detail_open && detail.is_some();
        let mut body = div()
            .flex()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .gap(px(theme::SECTION_GAP));
        if (split || !show_detail)
            && !self.management
            && !self.tools.read(cx).collection_workspace()
        {
            body = body.child(self.card_list(window, cx));
        }
        if !self.management
            && !self.tools.read(cx).collection_workspace()
            && show_detail
            && let Some(row) = detail
        {
            body = body.child(
                div()
                    .id("configuration-detail")
                    .test_support()
                    .overflow_y_scroll()
                    .role(gpui_kit::Role::Region)
                    .aria_label(text(cx, "Profile details", "配置详情"))
                    .track_focus(&self.detail_focus)
                    .tab_stop(false)
                    .flex_grow(2.)
                    .flex_basis(px(0.))
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(px(theme::SECTION_GAP))
                    .when(!self.tools.read(cx).expanded(), |pane| {
                        pane.child(
                            div().flex().items_center().child(semantics::ghost_button(
                                Button::new("configuration-back")
                                    .label(text(cx, "Back to profiles", "返回配置列表"))
                                    .ghost()
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.back(window, cx)),
                                    ),
                                false,
                                false,
                                cx,
                            )),
                        )
                        .child(
                            div()
                                .text_size(crate::theme::text_size(theme::TITLE_SIZE))
                                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                .child(row.name.clone()),
                        )
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child(row.description.clone()),
                        )
                        .child(profile_details(&row, cx))
                    })
                    .child(self.tools.clone())
                    .on_key_down(cx.listener(
                        |this, event: &gpui_kit::KeyDownEvent, window, cx| {
                            if event.keystroke.key == "escape" {
                                this.back(window, cx);
                                cx.stop_propagation();
                            }
                        },
                    )),
            );
        }
        let show_toolbar = split || !show_detail;
        let (title_en, title_zh) = if self.pinned {
            match self.platform {
                Platform::Codex => ("Codex", "Codex"),
                Platform::Grok => ("Grok", "Grok"),
                _ => ("Claude Code", "Claude Code"),
            }
        } else {
            ("Configurations", "配置")
        };
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_w_0()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .rounded_tr(px(theme::SHEET_CONTENT_RADIUS))
            .text_color(cx.theme().foreground)
            .text_size(crate::theme::text_size(theme::BODY_SIZE))
            .when(!self.pinned, |root| {
                root.child(
                    div()
                        .id("configuration-heading")
                        .role(gpui_kit::Role::Heading)
                        .aria_label(text(cx, title_en, title_zh))
                        .aria_level(1)
                        .text_size(crate::theme::text_size(theme::TITLE_SIZE))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .child(text(cx, title_en, title_zh)),
                )
            })
            .when(show_toolbar, |root| root.child(self.runtime_summary(cx)))
            .when(show_toolbar, |root| {
                root.child(self.toolbar(status, window, cx))
            })
            .when(!self.pinned, |parent| {
                parent.child(
                    div()
                        .flex()
                        .gap(px(4.))
                        .child(semantics::button(
                            Button::new("profiles-platform-claude")
                                .selected(self.platform == Platform::Claude)
                                .toggled(self.platform == Platform::Claude)
                                .label("Claude Code")
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.platform(Platform::Claude, window, cx)
                                })),
                            self.tools.read(cx).expanded() || self.tools.read(cx).busy(),
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("profiles-platform-codex")
                                .selected(self.platform == Platform::Codex)
                                .toggled(self.platform == Platform::Codex)
                                .label("Codex")
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.platform(Platform::Codex, window, cx)
                                })),
                            self.tools.read(cx).expanded() || self.tools.read(cx).busy(),
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("profiles-platform-grok")
                                .selected(self.platform == Platform::Grok)
                                .toggled(self.platform == Platform::Grok)
                                .label("Grok")
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.platform(Platform::Grok, window, cx)
                                })),
                            self.tools.read(cx).expanded() || self.tools.read(cx).busy(),
                            false,
                            cx,
                        )),
                )
            })
            .when(
                show_toolbar
                    && !self.detail_open
                    && !self.management
                    && !self.tools.read(cx).collection_workspace(),
                |root| root.children(self.tools.read(cx).feedback(cx)),
            )
            .child(
                if self.management || self.tools.read(cx).collection_workspace() {
                    self.tools.clone().into_any_element()
                } else {
                    body.into_any_element()
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ConfigSource;
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::test::TestWindowExt;
    use gpui_kit::{TestAppContext, WindowOptions};
    use std::time::Duration;
    #[gpui_kit::test]
    fn refresh_hides_previous_local_and_host_catalog_metadata_before_new_data_returns(
        cx: &mut TestAppContext,
    ) {
        for previous in [
            ccr_app::runtime::EnvironmentId::Local("old-local".into()),
            ccr_app::runtime::EnvironmentId::Ssh("old-host".into()),
        ] {
            let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture"));
            let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
            let services = Services::new(
                owner.handle(),
                ConfigSource::Fixture {
                    profiles: root.path().join("must-not-read.toml"),
                    registry: root.path().join("registry.toml"),
                    delay: Duration::ZERO,
                },
            );
            services.select_environment(previous.clone());
            let (handle, view) = cx.update(|cx| {
                gpui_kit::init(cx);
                gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                    cx.new(|cx| {
                        let mut view = ConfigurationApp::new_deferred(services.clone(), window, cx);
                        let mut cached = row(0);
                        cached.provider = Some("previous-provider".into());
                        cached.tags = Some(vec!["previous-tag".into()]);
                        view.accept_rows(Arc::new(vec![cached]));
                        view.selected = Some("profile-00000".into());
                        view
                    })
                })
                .unwrap_or_else(|_| panic!("window"))
            });
            cx.update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                window.click("configuration-filter-group", cx);
                assert!(window.try_find("configuration-provider-0").is_some());
                assert!(window.try_find("configuration-tag-0").is_some());
                services
                    .select_environment(ccr_app::runtime::EnvironmentId::Ssh("new-host".into()));
                view.update(cx, |view, cx| view.refresh(cx));
                window.render_frame(cx);
                let old = RequestKey {
                    environment: previous,
                    platform: Platform::Claude,
                    generation: 0,
                    request_id: 0,
                };
                let late = Completion::Success(ProfileCatalog {
                    rows: Arc::new(vec![row(0)]),
                    observation: ProfileObservation {
                        name: Some("previous-host".into()),
                        state: RuntimeState::Matched,
                    },
                    runtime_file: "previous-settings.json".into(),
                });
                view.update(cx, |this, cx| this.finish_read(&old, &late, cx));
                assert!(view.read(cx).observation.is_none());
                assert!(view.read(cx).runtime_file.is_empty());
                assert!(view.read(cx).loading);
                assert!(view.read(cx).all_rows.is_empty());
                assert!(view.read(cx).selected.is_none());
                assert!(window.try_find("configuration-provider-0").is_none());
                assert!(window.try_find("configuration-tag-0").is_none());
                window.remove_window();
            })
            .unwrap_or_else(|_| panic!("catalog transition"));
            owner.shutdown();
        }
    }
    fn row(index: usize) -> ConfigRow {
        ConfigRow {
            version: "fixture-v1".into(),
            name: format!("profile-{index:05}"),
            description: "Synthetic view fixture".into(),
            base_url: "https://fixture.invalid".into(),
            auth_token: "***".into(),
            model: None,
            auth_mode: None,
            small_fast_model: None,
            is_current: index == 0,
            is_default: false,
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
            usage_count: None,
            enabled: true,
        }
    }
    #[gpui_kit::test]
    fn catalog_filters_recover_and_narrow_back_preserves_selection(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(gpui_kit::WindowBounds::Windowed(gpui_kit::Bounds::new(
                        gpui_kit::point(px(0.), px(0.)),
                        gpui_kit::size(px(960.), px(640.)),
                    ))),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    cx.new(|cx| {
                        let mut view = ConfigurationApp::new(
                            Services::new(
                                owner.handle(),
                                ConfigSource::Fixture {
                                    profiles: root.path().join("missing.toml"),
                                    registry: root.path().join("registry.toml"),
                                    delay: Duration::ZERO,
                                },
                            ),
                            window,
                            cx,
                        );
                        view.suspend(cx);
                        view.pinned = true;
                        let mut a = row(0);
                        a.provider = Some("provider-a".into());
                        a.tags = Some(vec!["tag-a".into()]);
                        let mut b = row(1);
                        b.provider = Some("provider-b".into());
                        b.tags = Some(vec!["tag-b".into()]);
                        b.enabled = false;
                        view.accept_rows(Arc::new(vec![a, b]));
                        view
                    })
                },
            )
            .unwrap_or_else(|e| panic!("window: {e}"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("configuration-filter-group", cx);
            window.click("configuration-provider-1", cx);
            window.click("configuration-tag-1", cx);
            window.click("configuration-filter-state", cx);
            window.click("configuration-disabled-only", cx);
            window.click("configuration:profile-00001", cx);
            assert!(window.try_find("configuration-filter").is_none());
            assert!(window.try_find("list-focus").is_none());
            window.click("configuration-back", cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00001"));
            assert_eq!(view.read(cx).provider_filter.as_deref(), Some("provider-b"));
            assert_eq!(view.read(cx).tag_filter.as_deref(), Some("tag-b"));
            assert_eq!(
                window.find("configuration:profile-00001").focused(),
                Some(true)
            );
            window.click("configuration-filter-state", cx);
            window.click("configuration-enabled-only", cx);
            assert_eq!(
                window.find("configuration-status").label(),
                Some("No profiles match this filter")
            );
            assert!(view.read(cx).selected.is_none());
            assert!(
                window
                    .try_find("configuration-apply:profile-00001")
                    .is_none()
            );
            window.click("configuration-remove-tag", cx);
            assert!(view.read(cx).tag_filter.is_none());
            assert_eq!(view.read(cx).provider_filter.as_deref(), Some("provider-b"));
            window.click("configuration-filter-clear", cx);
            assert_eq!(
                window.find("configuration-status").label(),
                Some("2 / 2 profiles")
            );
            assert!(view.read(cx).selected.is_none());
            view.update(cx, |this, cx| {
                this.accept_rows(Arc::new(Vec::new()));
                this.sync_target(cx);
                cx.notify();
            });
            window.render_frame(cx);
            assert_eq!(
                window.find("configuration-status").label(),
                Some("This catalog has no profiles")
            );
            assert!(window.find("profile-create").visible());
            assert!(window.try_find("configuration-filter-clear").is_none());
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("filter recovery: {e}"));
        owner.shutdown();
    }
    #[gpui_kit::test]
    fn virtual_rows_keyboard_detail_back_and_stable_selection(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| {
                    let mut view = ConfigurationApp::new(
                        Services::new(
                            owner.handle(),
                            ConfigSource::Fixture {
                                profiles: root.path().join("missing.toml"),
                                registry: root.path().join("config.toml"),
                                delay: Duration::ZERO,
                            },
                        ),
                        window,
                        cx,
                    );
                    view.suspend(cx);
                    view.accept_rows(Arc::new((0..10_000).map(row).collect()));
                    view
                })
            })
            .unwrap_or_else(|e| panic!("window: {e}"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.render_frame(cx);
            let visible_cards = view.read(cx).card_focus.len();
            assert!((1..100).contains(&visible_cards));
            window.click("configuration:profile-00000", cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00000"));
            window.click("configuration-back", cx);
            assert_eq!(
                window.find("configuration:profile-00000").focused(),
                Some(true)
            );
            window.press("right", cx);
            assert_eq!(
                window.find("configuration:profile-00001").focused(),
                Some(true)
            );
            window.press("enter", cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00001"));
            assert!(window.find("configuration-back").visible());
            window.press("escape", cx);
            assert!(!view.read(cx).detail_open);
            assert_eq!(
                window.find("configuration:profile-00001").focused(),
                Some(true)
            );
            window.press("tab", cx);
            assert_eq!(
                window.find("configuration-apply:profile-00001").focused(),
                Some(true)
            );
            view.update(cx, |this, cx| {
                this.accept_rows(Arc::new(vec![row(20), row(1)]));
                cx.notify();
            });
            window.render_frame(cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00001"));
            for _ in 0..2 {
                window.render_frame(cx);
            }
            assert_eq!(
                window.find("configuration-apply:profile-00001").focused(),
                Some(true)
            );
        })
        .unwrap_or_else(|e| panic!("interaction: {e}"));
        cx.simulate_window_resize(handle, gpui_kit::size(px(540.), px(720.)));
        cx.update_window(handle, |_, window, cx| {
            theme::apply_typography(
                &crate::runtime::preferences::TypographyPreferences {
                    ui_size: 20,
                    ..Default::default()
                },
                cx,
            );
            window.set_rem_size(cx.theme().font_size);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            window.render_frame(cx);
            assert_eq!(view.read(cx).columns, 1);
            assert_eq!(
                window.find("configuration-apply:profile-00001").focused(),
                Some(true)
            );
            let long_name = format!("profile-00001-{}", "very-long-profile-segment-".repeat(5));
            let mut long = row(1);
            long.name = long_name.clone();
            long.base_url = format!(
                "https://user:password@{}.invalid:8443/private?token=secret",
                "long-host-name-".repeat(4)
            );
            let long_id = format!("configuration:{long_name}");
            let apply_id = format!("configuration-apply:{long_name}");
            view.update(cx, |this, cx| {
                this.accept_rows(Arc::new(vec![long, row(2)]));
                cx.notify();
            });
            window.render_frame(cx);
            let card = window.find(long_id).bounds();
            let apply = window.find(apply_id.clone()).bounds();
            assert!(card.size.height > px(140.));
            assert!(apply.top() >= card.bottom() && apply.right() <= card.right());
            let viewport = window.find("list-focus").bounds();
            if apply.bottom() > viewport.bottom() {
                window.scroll(
                    "list-focus",
                    gpui_kit::ScrollDelta::Pixels(gpui_kit::point(
                        px(0.),
                        viewport.bottom() - apply.bottom() - px(8.),
                    )),
                    cx,
                );
                window.render_frame(cx);
            }
            let visible_action = window.find(apply_id).bounds();
            assert!(
                visible_action.top() >= viewport.top()
                    && visible_action.bottom() <= viewport.bottom(),
                "action {visible_action:?} must be reachable in {viewport:?}, card {card:?}"
            );
            // Narrow drill-in round-trips from a scrolled band.
            view.update(cx, |this, cx| {
                this.accept_rows(Arc::new((0..100).map(row).collect()));
                this.scroll.scroll_to(ListOffset {
                    item_ix: 18,
                    offset_in_item: px(12.),
                });
                cx.notify();
            });
            window.render_frame(cx);
            let offset = view.read(cx).scroll.logical_scroll_top();
            window.click("configuration:profile-00019", cx);
            window.press("escape", cx);
            assert_eq!(
                window.find("configuration:profile-00019").focused(),
                Some(true)
            );
            let restored = view.read(cx).scroll.logical_scroll_top();
            assert_eq!(restored.item_ix, offset.item_ix);
            assert_eq!(restored.offset_in_item, offset.offset_in_item);
            window.press("tab", cx);
            assert_eq!(
                window.find("configuration-apply:profile-00019").focused(),
                Some(true)
            );
            window.press("enter", cx);
            assert_eq!(
                window.find("profile-result").label(),
                Some("Applying profile-00019…")
            );
            assert!(window.try_find("configuration-detail").is_none());
            assert!(view.read(cx).tools.read(cx).busy());
            window.click("configuration-apply:profile-00018", cx);
            assert_eq!(
                window.find("profile-result").label(),
                Some("Applying profile-00019…")
            );
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("resize and drill-in: {e}"));
        owner.shutdown();
    }

    #[gpui_kit::test]
    fn wide_catalog_uses_available_width_and_independent_apply_controls(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(gpui_kit::WindowBounds::Windowed(gpui_kit::Bounds::new(
                        gpui_kit::point(px(0.), px(0.)),
                        gpui_kit::size(px(1600.), px(1000.)),
                    ))),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    cx.new(|cx| {
                        let mut view = ConfigurationApp::new(
                            Services::new(
                                owner.handle(),
                                ConfigSource::Fixture {
                                    profiles: root.path().join("missing.toml"),
                                    registry: root.path().join("registry.toml"),
                                    delay: Duration::ZERO,
                                },
                            ),
                            window,
                            cx,
                        );
                        view.suspend(cx);
                        view.pinned = true;
                        let mut disabled = row(1);
                        disabled.enabled = false;
                        view.accept_rows(Arc::new(vec![row(0), disabled]));
                        view
                    })
                },
            )
            .unwrap_or_else(|e| panic!("window: {e}"))
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(view.read(cx).selected.is_none(), "no automatic selection");
            let list = window.find("list-focus").bounds();
            assert!(window.try_find("configuration-detail").is_none());
            window.render_frame(cx);
            let first = window.find("configuration:profile-00000").bounds();
            let second = window.find("configuration:profile-00001").bounds();
            assert_eq!(first.top(), second.top());
            assert!(second.left() > first.right());
            assert!(first.size.width < list.size.width / 2.);
            assert_eq!(
                window.find("configuration-apply:profile-00001").label(),
                Some("Enable and apply profile-00001")
            );
            assert_eq!(
                window.find("configuration-apply:profile-00000").label(),
                Some("Apply profile-00000")
            );
            window.click("configuration:profile-00001", cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00001"));
            assert!(window.find("configuration-detail").visible());
            assert!(window.find("configuration-apply:profile-00000").visible());
            view.update(cx, |this, cx| {
                this.accept_rows(Arc::new((0..500).map(row).collect()));
                cx.notify();
            });
            window.render_frame(cx);
            let list = window.find("list-focus").bounds();
            assert!(
                list.size.height > px(400.),
                "long list uses the available height: {list:?}"
            );
            assert!(
                list.bottom() <= px(1000.),
                "long list scrolls inside the window: {list:?}"
            );
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("layout: {e}"));
        owner.shutdown();
    }
}
