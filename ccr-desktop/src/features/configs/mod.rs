//! Native configuration catalog. Domain-backed actions live in the Profiles feature.
use crate::{
    features::profiles::ProfileTools,
    locales::{is_chinese, service_error, text},
    runtime::Services,
    shell::{badge, semantics},
    theme,
};
use ccr_app::{
    config::ConfigRow,
    runtime::{
        AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
    },
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Subscription, Task,
    UniformListScrollHandle, Window, div, px, uniform_list,
};
use std::sync::Arc;

pub struct ConfigurationApp {
    services: Services,
    rows: Arc<Vec<ConfigRow>>,
    all_rows: Arc<Vec<ConfigRow>>,
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
    _search_subscription: Subscription,
    list_focus: FocusHandle,
    detail_focus: FocusHandle,
    scroll: UniformListScrollHandle,
    tools: Entity<ProfileTools>,
    _tools_subscription: Subscription,
    tools_version: u64,
    platform: Platform,
    pinned: bool,
    #[cfg(test)]
    last_rendered_count: usize,
}

impl ConfigurationApp {
    pub fn has_dirty_draft(&self, cx: &gpui_kit::App) -> bool {
        self.tools.read(cx).has_dirty_draft(cx)
    }
    pub fn read_generation(&self) -> u64 {
        self.generation
    }
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
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
            services,
            rows: Arc::new(Vec::new()),
            all_rows: Arc::new(Vec::new()),
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
            _search_subscription: subscription,
            list_focus: cx.focus_handle(),
            detail_focus: cx.focus_handle().tab_stop(false),
            scroll: UniformListScrollHandle::new(),
            tools,
            _tools_subscription: tools_subscription,
            tools_version: 0,
            platform: Platform::Claude,
            pinned: false,
            #[cfg(test)]
            last_rendered_count: 0,
        };
        this.refresh(cx);
        this
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.search.read(cx).focus_handle(cx), cx);
    }
    /// Navigation dismisses the read subscription, not the application-owned runner.
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
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
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: self.platform,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.loading = true;
        self.error = None;
        // Drop the previous platform's rows before the next catalog returns.
        self.rows = Arc::new(Vec::new());
        self.detail_open = false;
        match self.services.profile_rows(key.clone(), String::new()) {
            Ok(mut subscription) => {
                self.read_task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.loading = false;
                                match result.as_ref() {
                                    Completion::Success(rows) => {
                                        this.accept_rows(rows.clone());
                                        this.sync_target(cx);
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.rows = Arc::new(Vec::new());
                                        this.selected = None;
                                        this.detail_open = false;
                                        this.error = Some(error.message());
                                    }
                                    Completion::Cancelled => {
                                        this.error = Some("Read cancelled / 已取消读取")
                                    }
                                    Completion::WorkerFailed => {
                                        this.error = Some(
                                            "Read worker stopped. Retry. / 读取线程已停止，请重试",
                                        )
                                    }
                                }
                                cx.notify();
                            });
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
    fn accept_rows(&mut self, rows: Arc<Vec<ConfigRow>>) {
        self.all_rows = rows;
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
        self.rows = Arc::new(rows);
        if !self
            .rows
            .iter()
            .any(|row| Some(&row.name) == self.selected.as_ref())
        {
            self.selected = None;
            self.detail_open = false;
        }
    }
    fn select_offset(&mut self, down: bool, cx: &mut Context<Self>) {
        if self.rows.is_empty() || self.tools.read(cx).expanded() || self.tools.read(cx).busy() {
            return;
        }
        let current = self
            .rows
            .iter()
            .position(|row| Some(&row.name) == self.selected.as_ref());
        let next = match (current, down) {
            (Some(index), true) => (index + 1).min(self.rows.len() - 1),
            (Some(index), false) => index.saturating_sub(1),
            _ => 0,
        };
        self.selected = Some(self.rows[next].name.clone());
        self.sync_target(cx);
        self.scroll
            .scroll_to_item(next, gpui_kit::ScrollStrategy::Top);
        cx.notify();
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
            self.all_rows = Arc::new(Vec::new());
            self.selected = None;
            self.detail_open = false;
            self.refresh(cx);
        }
    }
    fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.detail_open = false;
        window.focus(&self.list_focus, cx);
        cx.notify();
    }

    fn catalog_controls(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let blocked = self.tools.read(cx).expanded() || self.tools.read(cx).busy();
        let mut controls = div()
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.));
        controls = controls.child(semantics::ghost_button(
            Button::new("configuration-filter-clear")
                .label(text(cx, "Clear filters", "清除筛选"))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.provider_filter = None;
                    this.tag_filter = None;
                    this.enabled_filter = None;
                    this.search_text.clear();
                    this.search
                        .update(cx, |state, cx| state.set_value("", window, cx));
                    this.refilter();
                    this.sync_target(cx);
                    cx.notify();
                })),
            blocked,
            false,
            cx,
        ));
        let current = self
            .all_rows
            .iter()
            .find(|row| row.is_current)
            .map(|row| row.name.as_str())
            .unwrap_or(text(cx, "None observed", "未观测到"));
        controls = controls
            .child(div().text_size(px(12.)).child(format!(
                "{}: {current}",
                text(cx, "Current runtime profile", "当前运行方案")
            )))
            .child(semantics::ghost_button(
                Button::new("profile-off")
                    .label(text(cx, "Detach current profile", "解除当前方案"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.tools
                            .update(cx, |tools, cx| tools.detach_current(window, cx))
                    })),
                blocked || !self.tools.read(cx).can_detach(),
                false,
                cx,
            ))
            .child(semantics::ghost_button(
                Button::new("configuration-manage")
                    .label(if self.management {
                        text(cx, "Back to list", "返回列表")
                    } else {
                        text(cx, "Manage catalog", "管理方案集合")
                    })
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.management = !this.management;
                        this.tools
                            .update(cx, |tools, cx| tools.set_management(this.management, cx));
                        cx.notify();
                    })),
                blocked,
                false,
                cx,
            ));
        for (enabled, id, en, zh) in [
            (None, "configuration-enabled-all", "All states", "全部状态"),
            (
                Some(true),
                "configuration-enabled-only",
                "Enabled",
                "已启用",
            ),
            (
                Some(false),
                "configuration-disabled-only",
                "Disabled",
                "已禁用",
            ),
        ] {
            controls = controls.child(badge::filter_chip(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.enabled_filter = enabled;
                        this.refilter();
                        this.sync_target(cx);
                        cx.notify();
                    })),
                self.enabled_filter == enabled,
                blocked,
                cx,
            ));
        }
        controls = controls.child(semantics::ghost_button(
            Button::new("configuration-sort-usage")
                .label(if self.sort_usage {
                    text(cx, "Sort: observed uses", "排序：已观测使用次数")
                } else {
                    text(cx, "Sort: name", "排序：名称")
                })
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.sort_usage = !this.sort_usage;
                    this.refilter();
                    this.sync_target(cx);
                    cx.notify();
                })),
            blocked,
            false,
            cx,
        ));
        let providers = self
            .all_rows
            .iter()
            .filter_map(|row| row.provider.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let tags = self
            .all_rows
            .iter()
            .flat_map(|row| row.tags.clone().unwrap_or_default())
            .collect::<std::collections::BTreeSet<_>>();
        for (index, provider) in providers.into_iter().enumerate() {
            let selected = self.provider_filter.as_ref() == Some(&provider);
            controls = controls.child(badge::filter_chip(
                Button::new(format!("configuration-provider-{index}"))
                    .label(provider.clone())
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.provider_filter = if this.provider_filter.as_ref() == Some(&provider) {
                            None
                        } else {
                            Some(provider.clone())
                        };
                        this.refilter();
                        this.sync_target(cx);
                        cx.notify();
                    })),
                selected,
                blocked,
                cx,
            ));
        }
        for (index, tag) in tags.into_iter().enumerate() {
            let selected = self.tag_filter.as_ref() == Some(&tag);
            controls = controls.child(badge::filter_chip(
                Button::new(format!("configuration-tag-{index}"))
                    .label(tag.clone())
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.tag_filter = if this.tag_filter.as_ref() == Some(&tag) {
                            None
                        } else {
                            Some(tag.clone())
                        };
                        this.refilter();
                        this.sync_target(cx);
                        cx.notify();
                    })),
                selected,
                blocked,
                cx,
            ));
        }
        controls.into_any_element()
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
            row.model.clone().unwrap_or_else(|| unknown.into()),
        ),
        (
            text(cx, "Authentication", "认证方式"),
            row.auth_mode.clone().unwrap_or_else(|| unknown.into()),
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
                        .text_size(px(12.))
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .child(value),
        );
    }
    let credential = if row.auth_token.is_empty() {
        text(cx, "Not set", "未设置")
    } else {
        row.auth_token.as_str()
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
        let split = window.viewport_size().width >= px(theme::SPLIT_WIDTH);
        let muted = cx.theme().muted_foreground;
        let selected = self.selected.clone();
        let count = self.rows.len();
        let list = uniform_list(
            "configuration-list",
            count,
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                #[cfg(test)]
                {
                    this.last_rendered_count = range.len();
                }
                range
                    .map(|index| {
                        let row = &this.rows[index];
                        let name = row.name.clone();
                        let selected_name = name.clone();
                        let a11y_name = if row.is_current {
                            format!("{name}, {}", text(cx, "current", "当前"))
                        } else if !row.enabled {
                            format!("{name}, {}", text(cx, "disabled", "已禁用"))
                        } else {
                            name.clone()
                        };
                        let weak = cx.weak_entity();
                        let is_selected = selected.as_ref() == Some(&name);
                        let background = if is_selected {
                            cx.theme().secondary
                        } else {
                            cx.theme().background
                        };
                        let mut item = div()
                            .id(SharedString::from(format!("configuration:{}", row.name)))
                            .test_support()
                            .role(gpui_kit::Role::ListBoxOption)
                            .aria_label(a11y_name)
                            .aria_selected(is_selected)
                            .aria_position_in_set(index + 1)
                            .aria_size_of_set(count)
                            .on_a11y_action(
                                gpui_kit::AccessibleAction::Click,
                                move |_, window, cx| {
                                    let _ = weak.update(cx, |this, cx| {
                                        if this.tools.read(cx).expanded()
                                            || this.tools.read(cx).busy()
                                        {
                                            return;
                                        }
                                        this.selected = Some(selected_name.clone());
                                        this.sync_target(cx);
                                        this.detail_open = true;
                                        window.focus(
                                            if split {
                                                &this.list_focus
                                            } else {
                                                &this.detail_focus
                                            },
                                            cx,
                                        );
                                        cx.notify();
                                    });
                                },
                            )
                            .h(px(60.))
                            .w_full()
                            .px(px(theme::CONTROL_GAP))
                            .flex()
                            .items_center()
                            .gap(px(theme::SECTION_GAP))
                            .bg(background)
                            .rounded(px(theme::SELECTION_RADIUS))
                            .hover(|style| style.bg(cx.theme().muted))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                if this.tools.read(cx).expanded() || this.tools.read(cx).busy() {
                                    return;
                                }
                                this.selected = Some(name.clone());
                                this.sync_target(cx);
                                this.detail_open = true;
                                window.focus(
                                    if split {
                                        &this.list_focus
                                    } else {
                                        &this.detail_focus
                                    },
                                    cx,
                                );
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.))
                                    .child(
                                        div()
                                            .id(format!("configuration-name:{}", row.name))
                                            .truncate()
                                            .tooltip({
                                                let name = row.name.clone();
                                                move |window, cx| {
                                                    gpui_kit::component::tooltip::Tooltip::new(
                                                        name.clone(),
                                                    )
                                                    .build(window, cx)
                                                }
                                            })
                                            .child(row.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .truncate()
                                            .text_size(px(12.))
                                            .text_color(muted)
                                            .child(format!(
                                                "{} · {} · {} · {}",
                                                row.base_url,
                                                row.model.as_deref().unwrap_or(text(
                                                    cx,
                                                    "Unknown model",
                                                    "模型未知"
                                                )),
                                                row.auth_mode.as_deref().unwrap_or(text(
                                                    cx,
                                                    "Unknown auth",
                                                    "认证未知"
                                                )),
                                                row.tags
                                                    .as_ref()
                                                    .map(|tags| tags.join(", "))
                                                    .unwrap_or_default()
                                            )),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(theme::CONTROL_GAP))
                                    .text_size(px(theme::META_SIZE))
                                    .text_color(muted)
                                    .when(row.is_current, |status| {
                                        status.child(badge::dot(cx.theme().primary))
                                    })
                                    .child(if row.is_current {
                                        text(cx, "Current", "当前")
                                    } else if !row.enabled {
                                        text(cx, "Disabled", "已禁用")
                                    } else {
                                        ""
                                    }),
                            );
                        if is_selected {
                            item = item.aria_active_descendant();
                            item = item.child(semantics::button(
                                Button::new(format!("configuration-apply:{}", row.name))
                                    .label(if row.enabled {
                                        text(cx, "Apply", "应用")
                                    } else {
                                        text(cx, "Enable and apply", "启用并应用")
                                    })
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        cx.stop_propagation();
                                        this.tools.update(cx, |tools, cx| {
                                            tools.apply_selected(window, cx)
                                        });
                                    })),
                                this.tools.read(cx).expanded() || this.tools.read(cx).busy(),
                                false,
                                cx,
                            ));
                        }
                        item.into_any_element()
                    })
                    .collect()
            }),
        )
        .track_scroll(&self.scroll)
        .h(px((count as f32 * 60.).max(60.)))
        .max_h_full()
        .w_full();
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
            let count = self.rows.len();
            if is_chinese(cx) {
                format!("{count} 个配置")
            } else if count == 1 {
                "1 profile".into()
            } else {
                format!("{count} profiles")
            }
        };
        let detail = self
            .rows
            .iter()
            .find(|row| Some(&row.name) == self.selected.as_ref());
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
            body = body.child(
                div()
                    .id("list-focus")
                    .test_support()
                    .role(gpui_kit::Role::ListBox)
                    .aria_label(text(cx, "Configuration profiles", "配置列表"))
                    .track_focus(&self.list_focus)
                    .tab_stop(true)
                    .on_key_down(cx.listener(
                        move |this, event: &gpui_kit::KeyDownEvent, window, cx| {
                            match event.keystroke.key.as_str() {
                                "down" => this.select_offset(true, cx),
                                "up" => this.select_offset(false, cx),
                                "enter" => {
                                    this.detail_open = this.selected.is_some();
                                    if this.detail_open && !split {
                                        window.focus(&this.detail_focus, cx);
                                    }
                                    cx.notify();
                                }
                                "escape" => this.back(window, cx),
                                _ => return,
                            }
                            cx.stop_propagation();
                        },
                    ))
                    .flex_grow(3.)
                    .flex_basis(px(0.))
                    // 合同：宽窗列表至少 360px；窄窗走 drill-in，不靠 min_w_0 解除下限。
                    .min_w(px(360.))
                    .min_h_0()
                    .focus(|style| style.border_color(cx.theme().ring))
                    .rounded(px(theme::SELECTION_RADIUS))
                    .child(list),
            );
        }
        if !self.management
            && !self.tools.read(cx).collection_workspace()
            && let Some(row) = detail
        {
            body = body.child(
                div()
                    .id("configuration-detail")
                    .role(gpui_kit::Role::Region)
                    .aria_label(text(cx, "Profile details", "配置详情"))
                    .track_focus(&self.detail_focus)
                    .tab_stop(false)
                    .flex_grow(2.)
                    .flex_basis(px(0.))
                    // 合同：宽窗详情至少 320px。
                    .min_w(px(320.))
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(px(theme::SECTION_GAP))
                    .child(self.tools.clone())
                    .when(!self.tools.read(cx).expanded(), |pane| {
                        pane.child(semantics::ghost_button(
                            Button::new("configuration-back")
                                .label(text(cx, "Back to profiles", "返回配置列表"))
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| this.back(window, cx))),
                            false,
                            false,
                            cx,
                        ))
                        .child(
                            div()
                                .text_size(px(theme::TITLE_SIZE))
                                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                .child(row.name.clone()),
                        )
                        .child(div().text_color(muted).child(row.description.clone()))
                        .child(profile_details(row, cx))
                    })
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
        if split
            && detail.is_none()
            && !self.rows.is_empty()
            && !self.management
            && !self.tools.read(cx).collection_workspace()
        {
            body = body.child(
                div()
                    .flex_grow(2.)
                    .flex_shrink(1.)
                    .flex_basis(px(0.))
                    .min_w(px(320.))
                    .self_start()
                    .child(badge::empty_panel(
                        "configuration-empty-detail",
                        gpui_kit::component::IconName::Eye,
                        text(cx, "Select a profile", "选择一个方案"),
                        text(
                            cx,
                            "Selection shows details; it does not apply the profile.",
                            "选择仅显示详情，不会应用方案。",
                        ),
                        None,
                        cx,
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
            .text_size(px(theme::BODY_SIZE))
            .when(!self.pinned, |root| {
                root.child(
                    div()
                        .id("configuration-heading")
                        .role(gpui_kit::Role::Heading)
                        .aria_label(text(cx, title_en, title_zh))
                        .aria_level(1)
                        .text_size(px(theme::TITLE_SIZE))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .child(text(cx, title_en, title_zh)),
                )
            })
            .child(div().text_color(muted).child(text(
                cx,
                "Local profile catalog · Changes use domain validation and backups",
                "本地配置目录 · 修改遵循领域校验和备份规则",
            )))
            .when(show_toolbar, |root| {
                root.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(theme::CONTROL_GAP))
                        .child(badge::search_field(
                            Input::new(&self.search)
                                .disabled(
                                    self.tools.read(cx).expanded() || self.tools.read(cx).busy(),
                                )
                                .id("configuration-filter")
                                .aria_label(text(
                                    cx,
                                    "Search name, endpoint or model",
                                    "搜索名称、端点或模型",
                                )),
                            cx,
                        ))
                        .child(semantics::ghost_button(
                            Button::new("configuration-refresh")
                                .icon(gpui_kit::component::IconName::RefreshCw)
                                .label(if self.loading {
                                    text(cx, "Reading", "读取中")
                                } else if self.error.is_some() {
                                    text(cx, "Retry", "重试")
                                } else {
                                    text(cx, "Refresh", "刷新")
                                })
                                .ghost()
                                .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                            self.loading,
                            self.loading,
                            cx,
                        )),
                )
            })
            .child(
                div()
                    .id("configuration-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(status.clone())
                    .a11y_synthetic_children(|tree| {
                        tree.parent_node()
                            .set_live(gpui_kit::accesskit::Live::Polite)
                    })
                    .text_color(if self.error.is_some() {
                        cx.theme().danger
                    } else {
                        muted
                    })
                    .child(status),
            )
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
            .when(show_toolbar, |root| {
                root.child(semantics::button(
                    Button::new("profile-create")
                        .label(text(cx, "New profile", "新建方案"))
                        .primary()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.tools
                                .update(cx, |tools, cx| tools.new_profile(window, cx))
                        })),
                    self.tools.read(cx).expanded() || self.tools.read(cx).busy(),
                    false,
                    cx,
                ))
                .child(self.catalog_controls(cx))
            })
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
            window.click("configuration-provider-1", cx);
            window.click("configuration-tag-1", cx);
            window.click("configuration-disabled-only", cx);
            window.click("configuration:profile-00001", cx);
            assert!(window.try_find("configuration-filter").is_none());
            assert!(window.try_find("list-focus").is_none());
            window.click("configuration-back", cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00001"));
            assert_eq!(view.read(cx).provider_filter.as_deref(), Some("provider-b"));
            assert_eq!(view.read(cx).tag_filter.as_deref(), Some("tag-b"));
            assert_eq!(window.find("list-focus").focused(), Some(true));
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
            window.click("configuration-filter-clear", cx);
            assert_eq!(
                window.find("configuration-status").label(),
                Some("2 profiles")
            );
            assert!(view.read(cx).selected.is_none());
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
            assert!(view.read(cx).last_rendered_count > 0);
            assert!(
                view.read(cx).last_rendered_count < 100,
                "virtualization must render only a visible range"
            );
            let list_bounds = window.find("list-focus").bounds();
            let first_row = window.find("configuration:profile-00000").bounds();
            assert!(first_row.size.width >= list_bounds.size.width - px(4.),
                "row hitbox must fill the list, not shrink to its text: row={first_row:?}, list={list_bounds:?}");
            // Click far to the right of the short label, inside the real list.
            window.click_at("list-focus", gpui_kit::point(
                list_bounds.size.width - px(16.),
                first_row.center().y - list_bounds.origin.y,
            ), cx);
            assert!(view.read(cx).detail_open, "the blank part of a row must open its detail");
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00000"));
            window.click("configuration-back", cx);
            view.update(cx, |this, cx| window.focus(&this.list_focus, cx));
            window.press("down", cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00001"));
            window.press("enter", cx);
            assert!(view.read(cx).detail_open);
            assert!(window.find("configuration-back").visible());
            window.press("escape", cx);
            assert!(!view.read(cx).detail_open);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00001"));
            window.press("enter", cx);
            window.click("configuration-back", cx);
            assert!(!view.read(cx).detail_open);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00001"));
            view.update(cx, |this, cx| {
                this.accept_rows(Arc::new(vec![row(20), row(1)]));
                cx.notify();
            });
            window.render_frame(cx);
            assert_eq!(
                view.read(cx).selected.as_deref(),
                Some("profile-00001"),
                "identity follows domain name, not the old index"
            );
            window.press("down", cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00020"));
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("interaction: {e}"));
        owner.shutdown();
    }
}
