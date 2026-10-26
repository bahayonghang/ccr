//! Native configuration catalog. Domain-backed actions live in the Profiles feature.
use crate::{
    features::profiles::ProfileTools,
    locales::{service_error, text},
    runtime::Services,
    shell::semantics,
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
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Subscription, Task,
    UniformListScrollHandle, Window, div, px, uniform_list,
};
use std::sync::Arc;

pub struct ConfigurationApp {
    services: Services,
    rows: Arc<Vec<ConfigRow>>,
    selected: Option<String>,
    detail_open: bool,
    gate: ResponseGate,
    generation: u64,
    read_task: Option<Task<()>>,
    loading: bool,
    error: Option<&'static str>,
    search: Entity<InputState>,
    _search_subscription: Subscription,
    list_focus: FocusHandle,
    detail_focus: FocusHandle,
    scroll: UniformListScrollHandle,
    tools: Entity<ProfileTools>,
    _tools_subscription: Subscription,
    tools_version: u64,
    platform: Platform,
    #[cfg(test)]
    last_rendered_count: usize,
}

impl ConfigurationApp {
    pub fn read_generation(&self) -> u64 {
        self.generation
    }
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx));
        let subscription = cx.subscribe(&search, |this, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                this.refresh(cx);
            }
        });
        let tools = cx.new(|cx| ProfileTools::new(services.clone(), window, cx));
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
            selected: None,
            detail_open: false,
            gate: ResponseGate::default(),
            generation: 0,
            read_task: None,
            loading: false,
            error: None,
            search,
            _search_subscription: subscription,
            list_focus: cx.focus_handle(),
            detail_focus: cx.focus_handle().tab_stop(false),
            scroll: UniformListScrollHandle::new(),
            tools,
            _tools_subscription: tools_subscription,
            tools_version: 0,
            platform: Platform::Claude,
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
        self.tools.update(cx, |tools, cx| tools.leave(window, cx));
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
        match self
            .services
            .profile_rows(key.clone(), self.search.read(cx).value().to_string())
        {
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
        self.rows = rows;
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
        if self.rows.is_empty() {
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
                        let a11y_name = name.clone();
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
                            .aria_label(row.name.clone())
                            .aria_selected(is_selected)
                            .aria_position_in_set(index + 1)
                            .aria_size_of_set(count)
                            .on_a11y_action(
                                gpui_kit::AccessibleAction::Click,
                                move |_, window, cx| {
                                    let _ = weak.update(cx, |this, cx| {
                                        this.selected = Some(a11y_name.clone());
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
                            .h(px(theme::ROW_HEIGHT))
                            .w_full()
                            .px(px(theme::CONTROL_GAP))
                            .flex()
                            .items_center()
                            .gap(px(theme::SECTION_GAP))
                            .bg(background)
                            .rounded(px(theme::SELECTION_RADIUS))
                            .hover(|style| style.bg(cx.theme().muted))
                            .on_click(cx.listener(move |this, _, window, cx| {
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
                            .child(div().flex_1().overflow_hidden().child(row.name.clone()))
                            .child(
                                div()
                                    .text_size(px(theme::META_SIZE))
                                    .text_color(muted)
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
                        }
                        item.into_any_element()
                    })
                    .collect()
            }),
        )
        .track_scroll(&self.scroll)
        .flex_1()
        .size_full();
        let status = if self.loading {
            text(cx, "Reading catalog", "正在读取配置").into()
        } else if let Some(error) = self.error {
            service_error(cx, error).to_string()
        } else if self.rows.is_empty() {
            text(cx, "No matching profiles", "暂无匹配配置").into()
        } else {
            format!("{} {}", self.rows.len(), text(cx, "profiles", "个配置"))
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
        if split || !show_detail {
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
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .border_1()
                    .border_color(cx.theme().input)
                    .focus(|style| style.border_color(cx.theme().ring))
                    .rounded(px(theme::SELECTION_RADIUS))
                    .child(list),
            );
        }
        if show_detail && let Some(row) = detail {
            body = body.child(
                div()
                    .id("configuration-detail")
                    .role(gpui_kit::Role::Region)
                    .aria_label(text(cx, "Profile details", "配置详情"))
                    .track_focus(&self.detail_focus)
                    .tab_stop(false)
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(theme::SECTION_GAP))
                    .child(semantics::button(
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
                    .child(div().child(format!(
                        "{}: {}",
                        text(cx, "Endpoint", "端点"),
                        if row.base_url.is_empty() {
                            text(cx, "Not set", "未设置")
                        } else {
                            row.base_url.as_str()
                        }
                    )))
                    .child(div().child(format!(
                                "{}: {}",
                                text(cx, "Model", "模型"),
                                row.model
                                    .as_deref()
                                    .unwrap_or(text(cx, "Not set", "未设置"))
                            )))
                    .child(div().child(format!(
                        "{}: {}",
                        text(cx, "Credential", "凭据"),
                        row.auth_token
                    )))
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
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_w_0()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_size(px(theme::BODY_SIZE))
            .child(
                div()
                    .id("configuration-heading")
                    .role(gpui_kit::Role::Heading)
                    .aria_label(text(cx, "Configurations", "配置"))
                    .aria_level(1)
                    .text_size(px(theme::TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Configurations", "配置")),
            )
            .child(div().text_color(muted).child(text(
                cx,
                "Local profile catalog · Changes use domain validation and backups",
                "本地配置目录 · 修改遵循领域校验和备份规则",
            )))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div().flex_1().child(
                            Input::new(&self.search)
                                .id("configuration-filter")
                                .aria_label(text(cx, "Filter profiles by name", "按名称筛选配置")),
                        ),
                    )
                    .child(semantics::button(
                        Button::new("configuration-refresh")
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
            .child(
                div()
                    .id("configuration-status")
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
            .child(
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
            .child(self.tools.clone())
            .child(if self.tools.read(cx).expanded() {
                div().into_any_element()
            } else {
                body.into_any_element()
            })
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
            small_fast_model: None,
            is_current: index == 0,
            is_default: false,
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
            usage_count: 0,
            enabled: true,
        }
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
            window.press("up", cx);
            assert_eq!(view.read(cx).selected.as_deref(), Some("profile-00020"));
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("interaction: {e}"));
        owner.shutdown();
    }
}
