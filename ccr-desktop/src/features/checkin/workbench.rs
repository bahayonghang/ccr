//! Local scope, first-use actions and navigation of the check-in workbench.
use super::operations::EditorKind;
use super::*;
use crate::shell::{badge, page_header};
use gpui_kit::component::{ActiveTheme, IconName, input::Input};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CheckinTab {
    Accounts,
    Providers,
    Records,
    Transfer,
}

impl CheckinView {
    pub(super) fn switch_tab(
        &mut self,
        tab: CheckinTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.tab = tab;
        self.provider_detail = None;
        self.navigation.clear();
        self.editor = None;
        self.catalog = false;
        self.confirmation = None;
        self.cdk = None;
        self.extended = None;
        self.selected = None;
        self.operation = self.services.checkin_operation_state();
        if tab == CheckinTab::Transfer {
            self.open_extended(window, cx);
        }
        cx.notify();
    }

    fn tabs(&self, cx: &mut Context<Self>) -> gpui_kit::Div {
        let mut tabs = page_header::capsule(cx);
        for (tab, id, en, zh) in [
            (
                CheckinTab::Accounts,
                "checkin-tab-accounts",
                "Accounts",
                "账户",
            ),
            (
                CheckinTab::Providers,
                "checkin-tab-providers",
                "Providers",
                "提供商",
            ),
            (
                CheckinTab::Records,
                "checkin-tab-records",
                "Check-in records",
                "签到记录",
            ),
            (
                CheckinTab::Transfer,
                "checkin-tab-transfer",
                "Import & export",
                "导入导出",
            ),
        ] {
            tabs = tabs.child(semantics::tab(
                Button::new(id).ghost().label(text(cx, en, zh)).on_click(
                    cx.listener(move |this, _, window, cx| this.switch_tab(tab, window, cx)),
                ),
                self.tab == tab,
                false,
                cx,
            ));
        }
        tabs
    }

