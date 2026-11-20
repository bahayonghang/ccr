use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FilterPanel {
    State,
    Group,
}

impl ConfigurationApp {
    pub(super) fn count_label(&self, cx: &gpui_kit::App) -> String {
        let (hits, total) = (self.rows.len(), self.all_rows.len());
        if is_chinese(cx) {
            format!("{hits} / {total} 个方案")
        } else {
            format!(
                "{hits} / {total} {}",
                if total == 1 { "profile" } else { "profiles" }
            )
        }
    }

    fn panel_toggle(
        &self,
        panel: FilterPanel,
        id: &'static str,
        label: String,
        blocked: bool,
        cx: &mut Context<Self>,
    ) -> Button {
        badge::filter_chip(
            Button::new(id)
                .label(label)
                .dropdown_caret(true)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.filter_panel = (this.filter_panel != Some(panel)).then_some(panel);
                    cx.notify();
                })),
            self.filter_panel == Some(panel),
            blocked,
            cx,
        )
    }

    fn group_options(&self, blocked: bool, cx: &mut Context<Self>) -> gpui_kit::Div {
        let mut panel = div().flex().flex_col().gap(px(6.));
        for (provider, id, label) in [
            (
                true,
                "configuration-provider-options",
                text(cx, "Providers", "供应商"),
            ),
            (false, "configuration-tag-options", text(cx, "Tags", "标签")),
        ] {
            let values = if provider {
                self.all_rows
                    .iter()
                    .filter_map(|r| r.provider.as_ref())
                    .collect::<BTreeSet<_>>()
            } else {
                self.all_rows
                    .iter()
                    .flat_map(|r| r.tags.iter().flatten())
                    .collect()
            };
            if values.is_empty() {
                continue;
            }
            let mut options = div()
                .id(id)
                .flex()
                .flex_1()
                .min_w_0()
                .overflow_x_scroll()
                .gap(px(4.));
            for (index, value) in values.into_iter().enumerate() {
                let selected = if provider {
                    &self.provider_filter
                } else {
                    &self.tag_filter
                };
                let active = selected.as_ref() == Some(value);
                let value = value.clone();
                let id = format!(
                    "configuration-{}-{index}",
                    if provider { "provider" } else { "tag" }
                );
                options = options.child(badge::filter_chip(
                    Button::new(id).label(value.clone()).on_click(cx.listener(
                        move |this, _, _, cx| {
                            let target = if provider {
                                &mut this.provider_filter
                            } else {
                                &mut this.tag_filter
                            };
                            *target = (target.as_ref() != Some(&value)).then(|| value.clone());
                            this.refilter();
                            this.sync_target(cx);
                            cx.notify();
                        },
                    )),
                    active,
                    blocked,
                    cx,
                ));
            }
            panel = panel.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(label)
                    .child(options),
            );
        }
        panel.child(
            div().flex().child(badge::filter_chip(
                Button::new("configuration-sort-usage")
                    .label(if self.sort_usage {
                        text(cx, "Sort by observed uses", "按使用次数排序")
                    } else {
                        text(cx, "Sort by name", "按名称排序")
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sort_usage = !this.sort_usage;
                        this.refilter();
                        this.sync_target(cx);
                        cx.notify();
                    })),
                self.sort_usage,
                blocked,
                cx,
            )),
        )
    }

    pub(super) fn toolbar(
        &mut self,
        status: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let blocked = self.tools.read(cx).expanded() || self.tools.read(cx).busy();
        let placeholder = text(cx, "Search name, host or model", "搜索名称、主机或模型");
        if self.search_language != Some(is_chinese(cx)) {
            self.search_language = Some(is_chinese(cx));
            self.search.update(cx, |state, cx| {
                state.set_placeholder(placeholder, window, cx)
            });
        }
        let state = match self.enabled_filter {
            Some(true) => text(cx, "Enabled", "已启用"),
            Some(false) => text(cx, "Disabled", "已禁用"),
            None => text(cx, "Status", "状态"),
        };
        let mut root = div().flex_shrink_0().flex().flex_col().gap(px(6.));
        let status_label = div()
            .id("configuration-status")
            .test_support()
            .role(gpui_kit::Role::Status)
            .aria_label(status.clone())
            .a11y_synthetic_children(|tree| {
                tree.parent_node()
                    .set_live(gpui_kit::accesskit::Live::Polite)
            })
            .when(self.error.is_some(), |s| s.text_color(cx.theme().danger))
            .child(status);
        let refresh_label = if self.loading {
            text(cx, "Reading", "读取中")
        } else if self.error.is_some() {
            text(cx, "Retry", "重试")
        } else {
            text(cx, "Refresh", "刷新")
        };
        let refresh = semantics::ghost_button(
            Button::new("configuration-refresh")
                .icon(gpui_kit::component::IconName::RefreshCw)
                .tooltip(refresh_label)
                .accessibility_label(refresh_label)
                .xsmall()
                .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
            self.loading || blocked,
            self.loading,
            cx,
        );
        let left = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(6.))
            .child(
                badge::search_field(
                    Input::new(&self.search)
                        .disabled(blocked)
                        .id("configuration-filter")
                        .aria_label(placeholder),
                    cx,
                )
                .small(),
            )
            .child(self.panel_toggle(
                FilterPanel::State,
                "configuration-filter-state",
                state.into(),
                blocked,
                cx,
            ))
            .child(self.panel_toggle(
                FilterPanel::Group,
                "configuration-filter-group",
                text(cx, "Filters & sort", "筛选与排序").into(),
                blocked,
                cx,
            ))
            .child(status_label)
            .child(refresh);
        let right = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(4.))
            .child(semantics::button(
                Button::new("profile-create")
                    .small()
                    .compact()
                    .label(text(cx, "New profile", "新建方案"))
                    .primary()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.tools.update(cx, |t, cx| t.new_profile(window, cx))
                    })),
                blocked,
                false,
                cx,
            ))
            .child(semantics::ghost_button(
                Button::new("configuration-manage")
                    .small()
                    .compact()
                    .label(if self.management {
                        text(cx, "Back to list", "返回列表")
                    } else {
                        text(cx, "Manage", "集合管理")
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.management = !this.management;
                        this.filter_panel = None;
                        this.tools
                            .update(cx, |t, cx| t.set_management(this.management, cx));
                        cx.notify();
                    })),
                blocked,
                false,
                cx,
            ));
        root = root.child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap(px(6.))
                .child(left)
                .child(right),
        );
        if let Some(panel) = self.filter_panel {
            root = match panel {
                FilterPanel::Group => root.child(self.group_options(blocked, cx)),
                FilterPanel::State => {
                    let mut options = div().flex().gap(px(4.));
                    for (value, id, label) in [
                        (
                            None,
                            "configuration-enabled-all",
                            text(cx, "All states", "全部状态"),
                        ),
                        (
                            Some(true),
                            "configuration-enabled-only",
                            text(cx, "Enabled", "已启用"),
                        ),
                        (
                            Some(false),
                            "configuration-disabled-only",
                            text(cx, "Disabled", "已禁用"),
                        ),
                    ] {
                        options = options.child(badge::filter_chip(
                            Button::new(id).label(label).on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.enabled_filter = value;
                                    this.filter_panel = None;
                                    this.refilter();
                                    this.sync_target(cx);
                                    cx.notify();
                                },
                            )),
                            self.enabled_filter == value,
                            blocked,
                            cx,
                        ));
                    }
                    root.child(options)
                }
            };
        }
        let has_filters = !self.search_text.is_empty()
            || self.enabled_filter.is_some()
            || self.provider_filter.is_some()
            || self.tag_filter.is_some();
        let mut summary = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.))
            .text_size(theme::text_size(13.))
            .text_color(cx.theme().muted_foreground);
        if self.rows.is_empty() && !self.loading && self.error.is_none() {
            summary = summary.child(self.count_label(cx));
        }
        for (kind, value) in [
            (
                "search",
                (!self.search_text.is_empty()).then_some(self.search_text.as_str()),
            ),
            ("state", self.enabled_filter.map(|_| state)),
            ("provider", self.provider_filter.as_deref()),
            ("tag", self.tag_filter.as_deref()),
        ] {
            if let Some(value) = value {
                summary = summary.child(semantics::ghost_button(
                    Button::new(format!("configuration-remove-{kind}"))
                        .xsmall()
                        .label(format!("{value} ×"))
                        .accessibility_label(format!(
                            "{} {value}",
                            text(cx, "Remove filter", "移除筛选")
                        ))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            match kind {
                                "search" => {
                                    this.search_text.clear();
                                    this.search.update(cx, |s, cx| s.set_value("", window, cx));
                                }
                                "state" => this.enabled_filter = None,
                                "provider" => this.provider_filter = None,
                                _ => this.tag_filter = None,
                            }
                            this.refilter();
                            this.sync_target(cx);
                            cx.notify();
                        })),
                    blocked,
                    false,
                    cx,
                ));
            }
        }
        if has_filters {
            summary = summary.child(semantics::ghost_button(
                Button::new("configuration-filter-clear")
                    .xsmall()
                    .label(text(cx, "Clear filters", "清除筛选"))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.provider_filter = None;
                        this.tag_filter = None;
                        this.enabled_filter = None;
                        this.filter_panel = None;
                        this.search_text.clear();
                        this.search.update(cx, |s, cx| s.set_value("", window, cx));
                        this.refilter();
                        this.sync_target(cx);
                        cx.notify();
                    })),
                blocked,
                false,
                cx,
            ));
        }
        root.when(
            has_filters || (self.rows.is_empty() && !self.loading && self.error.is_none()),
            |root| root.child(summary),
        )
        .into_any_element()
    }
}