    fn add_button(&self, provider: bool, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let (id, en, zh) = if provider {
            ("checkin-new-provider", "Add provider", "添加提供商")
        } else {
            ("checkin-new-account", "Add account", "添加账户")
        };
        self.origin_control(
            id,
            semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .primary()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.remember_editor_source(id);
                        this.edit(
                            if provider {
                                EditorKind::Provider(None)
                            } else {
                                EditorKind::Account(None)
                            },
                            window,
                            cx,
                        )
                    })),
                self.operation.busy,
                false,
                cx,
            ),
        )
    }

    fn first_use(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let has_provider = self.data().is_some_and(|data| !data.providers.is_empty());
        let (id, title, body) = if matches!(self.snapshot, Some(CheckinSnapshot::NoDatabase)) {
            (
                "checkin-absent",
                text(
                    cx,
                    "No local check-in database was found.",
                    "尚无本机签到数据库。",
                ),
                text(
                    cx,
                    "Browsing does not create a database. Add a provider to get started.",
                    "浏览不会创建数据库。添加提供商即可开始。",
                ),
            )
        } else if has_provider {
            (
                "checkin-empty",
                text(cx, "No saved check-in accounts.", "尚无签到账户。"),
                text(
                    cx,
                    "Add an account using one of your configured providers.",
                    "选择已配置的提供商，添加第一个账户。",
                ),
            )
        } else {
            (
                "checkin-empty",
                text(
                    cx,
                    "No providers configured in the local database.",
                    "本机签到数据库尚未配置提供商。",
                ),
                text(
                    cx,
                    "Choose a built-in provider or enter a custom endpoint.",
                    "选择内置提供商，或填写自定义提供商地址。",
                ),
            )
        };
        let empty = badge::empty_panel(
            id,
            IconName::Calendar,
            title,
            body,
            Some(self.add_button(!has_provider, cx).into_any_element()),
            cx,
        )
        .aria_label(SharedString::from(format!("{title} {body}")))
        .test_support();
        if let Some(data) = self.data().filter(|d| !d.providers.is_empty()) {
            let current = self
                .provider_filter
                .as_ref()
                .or_else(|| data.providers.first().map(|p| &p.id));
            let mut options = div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(theme::CONTROL_GAP))
                .child(text(cx, "Provider", "提供商"));
            for (index, p) in data.providers.iter().enumerate() {
                let id = p.id.clone();
                options = options.child(badge::filter_chip(
                    Button::new(format!("checkin-onboarding-provider-{index}"))
                        .label(p.name.clone())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.provider_filter = Some(id.clone());
                            cx.notify();
                        })),
                    current == Some(&p.id),
                    false,
                    cx,
                ));
            }
            div()
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP))
                .child(options)
                .child(empty)
                .into_any_element()
        } else {
            empty.into_any_element()
        }
    }

    pub(super) fn render_workbench(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let mut page = div()
            .id("checkin-page")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .p(px(theme::content_padding(window.viewport_size().width)));
        let scope = text(cx, "Local check-ins", "本机签到");
        let header = div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div()
                            .id("checkin-heading")
                            .test_support()
                            .role(gpui_kit::Role::Heading)
                            .aria_level(1)
                            .text_size(theme::text_size(theme::TITLE_SIZE))
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child(text(cx, "Check-ins", "签到管理")),
                    )
                    .child(
                        div()
                            .id("checkin-scope")
                            .test_support()
                            .role(gpui_kit::Role::Label)
                            .aria_label(scope)
                            .text_color(cx.theme().muted_foreground)
                            .child(scope),
                    ),
            )
            .child(semantics::ghost_button(
                Button::new("checkin-read")
                    .label(if self.busy {
                        text(cx, "Reading…", "读取中…")
                    } else if self.message.is_some() {
                        text(cx, "Retry local read", "重读本机数据")
                    } else {
                        text(cx, "Reload", "重新读取")
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                self.busy,
                self.busy,
                cx,
            ))
            .children(self.header_job_actions(cx));
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(theme::SECTION_GAP))
            .child(header);
        if self.busy || self.message.is_some() {
            let status = self
                .message
                .map(|m| service_error(cx, m))
                .unwrap_or_else(|| text(cx, "Reading saved accounts…", "正在读取签到账户…"));
            body = body.child(
                div()
                    .id("checkin-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(status)
                    .child(status),
            );
        }
        if let Some(data) = self.data().filter(|d| !d.accounts.is_empty()) {
            body = body.child(balances::summary(data, cx));
        }
        body = body.child(self.tabs(cx));
        if self.confirmation.is_none()
            && self.editor.is_none()
            && (self.tab == CheckinTab::Accounts || self.tab == self.job_tab)
        {
            body = body.children(self.job_report(cx));
        }
        if self.confirmation.is_some() {
            body = body.child(self.entry_confirmation(cx));
        } else if self.editor.is_some() {
            body = body
                .child(self.render_operation_result(cx))
                .children(self.render_management_editor(cx));
        } else if self.selected.is_some() && self.data().is_some() {
            body = body.child(self.render_operation_result(cx));
            let wide = window.viewport_size().width >= px(1150.)
                && matches!(self.tab, CheckinTab::Accounts | CheckinTab::Records)
                && self.extended.is_none();
            let detail = self.detail_panel(cx);
            if wide {
                body =
                    body.child(
                        div()
                            .flex()
                            .gap(px(theme::SECTION_GAP))
                            .child(div().flex_1().min_w_0().child(
                                if self.tab == CheckinTab::Records {
                                    self.records_panel(cx)
                                } else {
                                    self.accounts_panel(cx)
                                },
                            ))
                            .child(div().flex_1().min_w_0().child(detail)),
                    );
            } else {
                body = body.child(detail);
            }
        } else if self.extended.is_some() {
            body = body
                .child(self.render_operation_result(cx))
                .child(self.render_extended(window, cx));
        } else if self.snapshot.is_some() {
            body = body.child(self.render_operation_result(cx));
            match self.tab {
                CheckinTab::Accounts => {
                    if self.rows().is_empty() {
                        body = body.child(self.first_use(cx));
                    } else {
                        body = body.child(self.accounts_panel(cx));
                    }
                }
                CheckinTab::Providers => {
                    if let Some(data) = self.data().filter(|d| !d.providers.is_empty()) {
                        body = body
                            .child(div().flex().items_start().child(self.add_button(true, cx)))
                            .children(self.render_provider_rows(data, cx));
                    } else {
                        body = body.child(self.first_use(cx));
                    }
                    body = body
                        .child(badge::section_heading(
                            text(cx, "Built-in providers", "内置提供商"),
                            None,
                            cx,
                        ))
                        .child(self.builtin_catalog(cx));
                }
                CheckinTab::Records => {
                    body = body.child(self.records_panel(cx));
                }
                CheckinTab::Transfer => {}
            }
        }
        self.restore_control(window, cx);
        page = page.child(body);
        page.into_any_element()
    }

    fn accounts_panel(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut panel = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        let search = badge::search_field(
            Input::new(&self.search)
                .id("checkin-search")
                .aria_label(search_placeholder(cx)),
            cx,
        );
        let toolbar = div()
            .flex()
            .flex_wrap()
            .justify_between()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(search)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(theme::CONTROL_GAP))
                    .child(self.add_button(false, cx))
                    .child(semantics::ghost_button(
                        Button::new("checkin-tools-open")
                            .label(text(cx, "Account tools", "账户工具"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close_detail();
                                this.open_extended(window, cx);
                            })),
                        self.operation.busy,
                        false,
                        cx,
                    )),
            );
        panel = panel.child(toolbar).children(self.selection_actions(cx));
        let all = badge::filter_chip(
            Button::new("checkin-provider-filter-all")
                .label(text(cx, "All providers", "全部提供商"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.provider_filter = None;
                    this.reconcile_selection();
                    cx.notify();
                })),
            self.provider_filter.is_none(),
            false,
            cx,
        );
        let mut filters = div()
            .flex()
            .flex_wrap()
            .gap(px(theme::CONTROL_GAP))
            .child(all);
        if let Some(data) = self.data() {
            for (index, p) in data.providers.iter().enumerate() {
                let id = p.id.clone();
                filters = filters.child(badge::filter_chip(
                    Button::new(format!("checkin-provider-filter-{index}"))
                        .label(p.name.clone())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.provider_filter = Some(id.clone());
                            this.reconcile_selection();
                            cx.notify();
                        })),
                    self.provider_filter.as_ref() == Some(&p.id),
                    false,
                    cx,
                ));
            }
        }
        panel = panel.child(filters);
        let mut shown = 0;
        let mut list = div()
            .flex()
            .flex_col()
            .bg(gpui_kit::rgb(theme::palette(cx.theme().mode).surface))
            .rounded(px(theme::INPUT_RADIUS))
            .overflow_hidden();
        for (index, account) in self
            .rows()
            .iter()
            .enumerate()
            .filter(|(_, a)| self.matches_account(a))
        {
            shown += 1;
            let id = account.id.clone();
            let row_actions = self.row_job_actions(account, cx);
            let state = if account.enabled {
                text(cx, "Enabled", "已启用")
            } else {
                text(cx, "Disabled", "已停用")
            };
            let provider = account
                .provider_name
                .as_deref()
                .unwrap_or(&account.provider_id);
            let last = account
                .last_checkin_at
                .as_deref()
                .map(|time| {
                    let display = chrono::DateTime::parse_from_rfc3339(time)
                        .map(|t| {
                            t.with_timezone(&chrono::Utc)
                                .format("%Y-%m-%d %H:%M:%S UTC")
                                .to_string()
                        })
                        .unwrap_or_else(|_| time.into());
                    format!("{} {display}", text(cx, "Last check-in", "上次签到"))
                })
                .unwrap_or_else(|| {
                    text(
                        cx,
                        "No check-in is recorded for this account.",
                        "尚无签到记录",
                    )
                    .to_string()
                });
            let balance = match &account.balance {
                StoredBalance::Missing => text(cx, "No stored balance.", "无余额快照").to_string(),
                StoredBalance::Unusable => {
                    text(cx, "Balance unavailable", "余额不可用").to_string()
                }
                StoredBalance::Numbers(n) => format!(
                    "{} {}",
                    n.remaining,
                    n.currency
                        .as_deref()
                        .unwrap_or(text(cx, "unknown unit", "单位未保存"))
                ),
            };
            let meta = format!("{provider} · {state} · {last}");
            let label = format!("{} · {meta} · {balance}", account.name);
            list = list.child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(theme::CONTROL_GAP))
                    .p(px(theme::CONTROL_GAP))
                    .border_b_1()
                    .border_color(gpui_kit::rgb(theme::palette(cx.theme().mode).subtle))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .items_start()
                            .child(
                                self.origin_control(
                                    &format!("row-{}", account.id),
                                    semantics::ghost_button(
                                        Button::new(format!("checkin-account-{index}"))
                                            .label(account.name.clone())
                                            .max_w_full()
                                            .tooltip(account.name.clone())
                                            .accessibility_label(label)
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.open_account_detail(
                                                    &id,
                                                    &format!("row-{id}"),
                                                    cx,
                                                );
                                            })),
                                        false,
                                        false,
                                        cx,
                                    ),
                                ),
                            )
                            .child(
                                div()
                                    .text_size(theme::text_size(theme::META_SIZE))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(meta),
                            ),
                    )
                    .child(
                        div()
                            .font_features(theme::tabular_figures())
                            .text_right()
                            .child(balance),
                    )
                    .child(row_actions),
            );
        }
        if shown == 0 {
            let clear = semantics::ghost_button(
                Button::new("checkin-clear-filters")
                    .label(text(cx, "Clear filters", "清除筛选"))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.provider_filter = None;
                        this.search.update(cx, |s, cx| s.set_value("", window, cx));
                        this.query.clear();
                        cx.notify();
                    })),
                false,
                false,
                cx,
            );
            panel = panel.child(
                badge::empty_panel(
                    "checkin-no-matches",
                    IconName::Search,
                    text(cx, "No matching accounts", "没有匹配的账户"),
                    text(
                        cx,
                        "Try another account name or provider.",
                        "试试其他账户名称或提供商。",
                    ),
                    Some(clear.into_any_element()),
                    cx,
                )
                .test_support(),
            );
        } else {
            panel = panel.child(list);
        }
        panel.into_any_element()
    }

    fn detail_panel(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut panel = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        if let Some(account) = self
            .rows()
            .iter()
            .find(|a| Some(&a.id) == self.selected.as_ref())
        {
            let back = self.origin_control(
                "back",
                semantics::ghost_button(
                    Button::new("checkin-back")
                        .label(if self.tab == CheckinTab::Accounts {
                            text(cx, "Back to accounts", "返回账户")
                        } else {
                            text(cx, "Back", "返回")
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.close_detail();
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                ),
            );
            panel = panel.child(div().flex().items_start().child(back));
            let lines = detail_lines(cx, account);
            let mut detail = div()
                .id("checkin-detail")
                .test_support()
                .aria_label(SharedString::from(lines.join(" ")))
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP));
            if let Some(data) = self.data() {
                detail = detail.children(self.render_account_rows(data, cx));
            }
            detail = detail
                .child(self.account_diagnostics(account, cx))
                .children(lines)
                .children(self.render_selected_actions(cx));
            panel = panel.child(detail).child(self.calendar_panel(cx));
        }
        panel.into_any_element()
    }
}
